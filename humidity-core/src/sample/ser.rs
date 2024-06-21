use super::SampleResult;

#[derive(Debug, PartialEq)]
pub enum SerializerError {
    ErrBufferSmall,
}

pub struct Serializer {
    pos: usize,
}

impl Serializer {
    pub fn serialize(
        &mut self,
        sample: &SampleResult,
        out: &mut [u8],
    ) -> Result<usize, SerializerError> {
        if out.len() < 6 {
            return Err(SerializerError::ErrBufferSmall);
        }
        self.write_u16(sample.avg, out);
        self.write_u16(sample.min, out);
        self.write_u16(sample.max, out);
        Ok(6)
    }

    fn write_u16(&mut self, value: u16, out: &mut [u8]) {
        out[self.pos..self.pos + 2].copy_from_slice(&value.to_le_bytes());
        self.pos += 2
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self { pos: 0 }
    }
}
