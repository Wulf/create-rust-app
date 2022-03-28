pub mod dev;
pub mod auth;
pub mod container;
pub mod storage;

use crate::utils::logger;
use anyhow::Result;
use std::path::PathBuf;
use crate::{BackendFramework, project};

#[derive(Clone)]
pub struct InstallConfig {
    pub project_dir: PathBuf,
    pub backend_framework: BackendFramework
}

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn install(&self, install_config: InstallConfig) -> Result<()>;

    fn before_install(&self) -> Result<()> {
        logger::command_msg("git status --porcelain");

        let output = std::process::Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .output()?;

        let output = std::str::from_utf8(&output.stdout).unwrap();

        if output.len() > 0 {
            logger::error(
                "Please stash and remove any changes (staged, unstaged, and untracked files)",
            );
            return Err(anyhow::anyhow!(
                "Couldn't add plugin because of a dirty git tree."
            ));
        }

        Ok(())
    }

    fn after_install(&self, install_config: InstallConfig) -> Result<()> {
        // cleanup
        project::remove_non_framework_files(&install_config.project_dir, install_config.backend_framework)?;

        logger::command_msg("git add -A");

        let git_add = std::process::Command::new("git")
            .current_dir(".")
            .arg("add")
            .arg("-A")
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");

        if !git_add.success() {
            logger::error("Failed to execute `git add -A`");
            std::process::exit(1);
        }

        logger::command_msg(&format!("git commit -m 'Added {} plugin'", self.name()));

        let git_commit = std::process::Command::new("git")
            .current_dir(".")
            .arg("commit")
            .arg("-m")
            .arg(format!("Added {} plugin", self.name()))
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute process");

        if !git_commit.success() {
            logger::error("Failed to execute `git commit`");
            std::process::exit(1);
        }

        Ok(())
    }
}

pub fn install(plugin: impl Plugin, install_config: InstallConfig) -> Result<()> {
    logger::plugin_msg(plugin.name());

    plugin.before_install()?;
    plugin.install(install_config.clone())?;
    plugin.after_install(install_config.clone())?;

    Ok(())
}
