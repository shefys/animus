//! Instruction dispatcher.
//!
//! Reads the first byte of instruction data to determine which handler
//! to invoke, then passes the remaining bytes and accounts to that
//! handler. No enum construction, no match on a deserialized value —
//! just a jump table on a single u8.

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::{err, ERR_INVALID_INSTRUCTION};
use crate::instruction::*;
use crate::instructions;

/// Main dispatch. Called from the entrypoint.
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let (tag, rest) = data.split_first().ok_or(err(ERR_INVALID_INSTRUCTION))?;

    match *tag {
        IX_INITIALIZE => instructions::initialize::process(program_id, accounts, rest),
        IX_DEPOSIT => instructions::deposit::process(program_id, accounts, rest),
        IX_WITHDRAW => instructions::withdraw::process(program_id, accounts, rest),
        IX_MINT_PEGGED => instructions::mint_pegged::process(program_id, accounts, rest),
        IX_REDEEM => instructions::redeem::process(program_id, accounts, rest),
        IX_LIQUIDATE => instructions::liquidate::process(program_id, accounts, rest),
        IX_SPAWN_CREATURE => instructions::spawn_creature::process(program_id, accounts, rest),
        IX_EVOLVE => instructions::evolve::process(program_id, accounts, rest),
        IX_REROLL => instructions::reroll::process(program_id, accounts, rest),
        IX_UPDATE_ORACLE => instructions::update_oracle::process(program_id, accounts, rest),
        IX_UPDATE_CONFIG => instructions::update_config::process(program_id, accounts, rest),
        _ => Err(err(ERR_INVALID_INSTRUCTION)),
    }
}
