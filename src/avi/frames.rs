use avi::frame::Frame;
use std::io::Read;
use std::io::Cursor;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Result as IoResult;
use byteorder::{LittleEndian, BigEndian, ByteOrder};

pub struct Frames {
    pub stream: Vec<u8>,
    pos_of_idx1: usize,
    pos_of_movi: usize,
    pub meta: Vec<Frame>,
}

impl Frames {
    pub fn new(file: Vec<u8>) -> IoResult<Frames> {
        let mut rdr = Cursor::new(&file);

        let mut pos_of_movi: usize = 0;
        let pos_of_idx1: usize;

        rdr.seek(SeekFrom::Start(12))?;
        let mut buf = [0u8; 4];
        rdr.read_exact(&mut buf)?;
        while buf == *b"LIST" || buf == *b"JUNK" {
            rdr.read_exact(&mut buf)?;
            let s = LittleEndian::read_u32(&buf);
            rdr.read_exact(&mut buf)?;
            if buf == *b"movi" {
                pos_of_movi = rdr.position() as usize - 4;
            }
            rdr.seek(SeekFrom::Current(s as i64 - 4))?;
            rdr.read_exact(&mut buf)?;
        }
        pos_of_idx1 = rdr.position() as usize - 4;
        rdr.read_exact(&mut buf)?;
        let s = LittleEndian::read_u32(&buf) + rdr.position() as u32;

        let mut meta: Vec<Frame> = Vec::new();
        let mut framebuffer = [0u8; 16];
        while rdr.position() < s.into() {
            rdr.read_exact(&mut framebuffer)?;
            meta.push(Frame::new(&framebuffer));
        }

        Ok(Frames {
            stream: file.to_vec(),
            pos_of_idx1: pos_of_idx1,
            pos_of_movi: pos_of_movi,
            meta: meta,
        })
    }

    pub fn make_framedata(&mut self) -> IoResult<Vec<u8>> {
        let mut framedata: Vec<u8> = Vec::new();
        framedata.reserve(self.stream.len());
        let mut reader = Cursor::new(&self.stream);
        let mut buf = [0u8; 4];
        for frame in &mut self.meta {
            reader.set_position(self.pos_of_movi as u64 + frame.offset as u64 + 8);
            let mut actual_frame = vec![0u8; frame.length as usize];
            reader.read_exact(&mut actual_frame)?;
            frame.offset = (self.pos_of_movi as u32 + frame.offset + frame.length) as u32 + 12;
            frame.length = actual_frame.len() as u32;
            BigEndian::write_u32_into(&[frame.id], &mut buf);
            framedata.extend_from_slice(&mut buf);
            LittleEndian::write_u32_into(&[frame.length], &mut buf);
            framedata.extend_from_slice(&mut buf);
            framedata.extend_from_slice(&mut actual_frame);
            if frame.length % 2 == 1 {
                framedata.push(0u8);
            }
        }
        Ok(framedata)
    }

    pub fn remove_keyframes(&mut self) {
        let mut data: Vec<Frame> = Vec::new();
        let mut lastpframe = self.meta[0];
        for frame in self.meta.iter() {
            if frame.is_iframe() {
                lastpframe = frame.clone();
                break;
            }
        }
        for frame in self.meta.iter() {
            if frame.is_audioframe() {
                data.push(*frame);
            } else if frame.is_pframe() {
                data.push(*frame);
                lastpframe = frame.clone();
            } else if frame.is_iframe() {
                data.push(lastpframe);
            }
        }
        self.meta = data;
    }

    pub fn overwrite(&mut self, framedata: Vec<u8>) {
        let mut new_stream: Vec<u8> = Vec::new();
        new_stream.extend_from_slice(&self.stream[..self.pos_of_movi as usize - 4]);
        let mut buf = [0u8; 4];
        LittleEndian::write_u32_into(&[4u32], &mut buf);
        new_stream.extend_from_slice(&mut buf);
        new_stream.extend_from_slice(&framedata[..]);
        new_stream.extend_from_slice(b"idx1");
        LittleEndian::write_u32_into(&[self.meta.len() as u32], &mut buf);
        new_stream.extend_from_slice(&mut buf);
        let mut framecount = 0u32;
        for frame in self.meta.iter() {
            new_stream.extend_from_slice(&frame.as_bytes());
            if frame.is_videoframe() {
                framecount += 1;
            }
        }
        let eof = new_stream.len() as u32;
        LittleEndian::write_u32_into(&[eof - 8], &mut buf);
        for i in 4..7 {
            new_stream[i] = buf[i-4];
        }
        LittleEndian::write_u32_into(&[framecount], &mut buf);
        for i in 48..51 {
            new_stream[i] = buf[i-48];
        }
        self.stream = new_stream;
    }

}
