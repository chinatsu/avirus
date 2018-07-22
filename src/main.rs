extern crate byteorder;
mod avi;

use byteorder::{ByteOrder, BigEndian};
use avi::frame::Frame;

fn main() {
    let frame = Frame::new([*b"00dc", *b"0000", *b"0001", *b"0002"]);
    println!("Videoframe: {:?}", &frame.is_videoframe());
    println!("Audioframe: {:?}", &frame.is_audioframe());
    println!("I-frame: {:?}", &frame.is_iframe());
    println!("P-frame: {:?}", &frame.is_pframe());
}
