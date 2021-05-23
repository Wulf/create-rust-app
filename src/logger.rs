use console::style;

pub fn message(msg: &str) {
  println!("[{}] {}", style("create-rust-app").blue(), msg)
}

pub fn command_msg(command: &str) {
  message(&format!("Running `{}`", style(command).yellow()));
}

pub fn file_msg(file: &str) {
  message(&format!("Adding {}", style(file).yellow()));
}

pub fn error(msg: &str) {
  message(&format!("{} {}", style("ERROR: ").red(), msg))
}

pub fn exit(msg: &str, err: std::io::Error) -> ! {
  eprintln!("{}: {:?}", msg, err);
  std::process::exit(1);
}

pub fn dependency_msg(name: &str) {
  message(&format!("Adding dependency {}", style(name).yellow()));
}