// SPDX-FileCopyrightText: 2021 Chorus One AG
// SPDX-License-Identifier: GPL-3.0

use solana_program_test::tokio;
use testlib::foo_ctx::Context;
#[tokio::test]
async fn test_successful_anchor_deposit() {
    let mut context = Context::new_empty().await;
}
