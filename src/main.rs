use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-rust-app")]
struct Opt {
    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    project_dir: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}