use super::internal_reader::InternalReader;
use std::{
    error, fmt,
    io::{Error, Read},
    str::from_utf8,
};

pub struct ByteToChar<R> {
    iter: InternalReader<R>,
}

impl<R: Read> ByteToChar<R> {
    #[inline]
    pub fn new(read: R, buffer_size: usize) -> Result<Self, Error> {
        Ok(ByteToChar {
            iter: InternalReader::new(read, buffer_size)?,
        })
    }

    fn get_next(&mut self) -> Result<Option<u8>, CharsError> {
        match self.iter.next() {
            None => Ok(None),
            Some(item) => match item {
                Ok(item) => Ok(Some(item)),
                Err(err) => Err(CharsError::Other(err)),
            },
        }
    }
}

impl<R: Read + fmt::Debug> fmt::Debug for ByteToChar<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Filter").field("iter", &self.iter).finish()
    }
}

impl<R: Read> Iterator for ByteToChar<R> {
    type Item = Result<char, CharsError>;

    #[inline]
    fn next(&mut self) -> Option<Result<char, CharsError>> {
        let first_byte = match self.get_next() {
            Err(err) => return Some(Err(err)),
            Ok(item) => match item {
                Some(item) => item,
                None => return None,
            },
        };

        let width = utf8_char_width(first_byte);
        if width == 1 {
            return Some(Ok(first_byte as char));
        }
        if width == 0 {
            return Some(Err(CharsError::NotUtf8));
        }
        let mut buf = [first_byte, 0, 0, 0];
        {
            let mut start = 1;
            while start < width {
                let byte = match self.get_next() {
                    Err(err) => return Some(Err(err)),
                    Ok(item) => match item {
                        Some(item) => item,
                        None => return Some(Err(CharsError::NotUtf8)),
                    },
                };
                buf[start] = byte;
                start += 1;
            }
        }
        Some(match from_utf8(&buf[..width]).ok() {
            Some(s) => Ok(s.chars().next().unwrap()),
            None => Err(CharsError::NotUtf8),
        })
    }
}

fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

/// An enumeration of possible errors that can be generated from the `Chars`
/// adapter.
#[derive(Debug)]
pub enum CharsError {
    /// Variant representing that the underlying stream was read successfully
    /// but it did not contain valid utf8 data.
    NotUtf8,

    /// Variant representing that an I/O error occurred.
    Other(Error),
}

impl error::Error for CharsError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            CharsError::NotUtf8 => None,
            CharsError::Other(ref e) => e.source(),
        }
    }
}

impl fmt::Display for CharsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CharsError::NotUtf8 => "byte stream did not contain valid utf8".fmt(f),
            CharsError::Other(ref e) => e.fmt(f),
        }
    }
}
