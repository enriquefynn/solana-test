// SPDX-FileCopyrightText: 2021 Chorus One AG
// SPDX-License-Identifier: GPL-3.0

//! Holds a test context, which makes it easier to test with a Solido instance set up.

use solana_program::instruction::Instruction;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::transport;

solana_program::declare_id!("GkEkdGe68DuTKg6FhVLLPZ3Wm8EcUPCPjhCeu8WrGDoD");

pub struct Context {
    pub context: ProgramTestContext,
}

pub async fn send_transaction(
    context: &mut ProgramTestContext,
    instructions: &[Instruction],
) -> transport::Result<()> {
    let mut transaction = Transaction::new_with_payer(instructions, Some(&context.payer.pubkey()));
    transaction.sign(&[&context.payer], context.last_blockhash);
    let result = context.banks_client.process_transaction(transaction).await;
    result
}

impl Context {
    pub async fn new_empty() -> Context {
        let mut program_test = ProgramTest::default();

        program_test.add_program("foo", id(), processor!(foo::processor::process));

        let mut result = Self {
            context: program_test.start_with_context().await,
        };

        send_transaction(
            &mut result.context,
            &[Instruction {
                program_id: id(),
                accounts: Vec::new(),
                data: Vec::new(),
            }],
        )
        .await
        .expect("Failed to initialize Solido instance.");

        result
    }
}

/// Return an `AccountInfo` for the given account, with `is_signer` and `is_writable` set to false.
pub fn get_account_info<'a>(address: &'a Pubkey, account: &'a mut Account) -> AccountInfo<'a> {
    let is_signer = false;
    let is_writable = false;
    let is_executable = false;
    AccountInfo::new(
        address,
        is_signer,
        is_writable,
        &mut account.lamports,
        &mut account.data,
        &account.owner,
        is_executable,
        account.rent_epoch,
    )
}
