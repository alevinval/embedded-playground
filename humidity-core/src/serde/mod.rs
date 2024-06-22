pub use de::Deserializer;
pub use ser::Serializer;

mod de;
mod ser;

#[derive(Debug, PartialEq)]
pub enum Error {
    ErrBufferSmall,
}

pub trait Serializable<T> {
    fn serialize(&self, out: &mut [u8]) -> Result<usize, Error>;
}
