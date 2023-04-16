//! Rewrites an AVI file to another file.
//!
//! Usage: `cargo run --example avi_remix INFILE OUTFILE`

extern crate avirus;

use avirus::frame::Frame;
use avirus::AVI;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} INFILE OUTFILE", args[0]);
        std::process::exit(1);
    }

    let mut avi = AVI::new(&args[1]).expect("Unable to read AVI file. Error");
    let mut new_meta: Vec<Frame> = Vec::new();

    for frame in &mut avi.frames.meta {
        if frame.is_pframe() || frame.is_audioframe() {
            for _ in 0..3 {
                new_meta.push(*frame);
            }
        } else {
            new_meta.push(*frame);
        }
    }

    avi.frames.meta = new_meta;
    avi.output(&args[2])
        .expect("Unable to write AVI file. Error");
}
