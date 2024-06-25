use crate::serde::{self, Deserializable, Serializable};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampleResult {
    pub avg: u16,
    pub min: u16,
    pub max: u16,
}

impl SampleResult {
    const AIR: u16 = 2000;
    const WATER: u16 = 700;
    const RANGE: u16 = Self::AIR - Self::WATER;

    pub fn dryness(&self) -> f32 {
        (self.avg - Self::WATER) as f32 / Self::RANGE as f32
    }
}

impl Serializable<SampleResult> for SampleResult {
    fn serialize(&self, ser: &mut serde::Serializer) -> Result<usize, serde::Error> {
        let mut n = ser.write_u16(self.avg)?;
        n += ser.write_u16(self.min)?;
        n += ser.write_u16(self.max)?;
        Ok(n)
    }
}

impl Deserializable<Self> for SampleResult {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        let avg = de.read_u16()?;
        let min = de.read_u16()?;
        let max = de.read_u16()?;
        Ok(Self { avg, min, max })
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
