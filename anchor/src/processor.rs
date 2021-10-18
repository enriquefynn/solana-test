use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

/// Processes [Instruction](enum.Instruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    msg!("ANCHOR!");
    Ok(())

    // let instruction = AnchorInstruction::try_from_slice(input)?;
    // match instruction {
    //     AnchorInstruction::Initialize => process_initialize(program_id, accounts),
    //     AnchorInstruction::Deposit { amount } => process_deposit(program_id, accounts, amount),
    //     AnchorInstruction::Withdraw { amount } => todo!(),
    //     AnchorInstruction::ClaimRewards => todo!(),
    // }
}
