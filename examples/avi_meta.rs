//! Outputs some metadata about a file.
//!
//! Usage: `cargo run --example avi_meta AVIFILE`

extern crate avirus;
use avirus::AVI;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} AVIFILE", args[0]);
        std::process::exit(1);
    }

    let avi = AVI::new(&args[1]).expect("Unable to read AVI file. Error");

    for frame in &avi.frames.meta {
        println!("{frame:?}");
    }
}
