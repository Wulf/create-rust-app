use crate::utils::git;
use crate::utils::logger;
use crate::content::cargo_toml::add_dependency;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use inflector::Inflector;
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "template"]
struct Asset;

fn add_bins_to_cargo_toml(project_dir: &std::path::PathBuf) -> Result<(), std::io::Error> {
    let mut path = std::path::PathBuf::from(project_dir);
    path.push("Cargo.toml");

    let toml: String = std::fs::read_to_string(&path)?;

    let mut parsed_toml = toml.parse::<toml::Value>().unwrap();

    let root: &mut toml::value::Table = parsed_toml.as_table_mut().unwrap();

    let deps_table: &mut toml::value::Table =
        root.get_mut("package").unwrap().as_table_mut().unwrap();

    let mut project_name = "app".to_string();
    let mut found_project_name: bool = false;

    match deps_table.get("name") {
        Some(name) => {
            project_name = name.as_str().unwrap().to_string();
            found_project_name = true;
            let project_name_toml_value = toml::value::Value::String(project_name.clone());
            deps_table.insert("default-run".to_string(), project_name_toml_value);
        }
        None => panic!("Could not determine project name from generated Cargo.toml"),
    };

    if !found_project_name { logger::error("Failed to find the project's package name! Defaulting main executable name to 'app'. Feel free to change it in `Cargo.toml`."); }

    let updated_toml = toml::to_string(&parsed_toml).unwrap();

    let append_to_toml = format!(
        r#"
[[bin]]
name = "fullstack"
path = "bin/fullstack.rs"

[[bin]]
name = "{project_name}"
path = "backend/main.rs"
"#,
        project_name = project_name
    );

    let mut final_toml = String::default();

    final_toml.push_str(&updated_toml);
    final_toml.push_str(&append_to_toml);

    std::fs::write(&path, final_toml)?;

    Ok(())
}

/**
 * create-rust-app project generation
 */
