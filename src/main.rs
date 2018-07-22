#![allow(dead_code)]
extern crate byteorder;
mod avi;

use avi::AVI;

fn main() {
    let mut avi = AVI::new("sample.avi").unwrap();
    let mut io = avi.frames.make_framedata();
    let new_avi = avi.frames.overwrite(&mut io);
}
