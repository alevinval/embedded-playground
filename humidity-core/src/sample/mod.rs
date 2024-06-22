pub use de::Deserializer;
pub use ser::{Error, Serializer};

mod de;
mod ser;

#[derive(Debug, Default, PartialEq)]
pub struct SampleResult {
    pub avg: u16,
    pub min: u16,
    pub max: u16,
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
