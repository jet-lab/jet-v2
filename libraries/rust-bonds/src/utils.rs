pub use jet_proto_math::fixed_point::Fp32;

pub struct OrderAmount {
    pub base: u64,
    pub quote: u64,
    pub price: u64,
}

impl OrderAmount {
    pub fn new(amount: u64, interest: u64) -> Option<Self> {
        let quote = amount;
        let base = amount + ((amount * interest) / 10_000);

        Self::price(base, quote).map(|price| Self { base, quote, price })
    }

    pub fn price(base: u64, quote: u64) -> Option<u64> {
        let price = Fp32::from(quote) / base;

        price.downcast_u64()
    }
}
