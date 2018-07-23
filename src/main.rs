#![allow(dead_code)]
extern crate byteorder;
mod avi;

use avi::AVI;

fn main() {
    let mut avi = AVI::new("heavens.avi").unwrap();
    avi.frames.remove_keyframes();
    let mut io = avi.frames.make_framedata();
    avi.frames.overwrite(&mut io);
    avi.output("sample_2.avi").unwrap();
}
