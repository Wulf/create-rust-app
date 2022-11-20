use std::fs::File;
///
/// Build Script
/// This is run as a pre-build step -- before the rust backend is compiled.
///
use std::{io::Write, process::Command};

/*
 * Note: We're waiting for this feature: println!("cargo:info=...")
 * https://github.com/rust-lang/cargo/issues/7037
 * so that we can include println!(...) statements in build.rs
 */

#[allow(dead_code)]
fn shell(command: &str) {
    // println!("build.rs => {}", command);

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect(format!("Failed to run {cmd}", cmd = command).as_str());

    // println!("build.rs => {:?}", output.stdout);
    let mut file =
        File::create(format!("build-log-{}.txt", command)).expect("Couldn't create file...");
    file.write(b"build log\n\n\n\nSTDOUT:\n")
        .expect("Couldn't write to build log");
    file.write_all(&output.stdout)
        .expect("Couldn't write to build log");
    file.write(b"\n\n\n\nSTDERR:\n")
        .expect("Couldn't write to build log");
    file.write_all(&output.stderr)
        .expect("Couldn't write to build log");
}

fn main() {
    // Only install frontend dependencies when building release
    #[cfg(not(debug_assertions))]
    shell("cd frontend && yarn install --frozen-lockfile");

    // Only build frontend when building a release
    #[cfg(not(debug_assertions))]
    shell("cd frontend && yarn build");
}
