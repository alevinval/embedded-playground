use crate::serde::{self, Deserializable, Serializable};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampleResult {
    pub avg: u16,
    pub min: u16,
    pub max: u16,
}

impl SampleResult {
    const DRY: u16 = 3562;
    const WATER: u16 = 1013;
    const RANGE: u16 = Self::DRY - Self::WATER;

    pub fn dryness(&self) -> f32 {
        (self.avg - Self::WATER) as f32 / Self::RANGE as f32
    }
}

impl Serializable<SampleResult> for SampleResult {
    fn serialize(&self, ser: &mut serde::Serializer) -> Result<usize, serde::Error> {
        let mut n = ser.write_u16(self.avg)?;

        let max_delta = self.max - self.avg;
        let min_delta = self.avg - self.min;

        if max_delta > (u8::MAX as u16) || min_delta > u8::MAX as u16 {
            return Err(serde::Error::Other);
        }

        n += ser.write_u8((self.avg - self.min) as u8)?;
        n += ser.write_u8((self.max - self.avg) as u8)?;
        Ok(n)
    }
}

impl Deserializable<Self> for SampleResult {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        let avg = de.read_u16()?;
        let min_delta = de.read_u8()? as u16;
        let max_delta = de.read_u8()? as u16;
        Ok(Self { avg, min: avg - min_delta, max: avg + max_delta })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample_result_serde() {
        let input = SampleResult { avg: 990, min: 813, max: 1238 };

        let mut buffer = [0u8; 60];
        let n = serde::serialize(&input, &mut buffer).unwrap();

        let output = serde::deserialize::<SampleResult>(&buffer[..n]).unwrap();
        assert_eq!(input, output);
    }
}
