use std::{process::Command, path::PathBuf};

mod dsync;
mod tsync;

pub fn main() {
    if !create_rust_app::net::is_port_free(21012) {
        println!("========================================================");
        println!(" ViteJS (the frontend compiler/bundler) needs to run on");
        println!(" port 21012 but it seems to be in use.");
        println!("========================================================");
        panic!("Port 21012 is taken but is required for development!")
    }

    let dir = env!("CARGO_MANIFEST_DIR");

    dsync::main();
    tsync::main();

    Command::new("yarn")
        .arg("fullstack")
        .current_dir(PathBuf::from_iter([dir, "frontend"]))
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}
