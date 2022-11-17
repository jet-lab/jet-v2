use thiserror::Error;

#[derive(Debug, Error)]
pub enum FixedTermWasmError {
    #[error("Order would match with limit order owned by same user.")]
    SelfMatch,
    #[error("Limit price required when estimating outcome of limit orders.")]
    LimitPriceRequired,
    #[error("Resting orders are not appropriately sorted.")]
    RestingOrdersNotSorted,
}
