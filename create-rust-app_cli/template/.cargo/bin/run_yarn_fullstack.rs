use std::io;
use std::process::Command;

#[cfg(windows)]
pub const YARN_COMMAND: &'static str = "yarn.cmd";

#[cfg(not(windows))]
pub const YARN_COMMAND: &'static str = "yarn";

pub fn main() -> Result<(), io::Error> {
    if !create_rust_app::net::is_port_free(21012) {
        println!("========================================================");
        println!(" ViteJS (the frontend compiler/bundler) needs to run on");
        println!(" port 21012 but it seems to be in use.");
        println!("========================================================");
        panic!("Port 21012 is taken but is required for development!")
    }

    Command::new(YARN_COMMAND)
        .arg("fullstack")
        .current_dir("frontend")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    Ok(())
}
