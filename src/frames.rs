use byteorder::{BigEndian, ByteOrder, LittleEndian};
use frame::Frame;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::SeekFrom;

/// The `Frames` type.
pub struct Frames {
    /// The byte stream, represented by a `Vec` of `u8`.
    pub stream: Vec<u8>,
    /// A private field to keep track of the position of a particular header
    pos_of_movi: usize,
    /// The frame list, represented by a `Vec` of [`Frame`](../frame/struct.Frame.html).
    pub meta: Vec<Frame>,
}

impl Frames {
    /// Loads a byte stream and does further processing on it to populate
    /// `Frames::meta`.
    /// Normally, this function is called automatically upon calling `AVI::new`.
    ///
    /// # Errors
    /// Two possible errors can be encountered in this function.
    /// * errors raised by `io::Cursor::seek`, see [`io::Seek::seek`](https://doc.rust-lang.org/std/io/trait.Seek.html#tymethod.seek) for more information
    /// * errors raised by `io::Cursor::read_exact`, see [`io::Read::read_exact`](https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact) for more information
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(file: &Vec<u8>) -> IoResult<Self> {
        let mut rdr = Cursor::new(&file);

        let mut pos_of_movi: usize = 0;

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
            rdr.seek(SeekFrom::Current(i64::from(s) - 4))?;
            rdr.read_exact(&mut buf)?;
        }
        rdr.read_exact(&mut buf)?;
        let s = LittleEndian::read_u32(&buf) + rdr.position() as u32;

        let mut meta: Vec<Frame> = Vec::new();
        let mut framebuffer = [0u8; 16];
        while rdr.position() < s.into() {
            rdr.read_exact(&mut framebuffer)?;
            meta.push(Frame::new(&framebuffer));
        }

        Ok(Self {
            stream: file.clone(),
            pos_of_movi,
            meta,
        })
    }

    /// This method builds a byte stream based on `Frames::meta`.
    /// This is normally called automatically on [`AVI::output`](../struct.AVI.html#method.output).
    ///
    /// # Errors
    /// Errors can be encountered during reading bytes, see [`io::Read::read_exact`](https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact) for more information.
    #[allow(clippy::cast_possible_truncation)]
    pub fn make_framedata(&mut self) -> IoResult<Vec<u8>> {
        let mut framedata: Vec<u8> = Vec::new();
        framedata.reserve(self.stream.len());
        let mut reader = Cursor::new(&self.stream);
        let mut buf = [0u8; 4];
        for frame in &mut self.meta {
            reader.set_position(self.pos_of_movi as u64 + u64::from(frame.offset) + 8);
            let mut actual_frame = vec![0u8; frame.length as usize];
            reader.read_exact(&mut actual_frame)?;
            frame.offset = (self.pos_of_movi as u32 + frame.offset + frame.length) + 12;
            frame.length = actual_frame.len() as u32;
            BigEndian::write_u32_into(&[frame.id], &mut buf);
            framedata.extend_from_slice(&buf);
            LittleEndian::write_u32_into(&[frame.length], &mut buf);
            framedata.extend_from_slice(&buf);
            framedata.extend_from_slice(&actual_frame);
            if frame.length % 2 == 1 {
                framedata.push(0u8);
            }
        }
        Ok(framedata)
    }

    /// A helper method to remove all keyframes except the first in `Frames::meta`
    /// It also attempts to sync audio and video by adding an additional pframe
    /// for every iframe it removes. `Frames::meta` will be overwritten by this
    /// method.
    ///
    /// # Examples
    /// ```
    /// use avirus::AVI;
    /// use avirus::frame::Frame;
    ///
    /// let mut avi = AVI::new("path_to.avi").unwrap();
    /// avi.frames.remove_keyframes();
    /// ```
    pub fn remove_keyframes(&mut self) {
        // this function is subject for removal, as it's more of a
        // fun helper function more than anything. people who wish to have more
        // fine-grained control is likely to reimplement this with slight changes anyway.
        let mut data: Vec<Frame> = Vec::new();
        let mut lastpframe: Option<Frame> = None;
        for frame in &self.meta {
            if frame.is_iframe() {
                lastpframe = Some(*frame);
                break;
            }
        }
        for frame in &self.meta {
            if frame.is_audioframe() {
                data.push(*frame);
            } else if frame.is_pframe() {
                data.push(*frame);
                lastpframe = Some(*frame);
            } else if frame.is_iframe() {
                // this clause is here to keep audio synced up to the video
                // instead of dropping iframes, we replace them with the last
                // found pframe.
                if let Some(ref value) = lastpframe {
                    data.push(*value);
                }
            }
        }
        self.meta = data;
    }

    /// A method which overwrites parts of `Frames::stream` with the input
    /// `framedata`. This is normally called automatically in `AVI::output` and
    /// uses the current state of `Frames`.
    #[allow(clippy::cast_possible_truncation)]
    pub fn overwrite(&mut self, framedata: &[u8]) {
        let mut new_stream: Vec<u8> = Vec::new();
        new_stream.extend_from_slice(&self.stream[..self.pos_of_movi - 4]);
        let mut buf = [0u8; 4];
        LittleEndian::write_u32_into(&[4u32], &mut buf);
        new_stream.extend_from_slice(&buf);
        new_stream.extend_from_slice(framedata);
        new_stream.extend_from_slice(b"idx1");
        LittleEndian::write_u32_into(&[self.meta.len() as u32], &mut buf);
        new_stream.extend_from_slice(&buf);
        let mut framecount = 0u32;
        for frame in &self.meta {
            new_stream.extend_from_slice(&frame.as_bytes());
            if frame.is_videoframe() {
                framecount += 1;
            }
        }
        let eof = new_stream.len() as u32;
        LittleEndian::write_u32_into(&[eof - 8], &mut buf);
        new_stream[4..7].copy_from_slice(&buf[..(7 - 4)]);

        LittleEndian::write_u32_into(&[framecount], &mut buf);
        new_stream[48..51].copy_from_slice(&buf[..(51 - 48)]);

        self.stream = new_stream;
    }
}
