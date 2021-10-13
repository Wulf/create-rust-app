use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn check_config(project_dir: &PathBuf, config_item: &str) -> bool {
    Command::new("git")
        .current_dir(project_dir)
        .arg("config")
        .arg(config_item)
        .stdout(Stdio::null())
        .status()
        .expect("failed to execute process")
        .success()
}

pub fn set_config(project_dir: &PathBuf, config_item: &str, config_value: &str) -> bool {
    Command::new("git")
        .current_dir(project_dir)
        .arg("config")
        .arg(config_item)
        .arg(config_value)
        .stdout(Stdio::null())
        .status()
        .expect("failed to execute process")
        .success()
}