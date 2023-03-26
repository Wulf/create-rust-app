use crate::BackendDatabase;
use console::style;

pub fn message(msg: &str) {
    println!("[{}] {}", style("create-rust-app").blue(), msg)
}

pub fn command_msg(command: &str) {
    message(&format!("Running `{}`", style(command).yellow()));
}

pub fn add_file_msg(file: &str) {
    message(&format!("Adding {}", style(file).yellow()));
}

pub fn register_service_msg(service_name: &str) {
    message(&format!(
        "Registering service {}",
        style(service_name).yellow()
    ));
}

pub fn modify_file_msg(file: &str) {
    message(&format!("Modifying {}", style(file).yellow()));
}

pub fn remove_file_msg(file: &str) {
    message(&format!("Removing {}", style(file).yellow()));
}

pub fn rename_file_msg(src: &str, dest: &str) {
    message(&format!(
        "Renaming {} to {}",
        style(src).yellow(),
        style(dest).yellow()
    ));
}

pub fn plugin_msg(name: &str) {
    message(&format!("Installing {} plugin", style(name).yellow()));
}

pub fn error(msg: &str) {
    message(&format!("{} {}", style("ERROR: ").red(), msg))
}

pub fn exit(msg: &str, err: std::io::Error) -> ! {
    eprintln!("{msg}: {err:?}");
    std::process::exit(1);
}

pub fn add_dependency_msg(name: &str) {
    message(&format!("Adding dependency {}", style(name).yellow()));
}

pub fn project_created_msg(install_config: crate::plugins::InstallConfig) {
    let project_name = install_config.project_name;

    command_msg("cargo watch --help\t# checking cargo-watch installation");

    let is_cargo_watch_installed = match std::process::Command::new("cargo")
        .arg("watch")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    command_msg("diesel --help\t# checking diesel_cli installation");

    let is_diesel_cli_installed = match std::process::Command::new("diesel")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    message(&format!(
        "{}",
        style("Congratulations, your project is ready!").underlined()
    ));

    // TODO: update dev server to watch rust and frontend files and execute tsync/dsync calls accordingly
    if !is_cargo_watch_installed {
        message("• Install cargo-watch (for development)");
        message(&format!(
            "  $ {}",
            style("cargo install cargo-watch").cyan()
        ));
    }

    if !is_diesel_cli_installed {
        message("• Install diesel (to manage the database)");
        message(&format!(
            "  $ {} \"{}\"",
            style("cargo install diesel_cli --no-default-features --features").cyan(),
            match install_config.backend_database {
                BackendDatabase::Postgres => "postgres",
                BackendDatabase::Sqlite => "sqlite-bundled",
            }
        ));
    }

    message("• Begin development!");
    message("  1. Change to your project directory");
    message(&format!(
        "     $ {}",
        style(format!("cd {project_name:#?}")).cyan()
    ));
    message("  2. Open `.env` and set the DATABASE_URL");
    message(&format!("     $ {}", style("vim .env").cyan()));
    message("  3. Setup your database:");
    message(&format!("     $ {}", style("diesel database reset").cyan()));
    message("  4. Develop! Run the following for continuous compilation:");
    message(&format!("     $ {}", style("cargo fullstack").cyan()));
    message("• Enjoy!");
}
