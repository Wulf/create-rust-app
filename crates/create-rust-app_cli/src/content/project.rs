use crate::content::cargo_toml::add_dependency;
use crate::utils::git;
use crate::utils::logger;
use crate::BackendDatabase;
use crate::BackendFramework;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use inflector::Inflector;
use rust_embed::RustEmbed;
use std::path::PathBuf;
use std::time::Duration;
use update_informer::{registry, Check};
use walkdir::WalkDir;

#[derive(RustEmbed)]
#[folder = "template"]
struct Asset;

// const CRA_CARGO_TOML: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../create-rust-app/Cargo.toml"));
// fn get_current_cra_lib_version() -> String {
//     let err_message = &format!("Could not parse create-rust-app toml! Here it is:\n================\n{:#?}", CRA_CARGO_TOML);
//
//     let cargo_toml = CRA_CARGO_TOML.to_string();
//     let cargo_toml = cargo_toml.parse::<toml::Value>()
//         .expect(err_message);
//     let cargo_toml = cargo_toml
//         .as_table()
//         .expect(err_message);
//
//     let package_table = cargo_toml.get("package")
//         .expect(err_message)
//         .as_table()
//         .expect(err_message);
//
//     let version = package_table.get("version")
//         .expect(err_message)
//         .as_str()
//         .expect(err_message);
//
//     version.to_string()
// }
fn get_current_cra_lib_version() -> String {
    "11.0.1".to_string()
}

#[derive(Clone)]
struct ProjectBinary {
    name: &'static str,
    path: &'static str,
}

fn is_restricted_project_name(project_name: &str, project_binaries: &[ProjectBinary]) -> bool {
    project_binaries
        .iter()
        .map(|bin| bin.name)
        .any(|x| x == project_name)
}

