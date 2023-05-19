use anchor_lang::prelude::*;
use jet_program_common::Number128;

pub fn read_price(pyth_price: &AccountInfo) -> Number128 {
    let price_result = pyth_sdk_solana::load_price_feed_from_account_info(pyth_price).unwrap();
    let price_value = price_result.get_price_unchecked();

    Number128::from_decimal(price_value.price, price_value.expo)
}
