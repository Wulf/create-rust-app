extern crate inflector;

mod project;
mod logger;
mod model;
mod service;
mod fs;
mod db;
mod plugins;
mod mail;

use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
pub struct Opt {
    #[structopt(long, short)]
    verbose: Option<bool>,
    
    #[structopt(long, short)]
    project: bool,
    
    #[structopt(long, short)]
    add: Option<String>,
    
    #[structopt(name = "...")]
    target: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let debug = opt.verbose.is_some();
    let build_project = opt.project;
    let add_to_project = opt.add.is_some();

    if debug {
        println!("CLI options\n{:#?}\n", opt)
    }

    if build_project {
        project::create(opt)?;
        std::process::exit(0);
    }

    if add_to_project {
        let add_type = opt.add.clone().unwrap();
        match add_type.as_str() {
            "resource" => {
                project::create_resource(opt)?;
                std::process::exit(0);
            },
            "plugin" => {
                match opt.target.as_str() {
                    "auth" => {
                        // TODO: confirm that the current directory actually is a project!
                        let project_dir = &PathBuf::from(".");
                        
                        plugins::install(plugins::auth::Auth {}, plugins::InstallConfig {
                            project_dir: PathBuf::from(&project_dir)
                        })?;
                    },
                    _ => {
                        logger::error(&format!("Plugin '{}' not found", opt.target));
                        std::process::exit(1);
                    }
                }
            },
            _ => {
                logger::error("Invalid type specified for --add option");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}