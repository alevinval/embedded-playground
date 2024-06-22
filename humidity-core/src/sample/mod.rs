pub use de::Deserializer;
pub use ser::{Error, Serializer};

use crate::serde::{self, Serializable};

mod de;
mod ser;

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
    fn serialize(&self, out: &mut [u8]) -> Result<usize, crate::serde::Error> {
        Serializer::new(out).serialize(&self).map_err(|_| serde::Error::ErrBufferSmall)
    }
}

#[cfg(test)]
mod test {
    use de::Deserializer;
    use ser::Serializer;

    use super::*;

    #[test]
    fn sample_result_serde() {
        let input = SampleResult { avg: 990, min: 813, max: 1238 };

        let mut buffer = [0u8; 60];
        let mut ser = Serializer::new(&mut buffer);
        let n = ser.serialize(&input).unwrap();

        let mut sut = Deserializer::new(&buffer[..n]);
        let output = sut.deserialize().unwrap();
        assert_eq!(input, output);
    }
}
