use crate::serde::{self, Deserializable, Serializable};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum SensorKind {
    Resistive,
    Capacitive,
}

impl SensorKind {
    pub const fn low(&self) -> u16 {
        match self {
            SensorKind::Resistive => 220,
            SensorKind::Capacitive => 700,
        }
    }

    pub const fn high(&self) -> u16 {
        match self {
            SensorKind::Resistive => 2053,
            SensorKind::Capacitive => 2000,
        }
    }

    pub fn percent(&self, value: u16) -> f32 {
        (value - self.low()) as f32 / (self.high() - self.low()) as f32
    }
}

impl Serializable<Self> for SensorKind {
    fn serialize(&self, se: &mut serde::Serializer) -> Result<usize, serde::Error> {
        se.write_u8(*self as u8)
    }
}

impl Deserializable<Self> for SensorKind {
    fn deserialize(de: &mut serde::Deserializer) -> Result<Self, serde::Error> {
        match de.read_u8()? {
            0 => Ok(Self::Resistive),
            1 => Ok(Self::Capacitive),
            _ => Err(serde::Error::Other),
        }
    }
}
