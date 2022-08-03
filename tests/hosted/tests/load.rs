use hosted_tests::load::load_test;

#[tokio::test(flavor = "multi_thread")]
async fn test() {
    load_test(40, 4).await.unwrap()
}
