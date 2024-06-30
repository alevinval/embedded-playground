use crate::{
    sensors::{Hygrometer, Sensor},
    serde::{self, Deserializable, Serializable},
};

/// Summarizes the results of a sampling operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Summary {
    /// Number of samples.
    pub n: u8,
    /// Average reading across all samples.
    pub avg: u16,
    /// Minimum reading across all samples.
    pub min: u16,
    /// Maximum reading across all samples.
    pub max: u16,
    /// Hygrometer model.
    pub hygrometer: Hygrometer,
}

impl Summary {
    pub fn dryness(&self) -> f32 {
        self.hygrometer.percentage(self.avg)
    }
}

impl Serializable for Summary {
    fn serialize(&self, ser: &mut serde::Serializer) -> Result<usize, serde::Error> {
        let mut n = ser.write_u8(self.n)?;
        n += ser.write_u16(self.avg)?;
        n += ser.write_u16(self.min)?;
        n += ser.write_u16(self.max)?;
        n += self.hygrometer.serialize(ser)?;
        Ok(n)
    }
}

impl Deserializable for Summary {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        let n = de.read_u8()?;
        let avg = de.read_u16()?;
        let min = de.read_u16()?;
        let max = de.read_u16()?;
        let hygrometer = Hygrometer::deserialize(de)?;
        Ok(Self { n, avg, min, max, hygrometer })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample_result_serde() {
        let input = Summary { n: 1, avg: 990, min: 813, max: 1238, hygrometer: Hygrometer::YL69 };

        let mut buffer = [0u8; 60];
        let n = serde::serialize(&input, &mut buffer).unwrap();

        let output = serde::deserialize::<Summary>(&buffer[..n]).unwrap();
        assert_eq!(input, output);
    }
}
