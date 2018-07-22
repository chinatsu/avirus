#![allow(dead_code)]
extern crate byteorder;
extern crate bytes;
mod avi;

use avi::AVI;

fn main() {
    let mut file = AVI::new("sample.avi").unwrap();
    println!("File is formatted: {:?}", &file.is_formatted());
}
