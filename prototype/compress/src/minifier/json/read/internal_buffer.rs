const ARRAY_DEFAULT: u8 = 0;

#[derive(Debug)]
pub struct Buffer {
    buffer: Vec<u8>,
    read_pos: usize,
    buffer_size: usize,
    data_size: usize,
}

impl Buffer {
    pub fn new(size: usize) -> Buffer {
        Buffer {
            buffer: vec![ARRAY_DEFAULT; size],
            read_pos: 0,
            buffer_size: size,
            data_size: 0,
        }
    }

    pub fn as_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut()
    }

    pub fn update_metadata(&mut self, size: usize) {
        self.read_pos = 0;
        self.data_size = size;
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.read_pos >= self.data_size {
            return None;
        }
        let item = self.buffer.get(self.read_pos);
        self.read_pos += 1;
        item.copied()
    }

    pub fn cont(&self) -> bool {
        self.data_size == self.buffer_size
    }
}
