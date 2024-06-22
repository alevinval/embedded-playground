pub use de::Deserializer;
pub use ser::Serializer;

mod de;
mod ser;

#[derive(Debug, PartialEq)]
pub enum Error {
    ErrBufferSmall,
}
