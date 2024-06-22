use super::Error;

pub struct Deserializer<'input> {
    input: &'input [u8],
    pos: usize,
}

impl<'input> Deserializer<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self { input, pos: 0 }
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        let n = 2;
        if self.input[self.pos..].len() < n {
            return Err(Error::ErrBufferSmall);
        }
        let ans = u16::from_le_bytes([self.input[self.pos], self.input[self.pos + 1]]);
        self.pos += n;
        Ok(ans)
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let n = 1;
        if self.input[self.pos..].len() < n {
            return Err(Error::ErrBufferSmall);
        }
        let ans = u8::from_le_bytes([self.input[self.pos]]);
        self.pos += n;
        Ok(ans)
    }
}
