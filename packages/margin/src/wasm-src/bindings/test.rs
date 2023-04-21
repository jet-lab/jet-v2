use jet_fixed_term::margin::state::MarginUser;
use jet_instructions::fixed_term::Market;
use wasm_bindgen_test::*;

use super::serialization::JsAnchorDeserialize;

#[wasm_bindgen_test]
fn can_deserialize() {
    use std::io::Write;

    let market = &mut [0u8; 8 + std::mem::size_of::<Market>()];
    (market as &mut [u8])
        .write_all(&<Market as anchor_lang::Discriminator>::discriminator())
        .unwrap();
    let user = &mut [0u8; 8 + std::mem::size_of::<MarginUser>()];
    (user as &mut [u8])
        .write_all(&<MarginUser as anchor_lang::Discriminator>::discriminator())
        .unwrap();

    Market::deserialize_from_buffer(market)
        .map_err(|_| ())
        .unwrap();
    MarginUser::deserialize_from_buffer(user)
        .map_err(|_| ())
        .unwrap();
}
