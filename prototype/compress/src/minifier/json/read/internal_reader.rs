use super::internal_buffer::Buffer;
use std::{
    fmt,
    io::{Error, Read},
};

pub struct InternalReader<R> {
    read: R,
    buffer_size: usize,
    buffer: Buffer,
}

impl<R: Read> InternalReader<R> {
    pub fn new(mut read: R, buffer_size: usize) -> Result<Self, Error> {
        let mut buffer = Buffer::new(buffer_size);
        InternalReader::read_data(&mut read, &mut buffer)?;
        Ok(InternalReader {
            read,
            buffer_size,
            buffer,
        })
    }

    fn read_data(read: &mut R, buffer: &mut Buffer) -> Result<(), Error> {
        let size = read.read(buffer.as_mut())?;
        buffer.update_metadata(size);
        Ok(())
    }
}

impl<R: Read + fmt::Debug> fmt::Debug for InternalReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonReader")
            .field("read", &self.read)
            .field("buffer_size", &self.buffer_size)
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl<R: Read> Iterator for InternalReader<R> {
    type Item = Result<u8, Error>;

    #[inline]
    fn next(&mut self) -> Option<Result<u8, Error>> {
        if self.buffer_size == 0 {
            return None;
        }
        loop {
            if let Some(item) = self.buffer.next() {
                return Some(Ok(item));
            } else if self.buffer.cont() {
                if let Err(err) = InternalReader::read_data(&mut self.read, &mut self.buffer) {
                    return Some(Err(err));
                };
            } else {
                return None;
            }
        }
    }
}
