use byteorder::{ByteOrder, BigEndian};

use avi::AVIIF_KEYFRAME;


pub struct Frame {
    pub id: u32,
    pub flag: u32,
    pub offset: u32,
    pub length: u32
}

impl Frame {
    pub fn new(bytes: [[u8; 4]; 4]) -> Frame {
        Frame {
            id: BigEndian::read_u32(&bytes[0]),
            flag: BigEndian::read_u32(&bytes[1]),
            offset: BigEndian::read_u32(&bytes[2]),
            length: BigEndian::read_u32(&bytes[3]),
        }
    }

    pub fn is_videoframe(&self) -> bool {
        let id = self.id_as_u8_array();
        &id[2..4] == b"db" || &id[2..4] == b"dc"
    }

    pub fn is_audioframe(&self) -> bool {
        let id = self.id_as_u8_array();
        &id[2..4] == b"wb"
    }

    pub fn is_iframe(&self) -> bool {
        if self.is_videoframe() {
            return self.flag & AVIIF_KEYFRAME != 0;
        }
        false
    }

    pub fn is_pframe(&self) -> bool {
        if self.is_videoframe() {
            return self.flag & AVIIF_KEYFRAME == 0;
        }
        false
    }

    fn id_as_u8_array(&self) -> [u8; 4] {
        let mut buf = [0u8; 4];
        BigEndian::write_u32(&mut buf, self.id);
        buf
    }
}
