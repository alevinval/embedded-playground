use crate::serde::{self, Deserializable, Serializable};

pub mod sensor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleResult {
    pub avg: u16,
    pub min: u16,
    pub max: u16,
    pub sensor_kind: sensor::SensorKind,
}

impl SampleResult {
    pub fn dryness(&self) -> f32 {
        self.sensor_kind.percent(self.avg)
    }
}

impl Serializable<SampleResult> for SampleResult {
    fn serialize(&self, ser: &mut serde::Serializer) -> Result<usize, serde::Error> {
        let mut n = ser.write_u16(self.avg)?;
        n += ser.write_u16(self.min)?;
        n += ser.write_u16(self.max)?;
        n += self.sensor_kind.serialize(ser)?;
        Ok(n)
    }
}

impl Deserializable<Self> for SampleResult {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        let avg = de.read_u16()?;
        let min = de.read_u16()?;
        let max = de.read_u16()?;
        let sensor_kind = sensor::SensorKind::deserialize(de)?;
        Ok(Self { avg, min, max, sensor_kind })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample_result_serde() {
        let input = SampleResult {
            avg: 990,
            min: 813,
            max: 1238,
            sensor_kind: sensor::SensorKind::Resistive,
        };

        let mut buffer = [0u8; 60];
        let n = serde::serialize(&input, &mut buffer).unwrap();

        let output = serde::deserialize::<SampleResult>(&buffer[..n]).unwrap();
        assert_eq!(input, output);
    }
}
