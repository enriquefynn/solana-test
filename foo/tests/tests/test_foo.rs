use solana_program_test::tokio;
use testlib::foo_ctx::Context;
#[tokio::test]
async fn test_foo() {
    let mut context = Context::new_empty().await;
}
