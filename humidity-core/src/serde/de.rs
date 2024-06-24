use super::Error;

pub struct Deserializer<'input> {
    input: &'input [u8],
    pos: usize,
}

impl<'input> Deserializer<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self { input, pos: 0 }
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.read::<1>().map(u8::from_le_bytes)
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        self.read::<2>().map(u16::from_le_bytes)
    }

    fn read<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let buffer = &self.input[self.pos..];
        if buffer.len() < N {
            return Err(Error::ErrBufferSmall);
        }

        self.pos += N;
        let mut buffer_s: [u8; N] = [0u8; N];
        buffer_s.copy_from_slice(&buffer[..N]);
        Ok(buffer_s)
    }
}
