#![allow(dead_code)]
extern crate byteorder;
mod avi;

use avi::AVI;
use avi::frame::Frame;

fn main() {
    let mut avi = AVI::new("sample.avi").unwrap();
    let mut new_meta: Vec<Frame> = Vec::new();
    for frame in &mut avi.frames.meta {
        if frame.is_pframe() || frame.is_audioframe() {
            for _ in 0..3 {
                new_meta.push(*frame);
            }
        }
        else {
            new_meta.push(*frame);
        }
    }
    avi.frames.meta = new_meta;
    avi.frames.remove_keyframes();
    avi.output("sample_2.avi").unwrap();
}
