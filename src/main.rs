extern crate inflector;

mod db;
mod fs;
mod logger;
mod mail;
mod model;
mod plugins;
mod project;
mod service;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};

#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
pub struct UnknownOpt {
    #[structopt(name = "name")]
    target: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
pub struct CreateOpt {
    #[structopt(name = "name")]
    target: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
pub struct UpdateOpt {}

/// CREATE RUST APP
///
/// A MODERN WAY TO BOOTSTRAP A RUST+REACT APP IN A SINGLE COMMAND
fn main() -> Result<()> {
    /*
        existing CRA project
            YES =>
            NO =>
    */

    let unknown_opts = UnknownOpt::from_args();

    let mut current_dir: PathBuf = fs::get_current_working_directory()?;

    if unknown_opts.target.is_some() {
        current_dir = PathBuf::from(unknown_opts.target.unwrap());

        if current_dir.exists() {
            logger::error(
                &format!(
                    "Cannot create a project: {:#?} already exists.",
                    &current_dir
                )
                .to_string(),
            );
            return Ok(());
        }
    }

    let is_rust_project_directory = current_dir.exists() && fs::is_rust_project(&current_dir)?;

    if !is_rust_project_directory {
        let create_opts = CreateOpt::from_args();
        let project_name = create_opts.target;

        if project_name.len() == 0 {
            logger::error("Please provide a project name");

            return Ok(());
        }

        logger::message("Please select plugins for your new project:");
        logger::message("Use UP/DOWN arrows to navigate, SPACE to enable/disable a plugin, and ENTER to confirm.");

        let items = vec![
            "Plugin: local auth",
            "Plugin: dockerfile",
            "Plugin: development box + admin portal",
        ];
        let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&items)
            .defaults(&[true, true, true])
            .interact()?;

        project::create(project_name.as_ref())?;

        let mut project_dir = PathBuf::from(".");
        project_dir.push(project_name);

        std::env::set_current_dir(project_dir.clone())
            .expect(&format!("Unable to change into {:#?}", project_dir.clone()));

        let add_plugin_auth = chosen.iter().position(|x| *x == 0).is_some();
        let add_plugin_dockerfile = chosen.iter().position(|x| *x == 1).is_some();
        let add_plugin_admin = chosen.iter().position(|x| *x == 2).is_some();

        if add_plugin_auth {
            plugins::install(
                plugins::auth::Auth {},
                plugins::InstallConfig {
                    project_dir: PathBuf::from("."),
                },
            )?;
        }

        if add_plugin_dockerfile {
            plugins::install(
                plugins::container::Container {},
                plugins::InstallConfig {
                    project_dir: PathBuf::from("."),
                },
            )?;
        }

        if add_plugin_admin {
            plugins::install(
                plugins::admin::Admin {},
                plugins::InstallConfig {
                    project_dir: PathBuf::from("."),
                },
            )?;
        }

        logger::project_created_msg(project_dir);
    } else {
        let items = vec!["Add resource", "Cancel"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => {
                match index {
                    0 => {
                        // Add resource
                        let resource_name: String = Input::new()
                            .with_prompt("Resource name")
                            .default("".into())
                            .interact_text()?;

                        if resource_name.len() == 0 {
                            return Ok(());
                        }

                        project::create_resource(resource_name.as_ref())?;
                        std::process::exit(0);
                    }
                    1 => return Ok(()),
                    _ => {
                        logger::error("Not implemneted");
                        std::process::exit(1);
                    }
                }
            }
            None => {}
        }
    }

    Ok(())
}
