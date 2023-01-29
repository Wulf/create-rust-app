use crate::utils::fs::{ensure_directory, ensure_file};
use crate::utils::logger;
use anyhow::Result;
use inflector::Inflector;
use std::path::PathBuf;

fn get_migration_number() -> usize {
    let migrations_dir = PathBuf::from("migrations");

    if !migrations_dir.is_dir() {
        logger::message("Migrations directory does not exist, create it?");
    }

    let v = migrations_dir.read_dir().unwrap();
    v.count()
}

pub fn create(name: &str, up: &str, down: &str) -> Result<()> {
    let mut migrations_dir = PathBuf::from("migrations");
    ensure_directory(&migrations_dir, true)?;

    let migration_number = get_migration_number();
    let migration_dir_name = format!("{:0>14}_{}", migration_number, name.to_snake_case());

    migrations_dir.push(&migration_dir_name);
    ensure_directory(&migrations_dir, false)?;

    let up_file = PathBuf::from(format!("migrations/{migration_dir_name}/up.sql"));
    let down_file = PathBuf::from(format!("migrations/{migration_dir_name}/down.sql"));
    ensure_file(&up_file, Some(up))?;
    ensure_file(&down_file, Some(down))?;

    Ok(())
}
