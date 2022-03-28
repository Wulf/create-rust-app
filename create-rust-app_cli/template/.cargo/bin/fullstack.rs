use std::io;
use std::process::Command;

pub fn main() -> Result<(), io::Error> {
    Command::new("yarn")
        .arg("fullstack")
        .current_dir("frontend")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    Ok(())
}
