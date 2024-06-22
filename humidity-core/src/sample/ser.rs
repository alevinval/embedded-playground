use crate::serde;

use super::SampleResult;

pub struct Serializer<'output> {
    ser: serde::Serializer<'output>,
}

#[derive(Debug)]
pub enum Error {
    Error(serde::Error),
    ErrUnexpectedDeviation,
}

impl<'output> Serializer<'output> {
    pub fn new(out: &'output mut [u8]) -> Self {
        Self { ser: serde::Serializer::new(out) }
    }

    pub fn serialize(&mut self, sample: &SampleResult) -> Result<usize, Error> {
        let mut n = self.ser.write_u16(sample.avg).map_err(Error::Error)?;

        let max_delta = sample.max - sample.avg;
        let min_delta = sample.avg - sample.min;

        if max_delta > (u8::MAX as u16) || min_delta > u8::MAX as u16 {
            return Err(Error::ErrUnexpectedDeviation);
        }

        n += self.ser.write_u8((sample.avg - sample.min) as u8).map_err(Error::Error)?;
        n += self.ser.write_u8((sample.max - sample.avg) as u8).map_err(Error::Error)?;
        Ok(n)
    }
}
