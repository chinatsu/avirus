use avi::frame::Frame;
use avi::read_n;
use std::io::Read;
use std::io::BufReader;
use byteorder::{LittleEndian, BigEndian, ByteOrder};

pub struct Frames {
    stream: Vec<u8>,
    pos_of_idx1: usize,
    pos_of_movi: usize,
    pub meta: Vec<Frame>,
}

impl Frames {
    pub fn new(file: &mut Vec<u8>) -> Frames {
        let mut f = &file[..];
        let mut reader = BufReader::new(&mut f);
        let mut absolute_position = 0;
        let mut pos_of_movi = 0;
        let mut pos_of_idx1 = 0;

        read_n(&mut reader, 12);
        let mut list_or_junk = read_n(&mut reader, 4);
        absolute_position += 16;
        while list_or_junk == *b"LIST" || list_or_junk == *b"JUNK" {
            let s = LittleEndian::read_u32(&mut read_n(&mut reader, 4));
            absolute_position += 4;
            if read_n(&mut reader, 4) == *b"movi" {
                pos_of_movi = absolute_position as usize;
            }
            absolute_position += 4;
            read_n(&mut reader, s as u64 - 4);
            absolute_position += s - 4;
            list_or_junk = read_n(&mut reader, 4);
            absolute_position += 4;
        }
        pos_of_idx1 = absolute_position as usize;
        absolute_position += 4; // this one comes before the actual move, for convenience
        let s = LittleEndian::read_u32(&read_n(&mut reader, 4)) + absolute_position;

        let mut meta: Vec<Frame> = Vec::new();
        while absolute_position < s {
            meta.push(Frame::new(&read_n(&mut reader, 16)));
            absolute_position += 16;
        }

        Frames {
            stream: file.to_vec(),
            pos_of_idx1: pos_of_idx1,
            pos_of_movi: pos_of_movi,
            meta: meta,
        }
    }

    pub fn make_framedata(&mut self) -> Vec<u8> {
        let mut framedata: Vec<u8> = Vec::new();
        for frame in &mut self.meta {
            let mut stream = &mut &self.stream[..];
            let mut reader = BufReader::new(&mut stream);
            read_n(&mut reader, self.pos_of_movi as u64 + frame.offset as u64 + 8);
            let mut actual_frame = read_n(&mut reader, frame.length as u64);
            frame.offset = (self.pos_of_movi as u32 + frame.offset + frame.length) as u32 + 12;
            frame.length = actual_frame.len() as u32;
            let mut buf = [0u8; 4];
            BigEndian::write_u32_into(&[frame.id], &mut buf);
            framedata.extend_from_slice(&mut buf);
            LittleEndian::write_u32_into(&[frame.length], &mut buf);
            framedata.extend_from_slice(&mut buf);
            framedata.extend_from_slice(&mut actual_frame[..]);
            if frame.length % 2 == 1 {
                framedata.push(0u8);
            }
        }
        framedata
    }

    pub fn overwrite(&mut self, framedata: &mut Vec<u8>) -> Vec<u8> {
        let mut stream = &mut &self.stream[..];
        let mut reader = BufReader::new(&mut stream);
        let mut new_stream: Vec<u8> = Vec::new();
        new_stream.extend_from_slice(&read_n(&mut reader, self.pos_of_movi as u64 - 4)[..]);
        let mut buf = [0u8; 4];
        LittleEndian::write_u32_into(&[4u32], &mut buf);
        new_stream.extend_from_slice(&mut buf);
        new_stream.extend_from_slice(&mut framedata[..]);
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
        let mut u: Vec<u8> = new_stream.splice(4..7, buf.iter().cloned()).collect();
        LittleEndian::write_u32_into(&[framecount], &mut buf);
        let v = u.splice(48..51, buf.iter().cloned()).collect();
        v
    }
}
