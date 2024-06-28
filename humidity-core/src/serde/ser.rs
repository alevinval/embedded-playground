use super::Error;

pub struct Serializer<'output> {
    out: &'output mut [u8],
    pos: usize,
}

impl<'output> Serializer<'output> {
    pub fn new(out: &'output mut [u8]) -> Self {
        Self { out, pos: 0 }
    }

    pub fn write_u16(&mut self, value: u16) -> Result<usize, Error> {
        self.write(&value.to_le_bytes())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<usize, Error> {
        self.write(&value.to_le_bytes())
    }

    fn write(&mut self, value: &[u8]) -> Result<usize, Error> {
        if self.out[self.pos..].len() < value.len() {
            return Err(Error::ErrBufferSmall);
        }
        self.out[self.pos..(self.pos + value.len())].copy_from_slice(value);
        self.pos += value.len();
        Ok(value.len())
    }
}
