//! P-Peg: pinocchio-based peg stability with on-chain creatures.
//!
//! The protocol has two halves that share one pool of collateral:
//!
//!   1. **Peg stability module** — deposit SOL, mint pUSD at the oracle
//!      price, redeem pUSD for SOL. Positions are overcollateralized;
//!      undercollateralized positions can be liquidated by anyone.
//!
//!   2. **Creature engine** — every position above the spawn threshold
//!      can summon an on-chain creature whose DNA is derived from the
//!      position parameters + slot entropy. Creatures have species,
//!      element, rarity, and power. They evolve when you add collateral.
//!      They die when you get liquidated. You can reroll for a new one
//!      by burning a fee.
//!
//! The entire program is written with pinocchio (zero external deps) so
//! every instruction fits inside tight CU budgets. State is zero-copy:
//! we read and write directly into account data without intermediate
//! structs or borsh overhead.
//!
//! # Instruction index
//!
//! | ix  | name             | description                              |
//! |-----|------------------|------------------------------------------|
//! |  0  | Initialize       | create protocol config + vault            |
//! |  1  | Deposit          | add SOL collateral to a position          |
//! |  2  | Withdraw         | remove SOL collateral from a position     |
//! |  3  | MintPegged       | mint pUSD against a collateralized pos    |
//! |  4  | Redeem           | burn pUSD and unlock collateral           |
//! |  5  | Liquidate        | liquidate an undercollateralized pos      |
//! |  6  | SpawnCreature    | spawn a creature from a position          |
//! |  7  | Evolve           | evolve creature by adding more collateral |
//! |  8  | Reroll           | burn creature, reroll new DNA             |
//! |  9  | UpdateOracle     | authority updates the price feed          |
//! | 10  | UpdateConfig     | authority updates protocol parameters     |

#![no_std]

pinocchio::entrypoint!(process_instruction);
pinocchio::default_allocator!();
pinocchio::default_panic_handler!();

mod constants;
mod engine;
mod error;
mod instruction;
mod instructions;
mod processor;
mod state;

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// Program entrypoint. Thin wrapper that delegates to the processor.
///
/// We do NOT deserialize instruction data into a rust enum. Instead
/// the processor reads the first byte (the instruction index) and
/// passes the remaining slice to the corresponding handler. This
/// saves the CU cost of constructing an intermediate enum value.
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    processor::process(program_id, accounts, data)
}
