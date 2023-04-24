use anchor_lang::AccountDeserialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use crate::JsResult;

/// Represents a struct that can be serialized into a [JsValue]
pub trait JsSerializable: serde::ser::Serialize {
    fn get_serializer() -> Serializer {
        Serializer::new().serialize_large_number_types_as_bigints(true)
    }
    fn to_js(&self, ser: &Serializer) -> JsResult {
        Ok(self.serialize(ser)?)
    }
    fn to_js_default_serializer(&self) -> JsResult {
        self.to_js(&Self::get_serializer())
    }
}
impl<T> JsSerializable for T where T: serde::ser::Serialize + ?Sized {}

/// Represents an anchor account capable of being deserialized from bytes and
/// re-serialized into a [JsValue]
pub trait JsAnchorDeserialize: JsSerializable + AccountDeserialize {
    fn deserialize_from_buffer(buf: &[u8]) -> Result<JsValue, JsError> {
        let acc = Self::try_deserialize(&mut buf.to_owned().as_slice())?;
        acc.to_js_default_serializer()
    }
}
impl<T> JsAnchorDeserialize for T where T: JsSerializable + AccountDeserialize {}

#[wasm_bindgen(typescript_custom_section)]
const INSTRUCTION_TYPE: &'static str = r#"
/**
 * Intermediate type for translation from wasm serializtion
 */
export type WasmTransactionInstruction = {
    accounts: Array<WasmAccountMeta>,
    program_id: PublicKey,
    data: Buffer,
}

export type WasmAccountMeta = {
    pubkey: PublicKey,
    is_signer: boolean,
    is_writable: boolean,
}
"#;
