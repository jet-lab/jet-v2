use crate::{
    context::test_context,
    setup_helper::{create_tokens, create_users}, MapAsync,
};

pub async fn load_test(user_count: usize, mint_count: usize) -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    println!("creating tokens");
    let (mut mints, _, pricer) = create_tokens(&ctx, mint_count).await?;
    println!("creating users");
    let users = create_users(&ctx, user_count).await?;
    println!("creating deposits");
    users
        .iter()
        .zip(mints.iter().cycle())
        .map_async(|(user, mint)| user.deposit(&mint, 100))
        .await?;
    println!("creating loans");
    mints.rotate_right(mint_count / 2);
    users
        .iter()
        .zip(mints.iter().cycle())
        .map_async(|(user, mint)| user.borrow_to_wallet(&mint, 10))
        .await?;
    println!("incrementally lowering prices of half of the assets");
    let assets_to_devalue = mints[0..mints.len() / 2].to_vec();
    let mut price = 1.0;
    for _ in 0..100 {
        price *= 0.99;
        println!("setting price to {price}");
        assets_to_devalue
            .iter()
            .map_async(|mint| pricer.set_price(mint, price))
            .await?;
    }

    Ok(())
}
