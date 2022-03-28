use std::io;
use std::process::Command;
use std::path::PathBuf;

pub fn main() -> Result<(), io::Error> {
    let dir = env!("CARGO_MANIFEST_DIR");

    println!("Running `yarn tsync` in `$project_dir/frontend/`...");

    Command::new("yarn")
        .arg("tsync")
        .current_dir(PathBuf::from_iter([dir, "frontend"]))
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    Ok(())
}
