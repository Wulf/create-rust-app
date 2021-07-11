use inflector::Inflector;
use std::path::PathBuf;
use crate::logger;
use crate::fs::{ensure_directory, ensure_file};
use anyhow::Result;

fn get_migration_number() -> usize {
  let migrations_dir = PathBuf::from("migrations");

  if !migrations_dir.is_dir() {
    logger::message("Migrations directory does not exist, create it?");
  }
  
  let count = match migrations_dir.read_dir().unwrap() {
    v => v.count() - 1 /* 0-indexed, so subtract 1 */,
    // Err(_) => 3 // guess the migration number
  };
  
  return count;
}

pub fn create_migration(name: &str, up: &str, down: &str) -> Result<()> {
  let mut migrations_dir = PathBuf::from("migrations");
  ensure_directory(&migrations_dir, true)?;
  
  let migration_number = get_migration_number();
  let migration_dir_name = format!("{:0>14}_{}", migration_number, name.to_snake_case());
  
  migrations_dir.push(&migration_dir_name);
  ensure_directory(&migrations_dir, false)?;
  
  let up_file = PathBuf::from(format!("migrations/{}/up.sql", migration_dir_name));
  let down_file = PathBuf::from(format!("migrations/{}/down.sql", migration_dir_name));
  ensure_file(&up_file, Some(up))?;
  ensure_file(&down_file, Some(down))?;

  Ok(())
}