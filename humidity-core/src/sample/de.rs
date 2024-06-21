use super::SampleResult;

pub struct Deserializer {
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum DeserializerError {
    ErrBufferSmall,
}

impl Deserializer {
    pub fn deserialize(&mut self, input: &[u8]) -> Result<SampleResult, DeserializerError> {
        if input.len() < 6 {
            return Err(DeserializerError::ErrBufferSmall);
        }

        let mut sample = SampleResult::default();
        sample.avg = self.read_humidity(input);
        sample.min = self.read_humidity(input);
        sample.max = self.read_humidity(input);
        Ok(sample)
    }

    fn read_humidity(&mut self, input: &[u8]) -> u16 {
        let bfr = &input[self.pos..self.pos + 2];
        self.pos += 2;
        u16::from_le_bytes([bfr[0], bfr[1]])
    }
}

impl Default for Deserializer {
    fn default() -> Self {
        Self { pos: 0 }
    }
}
