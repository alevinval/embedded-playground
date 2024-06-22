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
        let n = 2;
        if self.out[self.pos..].len() < n {
            return Err(Error::ErrBufferSmall);
        }

        self.out[self.pos..self.pos + n].copy_from_slice(&value.to_le_bytes());
        self.pos += n;
        Ok(n)
    }

    pub fn write_u8(&mut self, value: u8) -> Result<usize, Error> {
        let n = 1;
        if self.out[self.pos..].len() < n {
            return Err(Error::ErrBufferSmall);
        }

        self.out[self.pos..self.pos + n].copy_from_slice(&value.to_le_bytes());
        self.pos += 1;
        Ok(n)
    }
}
