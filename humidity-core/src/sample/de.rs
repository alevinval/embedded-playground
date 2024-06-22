use crate::serde;

use super::SampleResult;

pub struct Deserializer<'input> {
    de: serde::Deserializer<'input>,
}

impl<'input> Deserializer<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Self { de: serde::Deserializer::new(input) }
    }

    pub fn deserialize(&mut self) -> Result<SampleResult, serde::Error> {
        let avg = self.de.read_u16()?;
        let min_delta = self.de.read_u8()? as u16;
        let max_delta = self.de.read_u8()? as u16;
        Ok(SampleResult { avg, min: avg - min_delta, max: avg + max_delta })
    }
}
