use avi::frame::Frame;
use avi::read_n;
use std::io::Read;
use std::io::BufReader;
use byteorder::{LittleEndian, ByteOrder};

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
}
