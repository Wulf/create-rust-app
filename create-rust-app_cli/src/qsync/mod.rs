// TODO: add .gitignore (and other ignore files) parsing functinality
// TODO: add "create-module" functionality (so generated types can be under a specified namespace like Rust.MyType)
extern crate syn;

pub mod hook;
pub mod params;
pub mod processor;

use std::path::PathBuf;
use structopt::StructOpt;

pub use processor::process;

const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = DESCRIPTION, after_help = "This command helps generate type information for other languages. Currently, only typescript is supported.")]
struct Args {
    /// Activate debug mode
    #[structopt(long, help = "Dry-run, prints to stdout", short = "d", long = "debug")]
    debug: bool,

    // #[structopt(
    //     long = "use-ignore-file",
    //     help = "Optionally ignore files with a .gitignore (or similar file); for example: --use-ignore-file=.gitignore"
    // )]
    // use_ignore_file: Option<PathBuf>,
    /// Input file
    #[structopt(
        short = "i",
        long = "input",
        help = "Required; rust file(s) to read type information from",
        required = true
    )]
    input: Vec<PathBuf>,

    /// Output file, stdout if not present
    #[structopt(
        parse(from_os_str),
        short = "o",
        long = "output",
        help = "Required; file to write generated types to"
    )]
    output: PathBuf,
}

fn main() {
    let args: Args = Args::from_args();

    process(args.input, args.output, args.debug);
}
