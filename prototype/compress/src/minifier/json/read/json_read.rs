use std::{
    fmt,
    io::{Error, ErrorKind, Read},
    vec::IntoIter,
};
use {
    super::byte_to_char::{ByteToChar, CharsError},
    crate::minifier::json::json_minifier::JsonMinifier,
};

pub struct JsonRead<P, R> {
    minifier: JsonMinifier,
    read: Option<R>,
    iter: Option<ByteToChar<R>>,
    predicate: P,
    initialized: bool,
    item_iter: Option<IntoIter<u8>>,
    item1: Option<char>,
}

impl<P, R: Read> JsonRead<P, R> {
    #[inline]
    pub fn new(read: R, predicate: P) -> Self {
        JsonRead {
            minifier: JsonMinifier::default(),
            read: Some(read),
            iter: None,
            predicate,
            initialized: false,
            item_iter: None,
            item1: None,
        }
    }

    fn get_next(&mut self) -> Result<Option<char>, CharsError> {
        match self.iter.as_mut().unwrap().next() {
            None => Ok(None),
            Some(item) => match item {
                Ok(item) => Ok(Some(item)),
                Err(err) => Err(err),
            },
        }
    }

    fn add_char_to_buffer(&mut self, buf: &mut [u8], buf_pos: &mut usize) {
        if let Some(ref mut iter) = self.item_iter {
            while *buf_pos < buf.len() {
                if let Some(byte) = iter.next() {
                    buf[*buf_pos] = byte;
                    *buf_pos += 1;
                } else {
                    break;
                }
            }
        }
    }
}

impl<P, R: Read + fmt::Debug> fmt::Debug for JsonRead<P, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Filter")
            .field("iter", &self.iter)
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl<P, R> Read for JsonRead<P, R>
where
    R: Read,
    P: FnMut(&mut JsonMinifier, &char, Option<&char>) -> bool,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut buf_pos: usize = 0;

        if buf.is_empty() {
            return Ok(0);
        }

        if !self.initialized {
            self.iter = Some(ByteToChar::new(self.read.take().unwrap(), buf.len())?);
            self.item1 = self.get_next()?;
            self.initialized = true;
        }

        while let Some(item) = self.item1.take() {
            self.item1 = self.get_next()?;
            if (self.predicate)(&mut self.minifier, &item, self.item1.as_ref()) {
                self.item_iter = Some(item.to_string().into_bytes().into_iter());
                self.add_char_to_buffer(buf, &mut buf_pos);
            }
            if buf_pos >= buf.len() {
                break;
            }
        }
        Ok(buf_pos)
    }
}

impl From<CharsError> for Error {
    fn from(_: CharsError) -> Self {
        Error::from(ErrorKind::InvalidData)
    }
}
