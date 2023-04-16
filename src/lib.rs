extern crate byteorder;

pub mod frame;
pub mod frames;

use self::frames::Frames;
use byteorder::{ByteOrder, LittleEndian};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::SeekFrom;
use std::io::{Error, ErrorKind};

/// The `AVI` type.
pub struct AVI {
    /// A Frames object. See [Frames](frames/struct.Frames.html) for more.
    pub frames: Frames,
}

impl AVI {
    /// Loads a new `IoResult<AVI>` from an AVI file.
    ///
    /// # Examples
    /// ```
    /// use avirus::AVI;
    ///
    /// let mut avi = AVI::new("path_to.avi").unwrap();
    /// ```
    ///
    /// # Errors
    /// Several possible IO-related errors may be encountered in this function.
    /// * if `filename` does not already exist, see [`OpenOptions::open`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.open) for more details
    /// * if a read error occurs during the reading of `filename`, see [`io::Read::read`](https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read) for more details
    /// * if expected headers in the byte stream are not found, [`io::ErrorKind::InvalidData`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.InvalidData) will be encountered
    pub fn new(filename: &str) -> IoResult<Self> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf)?;
        is_formatted(&buf)?;
        let frames = Frames::new(&buf)?;
        Ok(Self { frames })
    }

    /// Constructs a binary AVI file from an AVI type.
    ///
    /// # Examples
    /// ```
    /// use avirus::AVI;
    ///
    /// let ut avi = AVI::new("path_to.avi").unwrap();
    /// avi.output("path_to_new.avi").unwrap();
    /// ```
    ///
    /// # Errors
    /// Several possible IO-related errors may be encountered in this function.
    /// * if a reading error is encountered during the creation of framedata, see [`frames::Frames::make_framedata`](frames/struct.Frames.html#method.make_framedata) for more details
    /// * if an error is encountered during creation of the file, see [`io::File::create`](https://doc.rust-lang.org/std/fs/struct.File.html#method.create) for more details
    /// * if an writing error is encountered during output, see [`io::Write::write`](https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.write) for more details
    pub fn output(&mut self, filename: &str) -> IoResult<()> {
        let io = self.frames.make_framedata()?;
        self.frames.overwrite(&io);
        let mut f = File::create(filename)?;
        f.write_all(&self.frames.stream)?;
        Ok(())
    }
}

/// Validates AVI formatting of an input binary file.
/// This is a private function used internally during
/// `AVI::new()`.
///
/// # Errors
/// `io::ErrorKind::InvalidData` upon encountering a missing header at an expected position.
fn is_formatted(file: &Vec<u8>) -> IoResult<()> {
    let mut reader = Cursor::new(&file);
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    if buf != *b"RIFF" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Malformed AVI, missing RIFF at expected position 0x{:x}",
                reader.position()
            ),
        ));
    }
    reader.seek(SeekFrom::Current(4))?;
    reader.read_exact(&mut buf)?;
    if buf != *b"AVI " {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Malformed AVI, missing AVI at expected position 0x{:x}",
                reader.position()
            ),
        ));
    }
    reader.read_exact(&mut buf)?;
    while buf == *b"LIST" || buf == *b"JUNK" {
        reader.read_exact(&mut buf)?;
        let s = LittleEndian::read_u32(&buf);
        reader.seek(SeekFrom::Current(s.into()))?;
        reader.read_exact(&mut buf)?;
    }
    if buf != *b"idx1" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Malformed AVI, missing idx1 at expected position 0x{:x}",
                reader.position()
            ),
        ));
    }
    Ok(())
}
