// SPDX-FileCopyrightText: 2021 Chorus One AG
// SPDX-License-Identifier: GPL-3.0

//! Holds a test context, which makes it easier to test with a Solido instance set up.

use num_traits::cast::FromPrimitive;
use rand::prelude::StdRng;
use rand::SeedableRng;
use solana_program::instruction::Instruction;
use solana_program::instruction::InstructionError;
use solana_program::system_instruction;
use solana_program_test::{processor, ProgramTest, ProgramTestBanksClientExt, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transport;
use solana_sdk::transport::TransportError;

use crate::proj1;

pub struct DeterministicKeypairGen {
    rng: StdRng,
}

impl DeterministicKeypairGen {
    fn new() -> Self {
        let rng = StdRng::seed_from_u64(0);
        DeterministicKeypairGen { rng }
    }
    pub fn new_keypair(&mut self) -> Keypair {
        Keypair::generate(&mut self.rng)
    }
}

#[test]
fn test_deterministic_key() {
    let mut deterministic_keypair = DeterministicKeypairGen::new();
    let kp1 = deterministic_keypair.new_keypair();
    let expected_result: &[u8] = &[
        178, 247, 245, 129, 214, 222, 60, 6, 168, 34, 253, 110, 126, 130, 101, 251, 192, 15, 132,
        1, 105, 106, 91, 220, 52, 245, 166, 210, 255, 63, 146, 47, 237, 208, 246, 222, 52, 42, 30,
        106, 114, 54, 214, 36, 79, 35, 216, 62, 237, 252, 236, 208, 89, 163, 134, 200, 80, 85, 112,
        20, 152, 231, 112, 51,
    ];
    assert_eq!(kp1.to_bytes(), expected_result);
}

// Program id for the Solido program. Only used for tests.

pub struct Context {
    // pub deterministic_keypair: DeterministicKeypairGen,
    /// Inner test context that contains the banks client and recent block hash.
    pub context: ProgramTestContext,
    // A nonce to make similar transactions distinct, incremented after every
    // `send_transaction`.
    // pub nonce: u64,

    // // Key pairs for the accounts in the Solido instance.
    // pub solido: Keypair,
    // pub manager: Keypair,
    // pub st_sol_mint: Pubkey,
    // pub maintainer: Option<Keypair>,
    // pub validator: Option<ValidatorAccounts>,

    // pub treasury_st_sol_account: Pubkey,
    // pub developer_st_sol_account: Pubkey,
    // pub reward_distribution: RewardDistribution,

    // pub reserve_address: Pubkey,
    // pub stake_authority: Pubkey,
    // pub mint_authority: Pubkey,
    // pub withdraw_authority: Pubkey,
}

/// Sign and send a transaction with a fresh block hash.
///
/// The payer always signs, but additional signers can be passed as well.
///
/// Takes a nonce to ensure that sending the same instruction twice will result
/// in distinct transactions. This function increments the nonce after using it.
pub async fn send_transaction(
    context: &mut ProgramTestContext,
    // nonce: &mut u64,
    instructions: &[Instruction],
    additional_signers: Vec<&Keypair>,
) -> transport::Result<()> {
    let mut instructions_mut = instructions.to_vec();

    // If we try to send exactly the same transaction twice, the second one will
    // not be considered distinct by the runtime, and it will not execute, but
    // instead immediately complete successfully. This is undesirable in tests,
    // sometimes we do want to repeat a transaction, e.g. update the exchange
    // rate twice in the same epoch, and confirm that the second one is rejected.
    // Normally the way to do this in Solana is to wait for a new recent block
    // hash. If the block hash is different, the transactions will be distinct.
    // Unfortunately, `get_new_blockhash` interacts badly with `warp_to_slot`.
    // See also https://github.com/solana-labs/solana/issues/18201. To work
    // around this, instead of changing the block hash, add a memo instruction
    // with a nonce to every transaction, to make the transactions distinct.
    // let memo = spl_memo::build_memo(&format!("nonce={}", *nonce).as_bytes(), &[]);
    // instructions_mut.push(memo);
    // *nonce += 1;

    // However, if we execute many transactions and don't request a new block
    // hash, the block hash will eventually be too old. `solana_program_test`
    // doesn’t tell you that this is the problem, instead `process_transaction`
    // will fail with a timeout `IoError`. So do refresh the block hash every
    // 300 transactions.
    // if *nonce % 300 == 299 {
    //     context.last_blockhash = context
    //         .banks_client
    //         .get_new_blockhash(&context.last_blockhash)
    //         .await
    //         .expect("Failed to get a new blockhash.")
    //         .0;
    // }

    // Change this to true to enable more verbose test output.
    // if false {
    //     for (i, instruction) in instructions_mut.iter().enumerate() {
    //         println!(
    //             "Instruction #{} calls program {}.",
    //             i, instruction.program_id
    //         );
    //         for (j, account) in instruction.accounts.iter().enumerate() {
    //             println!(
    //                 "  Account {:2}: [{}{}] {}",
    //                 j,
    //                 if account.is_writable { 'W' } else { '-' },
    //                 if account.is_signer { 'S' } else { '-' },
    //                 account.pubkey,
    //             );
    //         }
    //     }
    // }

    let mut transaction =
        Transaction::new_with_payer(&instructions_mut, Some(&context.payer.pubkey()));

    // Sign with the payer, and additional signers if any.
    let mut signers = additional_signers;
    signers.push(&context.payer);
    transaction.sign(&signers, context.last_blockhash);

    let result = context.banks_client.process_transaction(transaction).await;

    result
}

impl Context {
    /// Set up a new test context with an initialized Solido instance.
    ///
    /// The instance contains no maintainers yet.
    pub async fn new_empty() -> Context {
        // let mut deterministic_keypair = DeterministicKeypairGen::new();
        // let manager = deterministic_keypair.new_keypair();
        // let solido = deterministic_keypair.new_keypair();

        // let reward_distribution = RewardDistribution {
        //     validation_fee: 5,
        //     treasury_fee: 3,
        //     developer_fee: 2,
        //     st_sol_appreciation: 90,
        // };

        // let (reserve_address, _) = Pubkey::find_program_address(
        //     &[&solido.pubkey().to_bytes()[..], RESERVE_ACCOUNT],
        //     &id(),
        // );

        // let (stake_authority, _) = Pubkey::find_program_address(
        //     &[&solido.pubkey().to_bytes()[..], STAKE_AUTHORITY],
        //     &id(),
        // );
        // let (mint_authority, _) =
        //     Pubkey::find_program_address(&[&solido.pubkey().to_bytes()[..], MINT_AUTHORITY], &id());

        // let (withdraw_authority, _) = Pubkey::find_program_address(
        //     &[&solido.pubkey().to_bytes()[..], REWARDS_WITHDRAW_AUTHORITY],
        //     &id(),
        // );

        let mut program_test = ProgramTest::default();
        // Note: the program name *must* match the name of the .so file that contains
        // the program. If it does not, then it will still partially work, but we get
        // weird errors about resizing accounts.

        program_test.add_program(
            "anchor_integration",
            crate::anchor_integration::id(),
            processor!(anchor_integration::processor::process),
        );
        // program_test.add_program(
        //     "project1",
        //     crate::proj1::id(),
        //     processor!(project1::processor::process),
        // );
        program_test.add_program(
            "lido",
            crate::solido::id(),
            processor!(lido::processor::process),
        );

        let mut result = Self {
            context: program_test.start_with_context().await,
            // nonce: 0,
            // manager,
            // solido,
            // st_sol_mint: Pubkey::default(),
            // maintainer: None,
            // validator: None,
            // treasury_st_sol_account: Pubkey::default(),
            // developer_st_sol_account: Pubkey::default(),
            // reward_distribution,
            // reserve_address,
            // stake_authority,
            // mint_authority,
            // withdraw_authority,
            // deterministic_keypair: deterministic_keypair,
        };

        // let max_validators = 10_000;
        // let max_maintainers = 1000;
        // let solido_size = Lido::calculate_size(max_validators, max_maintainers);
        // let rent = result.context.banks_client.get_rent().await.unwrap();
        // let rent_solido = rent.minimum_balance(solido_size);

        // let payer = result.context.payer.pubkey();
        send_transaction(
            &mut result.context,
            // &mut result.nonce,
            &[
                // system_instruction::create_account(
                //     &payer,
                //     &result.solido.pubkey(),
                //     rent_solido,
                //     solido_size as u64,
                //     &id(),
                // ),
                // instruction::initialize(
                //     &id(),
                //     result.reward_distribution.clone(),
                //     max_validators,
                //     max_maintainers,
                //     &instruction::InitializeAccountsMeta {
                //         lido: result.solido.pubkey(),
                //         manager: result.manager.pubkey(),
                //         st_sol_mint: result.st_sol_mint,
                //         treasury_account: result.treasury_st_sol_account,
                //         developer_account: result.developer_st_sol_account,
                //         reserve_account: result.reserve_address,
                //     },
                // ),
                Instruction {
                    program_id: crate::anchor_integration::id(),
                    accounts: Vec::new(),
                    data: Vec::new(),
                },
                Instruction {
                    program_id: crate::solido::id(),
                    accounts: Vec::new(),
                    data: Vec::new(),
                },
                // Instruction {
                //     program_id: crate::proj1::id(),
                //     accounts: Vec::new(),
                //     data: Vec::new(),
                // },
            ],
            // vec![&result.solido],
            vec![],
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

#[macro_export]
macro_rules! assert_solido_error {
    ($result:expr, $error:expr $(, /* Accept an optional trailing comma. */)?) => {
        // Open a scope so the imports don't clash.
        {
            use solana_program::instruction::InstructionError;
            use solana_sdk::transaction::TransactionError;
            use solana_sdk::transport::TransportError;
            match $result {
                Err(TransportError::TransactionError(TransactionError::InstructionError(
                    _,
                    InstructionError::Custom(error_code),
                ))) => assert_eq!(
                    error_code,
                    $error as u32,
                    "Expected custom error with code for {}, got different code.",
                    stringify!($error)
                ),
                unexpected => panic!(
                    "Expected {} error, not {:?}",
                    stringify!($error),
                    unexpected
                ),
            }
        }
    };
}

/// Like `assert_solido_error`, but instead of testing for a Solido error, it tests
/// for a raw error code. Can be used to test for errors returned by different programs.
#[macro_export]
macro_rules! assert_error_code {
    ($result:expr, $error_code:expr $(, /* Accept an optional trailing comma. */)?) => {
        // Open a scope so the imports don't clash.
        {
            use solana_program::instruction::InstructionError;
            use solana_sdk::transaction::TransactionError;
            use solana_sdk::transport::TransportError;
            match $result {
                Err(TransportError::TransactionError(TransactionError::InstructionError(
                    _,
                    InstructionError::Custom(error_code),
                ))) => assert_eq!(
                    error_code, $error_code as u32,
                    "Custom error has an unexpected error code.",
                ),
                unexpected => panic!(
                    "Expected custom error with code {} error, not {:?}",
                    $error_code, unexpected
                ),
            }
        }
    };
}
