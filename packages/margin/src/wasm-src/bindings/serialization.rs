use anchor_lang::AccountDeserialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

/// Represents a struct that can be serialized into a [JsValue]
pub trait JsSerializable: serde::ser::Serialize {
    fn get_serializer() -> Serializer {
        Serializer::new().serialize_large_number_types_as_bigints(true)
    }
    fn to_js(&self, ser: &Serializer) -> Result<JsValue, JsError> {
        Ok(self.serialize(ser)?)
    }
}
impl<T> JsSerializable for T where T: serde::ser::Serialize + ?Sized {}

/// Represents an anchor account capable of being deserialized from bytes and
/// re-serialized into a [JsValue]
pub trait JsAnchorDeserialize: JsSerializable + AccountDeserialize {
    fn deserialize_from_buffer(buf: &[u8]) -> Result<JsValue, JsError> {
        let acc = Self::try_deserialize(&mut buf.to_owned().as_slice())?;
        let ser = Self::get_serializer();
        acc.to_js(&ser)
    }
}
impl<T> JsAnchorDeserialize for T where T: JsSerializable + AccountDeserialize {}
