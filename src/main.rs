#![allow(dead_code)]
extern crate byteorder;
mod avi;

use avi::AVI;
use avi::frame::Frame;

fn main() {
    let mut avi = AVI::new("heaven.avi").unwrap();
    let mut new_meta: Vec<Frame> = Vec::new();
    for frame in &mut avi.frames.meta {
        if frame.is_pframe() {
            for i in 0..15 {
                new_meta.push(*frame);
            }
        }
        else {
            new_meta.push(*frame);
        }
    }
    avi.frames.meta = new_meta;
    //avi.frames.remove_keyframes();
    let mut io = avi.frames.make_framedata();
    avi.frames.overwrite(&mut io);
    avi.output("sample_2.avi").unwrap();
}
