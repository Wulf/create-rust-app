use std::path::{PathBuf};
use structopt::StructOpt;
use console::{Style, style};
use dialoguer::{
    Confirm,
    theme::ColorfulTheme
};


#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
struct Opt {
    #[structopt(long, short)]
    verbose: Option<bool>,
    
    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    project_dir: PathBuf,
}

fn message(msg: &str) {
    println!("[{}] {}", style("create-rust-app").blue(), msg)
}

fn error(msg: &str) {
    message(&format!("{} {}", style("ERROR: ").red(), msg))
}

fn exit(err: std::io::Error) -> ! {
    eprintln!("Canonicalization Error: {:?}", err);
    std::process::exit(1);
}

fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    let debug = opt.verbose.is_some();

    if debug {
        println!("CLI options\n{:#?}\n", opt)
    }

    let project_dir = match std::fs::canonicalize(opt.project_dir) {
        Ok(p) => p,
        Err(err) => exit(err)
    };
    
    if project_dir.exists() {
        error("Directory already exists");

        let proceed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Delete directory contents?")
            .default(false)
            .interact()?;

        if proceed {
            match std::fs::remove_dir_all(&project_dir) {
                Ok(_) => {},
                Err(err) => exit(err)
            }
        } else {
            std::process::exit(0);
        }
    }
    
    let project_name = project_dir.components().last().unwrap().as_os_str().to_str().unwrap();
        
    message(&format!("Creating project {}", style(project_name).yellow()));

    message("Creating directories...");
        
    match std::fs::create_dir_all(&project_dir) {
        Ok(_) => {},
        Err(err) => exit(err)
    }

    message(&format!("Running `{}`", style("cargo init").green()));
    
    let cargo_init = std::process::Command::new("cargo")
        .current_dir(project_dir)
        .arg("init")
        .status()
        .expect("failed to execute process");

    if !cargo_init.success() {
        error("Failed to execute `cargo init`");
        std::process::exit(1);
    }
    
    Ok(())
}