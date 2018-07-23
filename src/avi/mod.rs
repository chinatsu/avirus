pub mod frame;
pub mod frames;

use std::fs::File;
use std::io::Result as IoResult;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::Read;
use byteorder::{ByteOrder, LittleEndian};
use self::frames::Frames;

pub const BUFFER_SIZE: u64 = 16777216; // 2 ^ 24
pub const AVIIF_LIST: u32 = 0x00000001;
pub const AVIIF_KEYFRAME: u32 = 0x00000010;
pub const AVIIF_NO_TIME: u32 = 0x00000100;
pub const SAFE_FRAMES_COUNT: u64 = 150000;

pub struct AVI {
    file: Vec<u8>,
    pub frames: Frames,
}

impl AVI {
    pub fn new(filename: &str) -> IoResult<AVI> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;
        if !is_formatted(&buf) {
            panic!("poorly formatted input :(");
        }
        let frames = Frames::new(&mut buf);
        Ok(AVI {
            file: buf,
            frames: frames,
        })
    }

    pub fn output(&mut self, filename: &str) -> IoResult<()> {
        let mut f = File::create(filename)?;
        f.write(&self.frames.stream)?;
        Ok(())
    }

}

fn is_formatted(file: &Vec<u8>) -> bool {
    let mut reader = BufReader::new(&file[..]);
    if read_n(&mut reader, 4) != *b"RIFF" {
        return false;
    }
    read_n(&mut reader, 4);
    if read_n(&mut reader, 4) != *b"AVI " {
        return false;
    }
    let mut list_or_junk = read_n(&mut reader, 4);
    while list_or_junk == *b"LIST" || list_or_junk == *b"JUNK" {
        let s = LittleEndian::read_u32(&read_n(&mut reader, 4)[..]);
        read_n(&mut reader, s.into());
        list_or_junk = read_n(&mut reader, 4);

    }
    if list_or_junk != *b"idx1" {
        return false;
    }
    true
}

fn read_n<R>(reader: &mut R, bytes_to_read: u64) -> Vec<u8>
where
    R: Read,
{
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    // Do appropriate error handling for your situation
    let n = chunk.read_to_end(&mut buf).expect("Didn't read enough");
    assert_eq!(bytes_to_read as usize, n);
    buf
}
