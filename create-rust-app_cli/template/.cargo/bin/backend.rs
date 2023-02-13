use std::{process::Command, path::PathBuf};

pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    Command::new("cargo")
        .args(["watch", "-x", "run", "-w", "backend"])
        .current_dir(PathBuf::from(dir))
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}