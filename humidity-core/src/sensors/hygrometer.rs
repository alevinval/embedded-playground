//! All about hygrometer sensors.

use super::Sensor;
use crate::serde::{self, Deserializable, Serializable};

/// Represents a variety of soil moisture sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Hygrometer {
    /// Models the YL-69 resistive sensor.
    YL69,
    /// Models HW-390 capacitive sensor.
    HW390,
}

impl Sensor for Hygrometer {
    /// The ADC reading for the sensor when exposed to water.
    fn low(&self) -> u16 {
        match self {
            Hygrometer::YL69 => 220,
            Hygrometer::HW390 => 1000,
        }
    }

    /// The ADC reading for the sensor when exposed to air.
    fn high(&self) -> u16 {
        match self {
            Hygrometer::YL69 => 2053,
            Hygrometer::HW390 => 2050,
        }
    }
}

impl Serializable for Hygrometer {
    fn serialize(&self, ser: &mut serde::Serializer) -> Result<usize, serde::Error> {
        ser.write_u8(*self as u8)
    }
}

impl Deserializable for Hygrometer {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        match de.read_u8()? {
            0 => Ok(Self::YL69),
            1 => Ok(Self::HW390),
            _ => Err(serde::Error::Other),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(Hygrometer::YL69)]
    #[test_case(Hygrometer::HW390)]
    fn test_percentage_boundaries(sut: Hygrometer) {
        let actual = sut.percentage(sut.low());
        assert_eq!(0.0, actual);

        let actual = sut.percentage(sut.high());
        assert_eq!(1.0, actual);
    }

    #[test_case(Hygrometer::YL69, 1400, 0.6437534)]
    #[test_case(Hygrometer::HW390, 1400, 0.3809524)]
    fn test_percentage(sut: Hygrometer, input: u16, expected: f32) {
        let actual = sut.percentage(input);
        assert_eq!(expected, actual);
    }
}
