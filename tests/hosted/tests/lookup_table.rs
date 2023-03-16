/// Tests for lookup table, to check that it behaves fine on simulator and test envs
#[cfg_attr(not(feature = "localnet"), ignore = "only run on localnet")]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn lookup_table() -> anyhow::Result<()> {
    use jet_margin_sdk::lookup_tables::LookupTable;
    use solana_sdk::pubkey::Pubkey;

    use hosted_tests::margin_test_context;

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
