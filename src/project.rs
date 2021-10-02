use crate::logger::{command_msg, dependency_msg, error, exit, file_msg, message};
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use inflector::Inflector;
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "template"]
struct Asset;

pub fn add_dependency(
    project_dir: &std::path::PathBuf,
    name: &str,
    version: toml::Value,
) -> Result<(), std::io::Error> {
    let mut path = std::path::PathBuf::from(project_dir);
    path.push("Cargo.toml");

    let toml: String = std::fs::read_to_string(&path)?;

    let mut parsed_toml = toml.parse::<toml::Value>().unwrap();

    // println!("Parsed toml:\n{:#?}", &parsed_toml);

    let root: &mut toml::value::Table = parsed_toml.as_table_mut().unwrap();

    let deps_table: &mut toml::value::Table = root
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();
    deps_table.insert(name.into(), version);

    let updated_toml = toml::to_string(&parsed_toml).unwrap();

    dependency_msg(name);

    std::fs::write(&path, updated_toml)?;

    Ok(())
}

pub fn update_cargo_toml(project_dir: &std::path::PathBuf) -> Result<(), std::io::Error> {
    let mut path = std::path::PathBuf::from(project_dir);
    path.push("Cargo.toml");

    let toml: String = std::fs::read_to_string(&path)?;

    let mut parsed_toml = toml.parse::<toml::Value>().unwrap();

    // println!("Parsed toml:\n{:#?}", &parsed_toml);

    let root: &mut toml::value::Table = parsed_toml.as_table_mut().unwrap();

    let deps_table: &mut toml::value::Table =
        root.get_mut("package").unwrap().as_table_mut().unwrap();

    let mut project_name: Option<String> = None;

    match deps_table.get("name") {
        Some(name) => {
            project_name = Some(name.as_str().unwrap().to_string());
            let project_name_toml_value = toml::value::Value::String(project_name.clone().unwrap());
            deps_table.insert("default-run".to_string(), project_name_toml_value);
        }
        None => panic!("Could not determine project name from generated Cargo.toml"),
    };

    let updated_toml = toml::to_string(&parsed_toml).unwrap();

    let append_to_toml = format!(
        r##"
[[bin]]
name = "fullstack"
path = "bin/fullstack.rs"

[[bin]]
name = "{project_name}"
path = "backend/main.rs"
"##,
        project_name = project_name.unwrap()
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
pub fn create(project_name: &str) -> Result<()> {
    let mut project_dir: PathBuf = PathBuf::from(project_name);

    if project_dir.exists() {
        message("Directory already exists");

        project_dir = match std::fs::canonicalize(project_dir) {
            Ok(p) => p,
            Err(err) => exit("std::fs::canonicalize():", err),
        };

        let proceed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Delete directory contents?")
            .default(false)
            .interact()?;

        if proceed {
            match std::fs::remove_dir_all(&project_dir) {
                Ok(_) => {}
                Err(err) => exit("std::fs::remove_dir_all():", err),
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

    message(&format!(
        "Creating project {}",
        style(project_name).yellow()
    ));

    match std::fs::create_dir_all(&project_dir) {
        Ok(_) => {}
        Err(err) => exit("std::fs::create_dir_all():", err),
    }

    command_msg("cargo init");

    let cargo_init = std::process::Command::new("cargo")
        .current_dir(&project_dir)
        .arg("init")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !cargo_init.success() {
        error("Failed to execute `cargo init`");
        std::process::exit(1);
    }

    // cleanup: remove src/main.rs
    command_msg("rm src/main.rs");
    let mut main_file = PathBuf::from(project_dir.clone());
    main_file.push("src");
    main_file.push("main.rs");
    std::fs::remove_file(main_file)?;

    // cleanup: remove src/
    command_msg("rmdir src/main.rs");
    let mut src_folder = PathBuf::from(project_dir.clone());
    src_folder.push("src");
    std::fs::remove_dir(src_folder)?;

    update_cargo_toml(&project_dir)?;

    add_dependency(
        &project_dir,
        "actix-files",
        toml::Value::String("0.5.0".into()),
    )?;
    add_dependency(
        &project_dir,
        "actix-http",
        toml::Value::String("2.2.0".into()),
    )?;
    add_dependency(
        &project_dir,
        "actix-web",
        toml::Value::String("3.3.2".into()),
    )?;
    add_dependency(
        &project_dir,
        "actix-web-httpauth",
        toml::Value::String("0.5.0".into()),
    )?;
    add_dependency(&project_dir, "anyhow", toml::Value::String("1.0.33".into()))?;
    add_dependency(
        &project_dir,
        "chrono",
        "version = \"0.4.19\"\nfeatures = [\"serde\"]"
            .parse::<toml::Value>()
            .unwrap(),
    )?;
    add_dependency(
        &project_dir,
        "derive_more",
        toml::Value::String("0.99.11".into()),
    )?;
    add_dependency(&project_dir, "diesel", "version = \"1.4.5\"\nfeatures = [\"postgres\", \"uuid\", \"r2d2\", \"chrono\", \"32-column-tables\"]\ndefault-features = false".parse::<toml::Value>().unwrap())?;
    add_dependency(&project_dir, "dotenv", toml::Value::String("0.15.0".into()))?;
    add_dependency(
        &project_dir,
        "env_logger",
        toml::Value::String("0.8.1".into()),
    )?;
    add_dependency(
        &project_dir,
        "futures-util",
        toml::Value::String("0.3.8".into()),
    )?;
    add_dependency(
        &project_dir,
        "jsonwebtoken",
        toml::Value::String("7.2.0".into()),
    )?;
    add_dependency(&project_dir, "lettre", toml::Value::String("0.9.5".into()))?;
    add_dependency(
        &project_dir,
        "lettre_email",
        toml::Value::String("0.9.4".into()),
    )?;
    add_dependency(
        &project_dir,
        "serde",
        "version = \"1.0.117\"\nfeatures = [\"derive\"]"
            .parse::<toml::Value>()
            .unwrap(),
    )?;
    add_dependency(
        &project_dir,
        "serde_derive",
        toml::Value::String("1.0.117".into()),
    )?;
    add_dependency(
        &project_dir,
        "serde_json",
        toml::Value::String("1.0.64".into()),
    )?;
    add_dependency(&project_dir, "tsync", toml::Value::String("1.2.1".into()))?;
    add_dependency(
        &project_dir,
        "uuid",
        "version = \"0.8.1\"\nfeatures = [\"serde\", \"v4\"]"
            .parse::<toml::Value>()
            .unwrap(),
    )?;

    /*
        Populate with project files
    */
    for filename in Asset::iter() {
        let file_contents = Asset::get(filename.as_ref()).unwrap();
        let mut file_path = std::path::PathBuf::from(&project_dir);
        file_path.push(filename.as_ref());
        let mut directory_path = std::path::PathBuf::from(&file_path);
        directory_path.pop();

        file_msg(filename.as_ref());
        std::fs::create_dir_all(directory_path)?;
        std::fs::write(file_path, file_contents)?;
    }

    command_msg("chmod +x ./bin/tsync.sh");

    let chmod_tsync = std::process::Command::new("chmod")
        .current_dir(&project_dir)
        .arg("+x")
        .arg("./bin/tsync.sh")
        .status()
        .expect("failed to execute process");

    if !chmod_tsync.success() {
        error("Failed to execute `chmod +x ./bin/tsync.sh`");
    }

    /*
        Finalize; create the initial commit.
    */

    command_msg("git init");

    let git_init = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("init")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_init.success() {
        error("Failed to execute `git init`");
        std::process::exit(1);
    }

    command_msg("git add -A");

    let git_add = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("add")
        .arg("-A")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_add.success() {
        error("Failed to execute `git add -A`");
        std::process::exit(1);
    }

    command_msg("git commit -m Initial commit");

    let git_commit = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_commit.success() {
        error("Failed to execute `git commit`");
        std::process::exit(1);
    }

    command_msg("git branch -M main");

    let git_branch = std::process::Command::new("git")
        .current_dir(&project_dir)
        .arg("branch")
        .arg("-M")
        .arg("main")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute process");

    if !git_branch.success() {
        error("Failed to execute `git branch -M main`");
        std::process::exit(1);
    }

    Ok(())
}

pub fn create_resource(resource_name: &str) -> Result<()> {
    let resource_name = resource_name.to_pascal_case();

    message(&format!("Creating resource '{}'", resource_name));

    crate::service::create(
        resource_name.as_str(),
        resource_name.to_snake_case().as_str(),
    )?;
    crate::model::create(resource_name.as_str())?;

    Ok(())
}
