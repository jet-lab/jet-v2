use crate::{
    context::test_context,
    setup_helper::{create_tokens, create_users},
};

pub async fn load_test(user_count: usize, mint_count: usize) -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    println!("\n\n\ncreating mints\n\n\n");
    let (mut mints, _, pricer) = create_tokens(&ctx, mint_count).await?;
    println!("\n\n\ncreating users\n\n\n");
    let users = create_users(&ctx, user_count).await?;
    println!("\n\n\ncreating deposits\n\n\n");
    for (user, mint) in users.iter().zip(mints.iter().cycle()) {
        user.deposit(&mint, 100).await?;
    }
    println!("\n\n\ncreating loans\n\n\n");
    mints.rotate_right(mint_count / 2);
    for (user, mint) in users.iter().zip(mints.iter().cycle()) {
        user.borrow_to_wallet(&mint, 10).await?;
    }
    println!("\n\n\nincrementally lowering prices of half of the assets\n\n\n");
    let assets_to_devalue = mints[0..mints.len() / 2].to_vec();
    let mut price = 1.0;
    for _ in 0..100 {
        price *= 0.99;
        println!("\n\n\nsetting price to {price}\n\n\n");
        for mint in assets_to_devalue.iter() {
            pricer.set_price(mint, price).await?;
        }
    }

    Ok(())
}
