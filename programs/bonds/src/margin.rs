use anchor_lang::prelude::{AccountInfo, AccountLoader, Result as AnchorResult};
use jet_margin::{AdapterResult, MarginAccount};

#[cfg(not(feature = "mock-margin"))]
pub fn return_to_margin(user: &AccountInfo, adapter_result: &AdapterResult) -> AnchorResult<()> {
    let loader = AccountLoader::<MarginAccount>::try_from(user)?;
    let margin_account = loader.load()?;
    jet_margin::write_adapter_result(&margin_account, adapter_result)
}

#[cfg(feature = "mock-margin")]
pub fn return_to_margin(_user: &AccountInfo, _adapter_result: &AdapterResult) -> AnchorResult<()> {
    Ok(())
}
