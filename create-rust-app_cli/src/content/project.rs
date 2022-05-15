use crate::utils::git;
use crate::utils::logger;
use crate::content::cargo_toml::add_dependency;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use inflector::Inflector;
use rust_embed::RustEmbed;
use std::path::PathBuf;
use walkdir::WalkDir;
use update_informer::{registry, Check};
use crate::BackendFramework;

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

    let project_name: String;
    let found_project_name: bool;

    match deps_table.get("name") {
        Some(name) => {
            project_name = name.as_str().unwrap().to_string();
            found_project_name = true;
            let project_name_toml_value = toml::value::Value::String(project_name.clone());
            deps_table.insert("default-run".to_string(), project_name_toml_value);
            deps_table.insert("publish".to_string(), toml::value::Value::Boolean(false));
        }
        None => panic!("Could not determine project name from generated Cargo.toml"),
    };

    if !found_project_name { logger::error("Failed to find the project's package name! Defaulting main executable name to 'app'. Feel free to change it in `Cargo.toml`."); }

    let updated_toml = toml::to_string(&parsed_toml).unwrap();


    let append_to_toml = format!(
        r#"
[[bin]]
name = "fullstack"
path = ".cargo/bin/fullstack.rs"

[[bin]]
name = "tsync"
path = ".cargo/bin/tsync.rs"

[[bin]]
name = "{project_name}"
path = "backend/main.rs"

[profile.dev]
debug-assertions=true
"#,
        project_name = project_name
    );

    let mut final_toml = String::default();

    final_toml.push_str(&updated_toml);
    final_toml.push_str(&append_to_toml);

    std::fs::write(&path, final_toml)?;

    Ok(())
}

pub struct CreationOptions {
    pub cra_enabled_features: Vec<String>,
    pub backend_framework: BackendFramework
}

pub fn remove_non_framework_files(project_dir: &PathBuf, framework: BackendFramework) -> Result<()> {
    /* Choose framework-specific files */
    for entry in WalkDir::new(project_dir) {
        let entry = entry.unwrap();

        let file = entry.path();
        let path = file.clone().to_str().unwrap().to_string();

        if path.ends_with("+actix_web") {
            if framework != BackendFramework::ActixWeb {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if framework == BackendFramework::ActixWeb {
                let dest = file.with_extension(file.extension().unwrap().to_string_lossy().replace("+actix_web", ""));
                logger::rename_file_msg(&format!("{:#?}", &file), &format!("{:#?}", &dest));
                std::fs::rename(file, dest)?;
            };
        } else if path.ends_with("+poem") {
            if framework != BackendFramework::Poem {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if framework == BackendFramework::Poem {
                let dest = file.with_extension(file.extension().unwrap().to_string_lossy().replace("+poem", ""));
                logger::rename_file_msg(&format!("{:#?}", &file), &format!("{:#?}", &dest));
                std::fs::rename(file, dest)?;
            };
        }
    }

    Ok(())
}

/**
 * create-rust-app project generation
 */
pub fn create(project_name: &str, creation_options: CreationOptions) -> Result<()> {
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
        "Creating project {} with backend {}",
        style(project_name).yellow(),
        style(&format!("{:?}", creation_options.backend_framework)).yellow()
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


    let framework = creation_options.backend_framework;
    let cra_enabled_features = creation_options.cra_enabled_features;

    let mut enabled_features: String = cra_enabled_features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<String>>().join(", ");
    if !cra_enabled_features.is_empty() { enabled_features = ", features=[".to_string() + &enabled_features + "]"; }

    match framework {
        BackendFramework::ActixWeb => {
            add_dependency(&project_dir, "actix-files", r#"actix-files = "0.6.0""#)?;
            add_dependency(&project_dir, "actix-http", r#"actix-http = "3.0.0""#)?;
            add_dependency(&project_dir, "actix-web", r#"actix-web = "4.0.1""#)?;
            add_dependency(&project_dir, "actix-multipart", r#"actix-multipart = "0.4.0""#)?;
            add_dependency(&project_dir, "tokio", r#"tokio = { version = "1", features = ["full"] }"#)?;
        },
        BackendFramework::Poem => {
            add_dependency(&project_dir, "poem", r#"poem = { version="1.3.18", features=["anyhow", "cookie", "static-files", "multipart"] }"#)?;
            add_dependency(&project_dir, "tokio", r#"tokio = { version = "1.15.0", features = ["rt-multi-thread", "macros"] }"#)?;
            add_dependency(&project_dir, "tracing_subscriber", r#"tracing-subscriber = "0.3.7""#)?;
        }
    }
    add_dependency(&project_dir, "futures-util", r#"futures-util = "0.3.21""#)?;
    add_dependency(&project_dir, "create-rust-app", &format!("create-rust-app = {{version=\"5.1.0\"{enabled_features}}}", enabled_features=enabled_features))?;
    add_dependency(&project_dir, "serde", r#"serde = { version = "1.0.133", features = ["derive"] }"#)?;
    add_dependency(&project_dir, "serde_json", r#"serde_json = "1.0.79""#)?;
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

        logger::add_file_msg(filename.as_ref());
        std::fs::create_dir_all(directory_path)?;
        std::fs::write(file_path, file_contents)?;
    }

    remove_non_framework_files(&project_dir, framework)?;

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

pub fn create_resource(backend: BackendFramework, resource_name: &str) -> Result<()> {
    let resource_name = resource_name.to_pascal_case();

    logger::message(&format!("Creating resource '{}'", resource_name));

    crate::content::service::create(
        backend,
        &resource_name,
        &format!("services::{}::api()", &resource_name),
        &resource_name.to_snake_case(),
    )?;
    crate::content::model::create(resource_name.as_str())?;

    Ok(())
}

pub fn check_cli_version() -> Result<()>{
  let name = env!("CARGO_PKG_NAME");
  let version = env!("CARGO_PKG_VERSION");
  let informer = update_informer::new(registry::Crates, name, version);
  if let Some(new_version) = informer.check_version().ok().flatten()  {
    logger::message(&style(&format!("You are running `{name}` v{version}, which is behind the latest release ({new_version}).")).yellow().to_string());
    logger::message(&format!("If you want to update, try: {}", style("cargo install --force create-rust-app_cli").yellow()));
  } else {
    logger::message(&format!("You are running the latest version of {name} ({version})!"));
  }
  Ok(())
}
