use jet_solana_client::NetworkUserInterface;

use crate::config::EnvironmentConfig;

use super::{Builder, BuilderError};

pub async fn create_swap_pools<'a, I: NetworkUserInterface>(
    _builder: &mut Builder<I>,
    _config: &EnvironmentConfig,
) -> Result<(), BuilderError> {
    // TODO: implement

    Ok(())
}
