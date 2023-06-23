use hosted_tests::margin_test_context;
use jet_simulation::{Keygen, RandomKeygen};

/// Tests for lookup table, to check that it behaves fine on simulator and test envs
#[cfg_attr(not(feature = "localnet"), ignore = "only run on localnet")]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn lookup_table() -> anyhow::Result<()> {
    use jet_margin_sdk::lookup_tables::LookupTable;
    use solana_sdk::pubkey::Pubkey;

    // Get the mocked runtime
    let ctx = margin_test_context!();

    let table = LookupTable::create_lookup_table(&ctx.rpc(), None)
        .await
        .unwrap();
    const NUM_ADDRESSES: usize = 40;

    let accounts = &[Pubkey::new_unique(); NUM_ADDRESSES];

    LookupTable::extend_lookup_table(&ctx.rpc(), table, None, accounts)
        .await
        .unwrap();

    // Lookup table should not add duplicate accounts
    let result = LookupTable::extend_lookup_table(&ctx.rpc(), table, None, accounts).await;
    assert!(result.is_err());

    // The lookup table should have 40 accounts
    let table = LookupTable::get_lookup_table(&ctx.rpc(), &table)
        .await?
        .unwrap();
    assert_eq!(table.addresses.len(), NUM_ADDRESSES);

    Ok(())
}

/// Test that a user can create a lookup table registry for a margin account
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_lookup_table_registry() -> anyhow::Result<()> {
    use solana_sdk::signer::Signer;

    // Get the mocked runtime
    let ctx = margin_test_context!();

    let wallet = ctx.create_wallet(2).await?;
    ctx.issue_permit(wallet.pubkey()).await?;
    let user = ctx.margin_client().user(&wallet, 0).created().await?;

    user.init_lookup_registry().await?;

    let keygen = RandomKeygen;
    // Sleep for some time to meet recent_slot constraints
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    // Create a lookup table in a registry
    let lookup_table = user.create_lookup_table().await?;

    // Trying to use the lookup table immediately doesn't work
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Add accounts to the lookup table
    // TODO: The library should have control over accounts to prevent
    // a free-for-all
    let addresses = (0..12)
        .map(|_| keygen.generate_key().pubkey())
        .collect::<Vec<_>>();
    user.append_to_lookup_table(lookup_table, &addresses[..])
        .await?;

    Ok(())
}
