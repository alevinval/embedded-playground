pub mod de;
pub mod ser;

#[derive(Debug, PartialEq)]
pub struct SampleResult {
    pub avg: u16,
    pub min: u16,
    pub max: u16,
}

impl Default for SampleResult {
    fn default() -> Self {
        Self { avg: 0, min: 0, max: 0 }
    }
}

#[cfg(test)]
mod test {
    use de::Deserializer;
    use ser::Serializer;

    use super::*;

    #[test]
    fn sample_serde() {
        let input = SampleResult { avg: 990, min: 813, max: 1250 };

        let mut buffer = [0u8; 60];
        let mut ser = Serializer::default();
        ser.serialize(&input, &mut buffer).unwrap();

        let mut sut = Deserializer::default();
        let output = sut.deserialize(&buffer).unwrap();
        assert_eq!(input, output);
    }
}
