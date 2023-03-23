/// refresh direct deposit positions in margin
pub mod deposit;
/// refresh fixed term positions
pub mod fixed_term;
/// refresh pool positions
pub mod pool;
/// generically represent the idea of refreshing margin account positions
pub mod position_refresher;

use std::sync::Arc;

use jet_simulation::SolanaRpcClient;

use self::{
    deposit::DepositRefresher, fixed_term::FixedTermRefresher, pool::PoolRefresher,
    position_refresher::SmartRefresher,
};

/// PositionRefresher that refreshes all known positions.
pub fn canonical_position_refresher(rpc: Arc<dyn SolanaRpcClient>) -> SmartRefresher<()> {
    SmartRefresher {
        refreshers: vec![
            Arc::new(DepositRefresher { rpc: rpc.clone() }),
            Arc::new(PoolRefresher { rpc: rpc.clone() }),
            Arc::new(FixedTermRefresher { rpc: rpc.clone() }),
        ],
        rpc,
        margin_account: (),
    }
}
