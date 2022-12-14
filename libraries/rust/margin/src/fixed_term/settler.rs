use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use super::FixedTermIxBuilder;
use crate::{solana::transaction::WithSigner, util::no_dupe_queue::AsyncNoDupeQueue};

pub async fn settle_margin_users(
    rpc: Arc<dyn SolanaRpcClient>,
    ix: FixedTermIxBuilder,
    mut margin_accounts: AsyncNoDupeQueue<Pubkey>,
) {
    let ix = Arc::new(ix);
    loop {
        while let Some(margin_account) = margin_accounts.next().await {
            tokio::spawn({
                println!(" INFO - attempting to settle margin account {margin_account}");
                let fut = settle(rpc.clone(), ix.clone(), margin_account);
                let queue = margin_accounts.clone();
                async move {
                    match fut.await {
                        Ok(_) => println!(" INFO - settled margin account {margin_account}"),
                        Err(e) => {
                            println!(
                                "ERROR - failed to settle margin account {margin_account} - {e}"
                            );
                            queue.push(margin_account).await;
                        }
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn settle(
    rpc: Arc<dyn SolanaRpcClient>,
    ix: Arc<FixedTermIxBuilder>,
    margin_account: Pubkey,
) -> Result<(), anyhow::Error> {
    ix.settle(margin_account, None, None)?
        .with_signers(&[])
        .send_and_confirm(&rpc)
        .await?;

    Ok(())
}
