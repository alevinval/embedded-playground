//! Serialization and deserialization

pub use de::Deserializer;
pub use ser::Serializer;

mod de;
mod ser;

#[derive(Debug, PartialEq)]
pub enum Error {
    ErrBufferSmall,
    Other,
}

pub trait Serializable<T> {
    fn serialize(&self, se: &mut Serializer) -> Result<usize, Error>;
}

pub trait Deserializable<T> {
    fn deserialize(de: &mut Deserializer) -> Result<T, Error>;
}

pub fn serialize<T>(value: &T, out: &mut [u8]) -> Result<usize, Error>
where
    T: Serializable<T>,
{
    let mut se = Serializer::new(out);
    value.serialize(&mut se)
}

pub fn deserialize<T>(out: &[u8]) -> Result<T, Error>
where
    T: Deserializable<T>,
{
    let mut de = Deserializer::new(out);
    T::deserialize(&mut de)
}
