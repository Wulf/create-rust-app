use std::io;
use std::process::Command;
use std::path::PathBuf;

#[cfg(windows)]
pub const YARN_COMMAND: &'static str = "yarn.cmd";

#[cfg(not(windows))]
pub const YARN_COMMAND: &'static str = "yarn";

pub fn main() -> Result<(), io::Error> {
    let dir = env!("CARGO_MANIFEST_DIR");

    println!("Running `yarn tsync` in `$project_dir/frontend/`...");

    Command::new(YARN_COMMAND)
        .arg("tsync")
        .current_dir(PathBuf::from_iter([dir, "frontend"]))
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    Ok(())
}