fn add_bins_to_cargo_toml(
    project_dir: &std::path::PathBuf,
    creations_options: &CreationOptions,
) -> Result<()> {
    let mut path = std::path::PathBuf::from(project_dir);
    path.push("Cargo.toml");

    let toml: String = std::fs::read_to_string(&path)?;

    let mut parsed_toml = toml.parse::<toml::Value>().unwrap();

    let root: &mut toml::value::Table = parsed_toml.as_table_mut().unwrap();

    let deps_table: &mut toml::value::Table =
        root.get_mut("package").unwrap().as_table_mut().unwrap();

    let project_name: String;
    let found_project_name: bool;

    #[allow(clippy::option_if_let_else)]
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

    if !found_project_name {
        logger::error("Failed to find the project's package name! Defaulting main executable name to 'app'. Feel free to change it in `Cargo.toml`.");
    }

    let updated_toml = toml::to_string(&parsed_toml).unwrap();

    let mut project_binaries: Vec<ProjectBinary> = vec![
        ProjectBinary {
            name: "fullstack",
            path: ".cargo/bin/fullstack.rs",
        },
        ProjectBinary {
            name: "tsync",
            path: ".cargo/bin/tsync.rs",
        },
        ProjectBinary {
            name: "dsync",
            path: ".cargo/bin/dsync.rs",
        },
        ProjectBinary {
            name: "backend",
            path: ".cargo/bin/backend.rs",
        },
        ProjectBinary {
            name: "frontend",
            path: ".cargo/bin/frontend.rs",
        },
    ];

    if creations_options
        .cra_enabled_features
        .contains(&"plugin_tasks".to_string())
    {
        project_binaries.push(ProjectBinary {
            name: "queue",
            path: "backend/queue.rs",
        });
        project_binaries.push(ProjectBinary {
            name: "async_queue",
            path: "backend/async_queue.rs",
        });
    };

    let binaries_cargo_toml_string = project_binaries
        .clone()
        .iter()
        .map(|bin| {
            format!(
                r#"[[bin]]
name = "{name}"
path = "{path}"
"#,
                name = bin.name,
                path = bin.path
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let append_to_toml = format!(
        r#"
{binaries_cargo_toml_string}
[[bin]]
name = "{project_name}"
path = "backend/main.rs"

[profile.dev]
debug-assertions=true
"#
    );

    let mut final_toml = String::default();

    final_toml.push_str(&updated_toml);
    final_toml.push_str(&append_to_toml);

    std::fs::write(&path, final_toml)?;

    // check if the project name is valid
    if is_restricted_project_name(&project_name, &project_binaries) {
        logger::error(&format!(
            "Invalid project name: '{project_name}' (running `cargo run --bin {project_name}` is used by a binary generated by create-rust-app)."
        ));
        return Err(anyhow::anyhow!("Invalid project name"));
    }

    Ok(())
}

pub struct CreationOptions {
    pub cra_enabled_features: Vec<String>,
    pub backend_framework: BackendFramework,
    pub backend_database: BackendDatabase,
    pub cli_mode: bool,
}

pub fn remove_non_framework_files(
    project_dir: &PathBuf,
    framework: BackendFramework,
) -> Result<()> {
    /* Choose framework-specific files */
    for entry in WalkDir::new(project_dir) {
        let entry = entry.unwrap();

        let file = entry.path();
        let path = file.to_str().unwrap().to_string();

        if path.ends_with("+actix_web") {
            if framework != BackendFramework::ActixWeb {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if framework == BackendFramework::ActixWeb {
                let dest = file.with_extension(
                    file.extension()
                        .unwrap()
                        .to_string_lossy()
                        .replace("+actix_web", ""),
                );
                logger::rename_file_msg(&format!("{:#?}", &file), &format!("{:#?}", &dest));
                std::fs::rename(file, dest)?;
            };
        } else if path.ends_with("+poem") {
            if framework != BackendFramework::Poem {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if framework == BackendFramework::Poem {
                let dest = file.with_extension(
                    file.extension()
                        .unwrap()
                        .to_string_lossy()
                        .replace("+poem", ""),
                );
                logger::rename_file_msg(&format!("{:#?}", &file), &format!("{:#?}", &dest));
                std::fs::rename(file, dest)?;
            };
        }
    }

    Ok(())
}

pub fn remove_non_database_files(project_dir: &PathBuf, database: BackendDatabase) -> Result<()> {
    /* Choose framework-specific files */
    for entry in WalkDir::new(project_dir) {
        let entry = entry.unwrap();

        let file = entry.path();
        let path = file.to_str().unwrap().to_string();

        if path.ends_with("+database_postgres") {
            if database != BackendDatabase::Postgres {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if database == BackendDatabase::Postgres {
                let dest = file.with_extension(
                    file.extension()
                        .unwrap()
                        .to_string_lossy()
                        .replace("+database_postgres", ""),
                );
                logger::rename_file_msg(&format!("{:#?}", &file), &format!("{:#?}", &dest));
                std::fs::rename(file, dest)?;
            };
        } else if path.ends_with("+database_sqlite") {
            if database != BackendDatabase::Sqlite {
                logger::remove_file_msg(&format!("{:#?}", &file));
                std::fs::remove_file(file)?;
            };
            if database == BackendDatabase::Sqlite {
                let dest = file.with_extension(
                    file.extension()
                        .unwrap()
                        .to_string_lossy()
                        .replace("+database_sqlite", ""),
                );
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
#[allow(clippy::too_many_lines)] //TODO: refactor to reduce complexity
#[allow(clippy::cognitive_complexity)]
pub fn create(project_name: &str, creation_options: CreationOptions) -> Result<()> {
    /*
       Temporary guard until we get poem supported again
    */
    let cra_version = env!("CARGO_PKG_VERSION");
    if !cra_version.eq("9.2.2") && creation_options.backend_framework == BackendFramework::Poem {
        logger::error("Poem is not supported in this version of create-rust-app. Please use version 9.2.2 (cargo install create-rust-app_cli@9.2.2). We hope to bring back poem-web as well as other frameworks in the future.");
        std::process::exit(1);
    }

    let mut project_dir: PathBuf = PathBuf::from(project_name);

    if project_dir.exists() {
        logger::message("Directory already exists");

        project_dir = match std::fs::canonicalize(project_dir) {
            Ok(p) => p,
            Err(err) => logger::exit_error("std::fs::canonicalize():", &err),
        };

        let proceed = !creation_options.cli_mode
            && Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Delete directory contents?")
                .default(false)
                .interact()?;

        if proceed {
            match std::fs::remove_dir_all(&project_dir) {
                Ok(()) => {}
                Err(err) => logger::exit_error("std::fs::remove_dir_all():", &err),
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

    logger::message("Creating Project Directory");
    match std::fs::create_dir_all(&project_dir) {
        Ok(()) => {}
        Err(err) => logger::exit_error("std::fs::create_dir_all():", &err),
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
    let mut main_file = project_dir.clone();
    main_file.push("src");
    main_file.push("main.rs");
    std::fs::remove_file(main_file)?;

    // cleanup: remove src/
    logger::command_msg("rmdir src/main.rs");
    let mut src_folder = project_dir.clone();
    src_folder.push("src");
    std::fs::remove_dir(src_folder)?;

    add_bins_to_cargo_toml(&project_dir, &creation_options)?;

    let framework = creation_options.backend_framework;
    let database = creation_options.backend_database;
    let cra_enabled_features = creation_options.cra_enabled_features;

    let mut enabled_features: String = cra_enabled_features
        .iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<String>>()
        .join(", ");
    if !cra_enabled_features.is_empty() {
        enabled_features = ", features=[".to_string() + &enabled_features + "]";
    }

    // TODO: update dependencies to use the latest versions
    match framework {
        BackendFramework::ActixWeb => {
            add_dependency(&project_dir, "actix-files", r#"actix-files = "0.6.0""#)?;
            add_dependency(&project_dir, "actix-http", r#"actix-http = "3.6""#)?;
            add_dependency(&project_dir, "actix-web", r#"actix-web = "4.5""#)?;
            add_dependency(
                &project_dir,
                "actix-multipart",
                r#"actix-multipart = "0.6.0""#,
            )?;
            add_dependency(
                &project_dir,
                "tokio",
                r#"tokio = { version = "1", features = ["full"] }"#,
            )?;
        }
        BackendFramework::Poem => {
            add_dependency(
                &project_dir,
                "poem",
                r#"poem = { version="1.3", features=["anyhow", "cookie", "static-files", "multipart"] }"#,
            )?;
            add_dependency(
                &project_dir,
                "tokio",
                r#"tokio = { version = "1", features = ["rt-multi-thread", "macros"] }"#,
            )?;
            add_dependency(
                &project_dir,
                "tracing_subscriber",
                r#"tracing-subscriber = "0.3.7""#,
            )?;
        }
    }
    add_dependency(&project_dir, "simple_logger", r#"simple_logger = "5.0""#)?;
    add_dependency(&project_dir, "futures-util", r#"futures-util = "0.3.30""#)?;
    add_dependency(
        &project_dir,
        "serde",
        r#"serde = { version = "1", features = ["derive"] }"#,
    )?;
    add_dependency(&project_dir, "serde_json", r#"serde_json = "1""#)?;
    add_dependency(
        &project_dir,
        "chrono",
        r#"chrono = { version = "0.4.38", features = ["serde"] }"#,
    )?;
    add_dependency(&project_dir, "tsync", r#"tsync = "3""#)?;
    add_dependency(
        &project_dir,
        "dsync",
        r#"dsync = { version = "0", features = ["advanced-queries"] }"#,
    )?;
    add_dependency(
        &project_dir,
        "diesel",
        &format!(
            r#"diesel = {{ version="2.1", default-features = false, features = ["{db}", "r2d2", "chrono"] }}"#,
            db = match database {
                BackendDatabase::Postgres => "postgres",
                BackendDatabase::Sqlite => "sqlite",
            }
        ),
    )?;
    add_dependency(
        &project_dir,
        "create-rust-app",
        &format!(
            "create-rust-app = {{version=\"{version}\", default-features = false{enabled_features}}}",
            version = get_current_cra_lib_version(),
            enabled_features = enabled_features
        ),
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

        logger::add_file_msg(filename.as_ref());
        std::fs::create_dir_all(directory_path)?;
        std::fs::write(file_path, file_contents.data)?;
    }

    remove_non_framework_files(&project_dir, framework)?;
    remove_non_database_files(&project_dir, database)?;

    if database == BackendDatabase::Sqlite {
        // for sqlite, we don't want the initial diesel setup or database timezone adjustment
        let mut mig1 = PathBuf::from(&project_dir);
        mig1.push("migrations");
        let mut mig2 = mig1.clone();
        let mut mig3 = mig1.clone();
        let mut mig0 = mig1.clone();
        mig1.push("00000000000000_diesel_initial_setup");
        mig2.push("00000000000001_utc");
        mig3.push("00000000000002_todos");
        mig0.push("00000000000099_todos");
        std::fs::remove_dir(mig1)?;
        std::fs::remove_dir(mig2)?;
        std::fs::create_dir(&mig0)?;
        let mut up = mig3.clone();
        up.push("up.sql");
        let mut down = mig3.clone();
        down.push("down.sql");
        let mut new_up = mig0.clone();
        new_up.push("up.sql");
        let mut new_down = mig0.clone();
        new_down.push("down.sql");
        std::fs::rename(up, new_up)?;
        std::fs::rename(down, new_down)?;
        std::fs::remove_dir(mig3)?;

        // also, let's update the env files
        let mut env_example_file = PathBuf::from(&project_dir);
        let mut env_file = PathBuf::from(&project_dir);
        env_example_file.push(".env.example");
        env_file.push(".env");
        let contents = std::fs::read_to_string(&env_example_file)?;
        let contents = contents.replace(
            "postgres://postgres:postgres@localhost/database",
            "dev.sqlite",
        );
        std::fs::write(env_example_file, contents.clone())?;
        std::fs::write(env_file, contents)?;

        // update dsync bin
        let mut dsync_bin = PathBuf::from(&project_dir);
        dsync_bin.push(".cargo");
        dsync_bin.push("bin");
        dsync_bin.push("dsync.rs");
        crate::utils::fs::replace(
            dsync_bin.to_str().unwrap(),
            "diesel::pg::Pg",
            "diesel::sqlite::Sqlite",
        )
        .unwrap();
    }

    /*
        Initial code gen (dsync, tsync)
    */
    // logger::command_msg("cargo dsync");
    //
    // let cargo_dsync = std::process::Command::new("cargo")
    //     .current_dir(&project_dir)
    //     .arg("dsync")
    //     .stdout(std::process::Stdio::null())
    //     .status()
    //     .expect("failed to execute process");
    //
    // if !cargo_dsync.success() {
    //     logger::error("failed to execute process");
    //     std::process::exit(1);
    // }
    //
    // logger::command_msg("cargo tsync");
    //
    // let cargo_tsync = std::process::Command::new("cargo")
    //     .current_dir(&project_dir)
    //     .arg("tsync")
    //     .stdout(std::process::Stdio::null())
    //     .status()
    //     .expect("failed to execute process");
    //
    // if !cargo_tsync.success() {
    //     logger::error("failed to execute process");
    //     std::process::exit(1);
    // }

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

    // check if a git user name & email are configured, if not either ask for one or fail if in cli-mode
    logger::command_msg("git config user.name");
    let git_config_user_name = git::check_config(&project_dir, "user.name");
    if !git_config_user_name {
        logger::message("You do not have a git user name set.");

        // if being created in cli-only mode, don't ask for a new user.name,
        // just tell them how to set it and exit with status code 1
        if creation_options.cli_mode {
            logger::error("Running in non-interactive mode and git user.name not set.\nYou can set it with this command:\n\t`git config --global user.name <name>`");
            std::process::exit(1);
        }

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

            if !input.is_empty()
                && git::set_config(&project_dir, "user.name", &input)
                && git::check_config(&project_dir, "user.name")
            {
                valid_user_name = true;
            } else {
                invalid_input = true;
            }
        }
    }

    logger::command_msg("git config user.email");
    let git_config_user_email = git::check_config(&project_dir, "user.email");
    if !git_config_user_email {
        logger::message("You do not have a git user email set.");

        // if being created in cli-only mode, don't ask for a new user.email,
        // just tell them how to set it and exit with status code 1
        if creation_options.cli_mode {
            logger::error("Running in non-interactive mode and git user.email not set.\nYou can set it with this command:\n\t`git config --global user.email <email>`");
            std::process::exit(1);
        }

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

            if !input.is_empty()
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

pub fn create_resource(
    backend: BackendFramework,
    resource_name: &str,
    include_qsync_attr: bool,
) -> Result<()> {
    let resource_name = resource_name.to_pascal_case();

    logger::message(&format!("Creating resource '{resource_name}'"));

    crate::content::service::create(
        backend,
        &resource_name,
        &format!("services::{}::api()", &resource_name),
        &resource_name.to_snake_case(),
        include_qsync_attr,
    )?;

    Ok(())
}

pub fn check_cli_version() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let informer = update_informer::new(registry::Crates, name, version)
        .timeout(Duration::from_secs(2))
        .interval(Duration::ZERO);
    informer.check_version().ok().flatten().map_or_else(|| {
        logger::message(&format!("v{version}"));
    }, |new_version| {
        logger::message(&style(&format!("You are running `{name}` v{version}, which is behind the latest release ({new_version}).")).yellow().to_string());
        logger::message(&format!(
            "If you want to update, try: {}",
            style("cargo install --force create-rust-app_cli").yellow()
        ));
    });
}
