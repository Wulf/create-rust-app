use crate::utils::logger::{add_file_msg, modify_file_msg};
use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::path::PathBuf;

pub fn is_rust_project(directory: &PathBuf) -> Result<bool> {
    if !directory.is_dir() {
        return Err(anyhow!(
            "Expected a directory, but got: '{}'!",
            &directory.display()
        ));
    }

    let is_verified_rust_project = std::process::Command::new("cargo")
        .current_dir(directory)
        .arg("verify-project")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("failed to execute `cargo verify-project`")
        .success();

    Ok(is_verified_rust_project)
}

pub fn get_current_working_directory() -> Result<PathBuf> {
    match std::env::current_dir() {
        Ok(path) => Ok(path),
        Err(_) => Err(anyhow!("Could not get current working directory.")),
    }
}

pub fn ensure_file(file: &PathBuf, contents: Option<&str>) -> Result<()> {
    if file.is_dir() {
        return Err(anyhow!(
            "Expected a file, but found a directory at '{:#?}'!",
            &file
        ));
    }

    if contents.is_some() {
        std::fs::write(file, contents.unwrap())?;
    }

    Ok(())
}

pub fn ensure_directory(directory: &PathBuf, prompt_before_create: bool) -> Result<()> {
    if !directory.exists() {
        let proceed = match prompt_before_create {
            false => true,
            true => Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Directory does not exist, create '{:#?}' directory?",
                    &directory
                ))
                .default(false)
                .interact()?,
        };

        if proceed {
            std::fs::create_dir_all(directory)?;
            return Ok(());
        } else {
            return Err(anyhow!(
                "Required directory '{:#?}' was not present!",
                &directory
            ));
        }
    }

    if !directory.is_dir() {
        let proceed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Expected directory but found file at '{:#?}'.",
                &directory
            ))
            .default(false)
            .interact()?;

        if proceed {
            std::fs::remove_file(directory)?;
            std::fs::create_dir_all(directory)?;
            return Ok(());
        } else {
            return Err(anyhow!(
                "Found file '{:#?}' but expecte directory!",
                &directory
            ));
        }
    }

    Ok(())
}

fn read(file_path: &str) -> Result<(PathBuf, String)> {
    let file_path = PathBuf::from(file_path);
    ensure_file(&file_path, None)?;
    let contents: String = std::fs::read_to_string(&file_path)?;
    Ok((file_path, contents))
}

pub fn prepend(file_path: &str, content: &str) -> Result<()> {
    modify_file_msg(&format!("{:#?}", &file_path));

    let (file_path, file_contents) = read(file_path)?;

    let mut new_content = String::new();
    new_content.push_str(content);
    new_content.push('\n');
    new_content.push_str(file_contents.as_str());

    std::fs::write(file_path, new_content)?;

    Ok(())
}

/*
/// Try to create files via the template-* folders for plugins
pub fn create(file_path: &str, content: &str) -> Result<()> {
    if PathBuf::from(file_path).exists() {
        panic!("File already exists!");
    }

    std::fs::write(file_path, content)?;

    Ok(())
}
*/

pub fn append(file_path: &str, content: &str) -> Result<()> {
    modify_file_msg(&format!("{:#?}", &file_path));

    let (file_path, file_contents) = read(file_path)?;

    let mut new_content = String::new();
    new_content.push_str(file_contents.as_str());
    new_content.push('\n');
    new_content.push_str(content);

    std::fs::write(file_path, new_content)?;

    Ok(())
}

pub fn replace(file_path: &str, from: &str, to: &str) -> Result<()> {
    modify_file_msg(&format!("{:#?}", &file_path));

    let (file_path, file_contents) = read(file_path)?;
    let file_contents = file_contents.replace(from, to);

    std::fs::write(file_path, file_contents)?;

    Ok(())
}

pub fn add_rust_file(file_directory: &str, file_name: &str, file_contents: &str) -> Result<()> {
    let file_path = PathBuf::from(format!("{file_directory}/{file_name}.rs"));
    let mod_file_path = PathBuf::from(format!("{file_directory}/mod.rs"));
    let file_directory = PathBuf::from(file_directory);

    ensure_directory(&file_directory, true)?;

    add_file_msg(&format!("{:#?}", &file_path));

    let mut mod_file_contents: String;

    if mod_file_path.exists() {
        ensure_file(&mod_file_path, None)?;
        mod_file_contents = std::fs::read_to_string(&mod_file_path)?;
        mod_file_contents.push('\n');
    } else {
        mod_file_contents = String::new();
    }

    mod_file_contents.push_str("pub mod ");
    mod_file_contents.push_str(file_name);
    mod_file_contents.push_str(";\n");
    std::fs::write(mod_file_path, mod_file_contents)?;

    std::fs::write(file_path, file_contents)?;

    Ok(())
}
