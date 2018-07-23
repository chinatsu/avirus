pub mod frame;
pub mod frames;

use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error, ErrorKind};
use std::io::Cursor;
use std::io::SeekFrom;
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
    pub frames: Frames,
}

impl AVI {
    pub fn new(filename: &str) -> IoResult<AVI> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;
        is_formatted(&buf)?;
        let frames = Frames::new(buf)?;
        Ok(AVI {
            frames: frames,
        })
    }

    pub fn output(&mut self, filename: &str) -> IoResult<()> {
        let io = self.frames.make_framedata()?;
        self.frames.overwrite(io);
        let mut f = File::create(filename)?;
        f.write(&self.frames.stream)?;
        Ok(())
    }

}

fn is_formatted(file: &Vec<u8>) -> IoResult<()> {
    let mut reader = Cursor::new(&file);
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    if buf != *b"RIFF" {
        return Err(Error::new(ErrorKind::InvalidData, "Malformed AVI"));
    }
    reader.seek(SeekFrom::Current(4))?;
    reader.read_exact(&mut buf)?;
    if buf != *b"AVI " {
        return Err(Error::new(ErrorKind::InvalidData, "Malformed AVI"));
    }
    reader.read_exact(&mut buf)?;
    while buf == *b"LIST" || buf == *b"JUNK" {
        reader.read_exact(&mut buf)?;
        let s = LittleEndian::read_u32(&buf);
        reader.seek(SeekFrom::Current(s.into()))?;
        reader.read_exact(&mut buf)?;
    }
    if buf != *b"idx1" {
        return Err(Error::new(ErrorKind::InvalidData, "Malformed AVI"));
    }
    Ok(())
}
