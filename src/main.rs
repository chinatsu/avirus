#![allow(dead_code)]
extern crate byteorder;
mod avi;

use avi::AVI;

fn main() {
    let file = AVI::new("sample.avi").unwrap();
    for frame in file.frames.meta.iter() {
        println!("Frame is videoframe: {}", &frame.is_videoframe());
    }
}