pub fn create(project_name: &str, cra_enabled_features: Vec<String>) -> Result<()> {
    let mut project_dir: PathBuf = PathBuf::from(project_name);

    if project_dir.exists() {
        logger::message("Directory already exists");

        project_dir = match std::fs::canonicalize(project_dir) {
            Ok(p) => p,
            Err(err) => logger::exit("std::fs::canonicalize():", err),
        };

        let proceed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Delete directory contents?")
            .default(false)
            .interact()?;

        if proceed {
            match std::fs::remove_dir_all(&project_dir) {
                Ok(_) => {}
                Err(err) => logger::exit("std::fs::remove_dir_all():", err),
            }
        } else {
            std::process::exit(0);
        }
    }

    let project_name = project_dir
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    logger::message(&format!(
        "Creating project {}",
        style(project_name).yellow()
    ));

    match std::fs::create_dir_all(&project_dir) {
        Ok(_) => {}
        Err(err) => logger::exit("std::fs::create_dir_all():", err),
    }

    logger::command_msg("cargo init");

    let cargo_init = std::process::Command::new("cargo")
        .current_dir(&project_dir)
        .arg("init")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !cargo_init.success() {
        logger::error("Failed to execute `cargo init`");
        std::process::exit(1);
    }

    // cleanup: remove src/main.rs
    logger::command_msg("rm src/main.rs");
    let mut main_file = PathBuf::from(project_dir.clone());
    main_file.push("src");
    main_file.push("main.rs");
    std::fs::remove_file(main_file)?;

    // cleanup: remove src/
    logger::command_msg("rmdir src/main.rs");
    let mut src_folder = PathBuf::from(project_dir.clone());
    src_folder.push("src");
    std::fs::remove_dir(src_folder)?;

    add_bins_to_cargo_toml(&project_dir)?;

    let mut enabled_features: String = cra_enabled_features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<String>>().join(", ");
    if !cra_enabled_features.is_empty() { enabled_features = ", features=[".to_string() + &enabled_features + "]"; }

    add_dependency(&project_dir, "create-rust-app", &format!("create-rust-app = {{version=\"3.0.0\"{enabled_features}}}", enabled_features=enabled_features))?;
    add_dependency(&project_dir, "poem", r#"poem = { version="1.2.33", features=["anyhow", "cookie", "static-files"] }"#)?;
    add_dependency(&project_dir, "tokio", r#"tokio = { version = "1.15.0", features = ["rt-multi-thread", "macros"] }"#)?;
    add_dependency(&project_dir, "serde", r#"serde = { version = "1.0.133", features = ["derive"] }"#)?;
    add_dependency(&project_dir, "serde_json", r#"serde_json = "1.0.74""#)?;
    add_dependency(&project_dir, "chrono", r#"chrono = { version = "0.4.19", features = ["serde"] }"#)?;
    add_dependency(&project_dir, "tsync", r#"tsync = "1.2.1""#)?;
    add_dependency(&project_dir, "diesel", r#"diesel = { version="1.4.8", default-features = false, features = ["postgres", "r2d2", "chrono"] }"#)?;

    /*
        Populate with project files
    */
    for filename in Asset::iter() {
        let file_contents = Asset::get(filename.as_ref()).unwrap();
        let mut file_path = std::path::PathBuf::from(&project_dir);
        file_path.push(filename.as_ref());
        let mut directory_path = std::path::PathBuf::from(&file_path);
        directory_path.pop();

        logger::file_msg(filename.as_ref());
        std::fs::create_dir_all(directory_path)?;
        std::fs::write(file_path, file_contents)?;
    }

    /*
        Finalize; create the initial commit.
    */

    logger::command_msg("git init");

    let git_init = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("init")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_init.success() {
        logger::error("Failed to execute `git init`");
        std::process::exit(1);
    }

    logger::command_msg("git config user.name");

    let git_config_user_name = git::check_config(&project_dir, "user.name");

    if !git_config_user_name {
        logger::message("You do not have a git user name set.");

        let mut valid_user_name = false;
        let mut invalid_input = false;

        while !valid_user_name {
            let prompt_message = if invalid_input {
                "(try again) Choose a name to use when committing:"
            } else {
                "Choose a name to use when committing:"
            };
            let input: String = Input::new().with_prompt(prompt_message).interact()?;

            logger::command_msg(&format!("git config user.name {:#?}", &input));

            if input.len() > 0
                && git::set_config(&project_dir, "user.name", &input)
                && git::check_config(&project_dir, "user.name")
            {
                valid_user_name = true;
            } else {
                invalid_input = true;
            }
        }
    }

    let git_config_user_email = git::check_config(&project_dir, "user.email");

    if !git_config_user_email {
        logger::message("You do not have a git user email set.");

        let mut valid_user_email = false;
        let mut invalid_input = false;

        while !valid_user_email {
            let prompt_message = if invalid_input {
                "(try again) Choose an email to use when committing:"
            } else {
                "Choose an email to use when committing:"
            };
            let input: String = Input::new().with_prompt(prompt_message).interact()?;

            logger::command_msg(&format!("git config user.email {:#?}", &input));

            if input.len() > 0
                && git::set_config(&project_dir, "user.email", &input)
                && git::check_config(&project_dir, "user.email")
            {
                valid_user_email = true;
            } else {
                invalid_input = true;
            }
        }
    }

    logger::command_msg("git add -A");

    let git_add = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("add")
        .arg("-A")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_add.success() {
        logger::error("Failed to execute `git add -A`");
        std::process::exit(1);
    }

    logger::command_msg("git commit -m Initial commit");

    let git_commit = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_commit.success() {
        logger::error("Failed to execute `git commit`");
        std::process::exit(1);
    }

    logger::command_msg("git branch -M main");

    let git_branch = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("branch")
        .arg("-M")
        .arg("main")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_branch.success() {
        logger::error("Failed to execute `git branch -M main`");
        std::process::exit(1);
    }

    Ok(())
}

pub fn create_resource(resource_name: &str) -> Result<()> {
    let resource_name = resource_name.to_pascal_case();

    logger::message(&format!("Creating resource '{}'", resource_name));

    crate::content::service::create(
        &resource_name,
        &format!("services::{}::api()", &resource_name),
        &resource_name.to_snake_case(),
    )?;
    crate::content::model::create(resource_name.as_str())?;

    Ok(())
}
