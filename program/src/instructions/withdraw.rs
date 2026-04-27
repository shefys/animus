//! Withdraw SOL collateral from a position.
//!
//! The withdrawal must not push the position below the minimum
//! collateral ratio. If the position has no minted pUSD, any amount
//! up to the full collateral can be withdrawn.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. [writable]  config PDA
//!   3. [writable]  vault PDA
//!   4. []          oracle PDA
//!   5. []          system_program
//!   6. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::engine::peg;
use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, oracle, position, vault};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let owner_ai = &accounts[0];
    let position_ai = &accounts[1];
    let config_ai = &accounts[2];
    let vault_ai = &accounts[3];
    let oracle_ai = &accounts[4];
    let _system = &accounts[5];
    let clock_ai = &accounts[6];

    if !owner_ai.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;
    oracle::validate(oracle_ai)?;

    let pos_owner = position::owner(position_ai)?;
    if pos_owner != *owner_ai.key() {
        return Err(err(ERR_POSITION_OWNER_MISMATCH));
    }

    let amount = read_u64(data, 0)?;
    let current_collateral = position::collateral(position_ai)?;
    if amount > current_collateral {
        return Err(err(ERR_INSUFFICIENT_COLLATERAL));
    }

    let now = vault::current_timestamp(clock_ai)?;

    // If position has outstanding debt, verify the withdrawal is safe.
    let current_minted = position::minted(position_ai)?;
    if current_minted > 0 {
        let price = oracle::get_valid_price(oracle_ai, now)?;
        let min_ratio = config::min_collateral_ratio(config_ai)?;
        let remaining = checked_sub(current_collateral, amount)?;

        if !peg::is_healthy(remaining, current_minted, price, min_ratio) {
            return Err(err(ERR_WITHDRAW_EXCEEDS_SAFE));
        }
    }

    // Transfer SOL from vault to owner.
    // Build vault signer seeds.
    let config_key = config_ai.key();
    let vault_seeds_base: &[&[u8]] = &[VAULT_SEED, config_key.as_ref()];
    let (_vault_addr, vault_bump) =
        pinocchio::pubkey::find_program_address(vault_seeds_base, program_id);
    let bump_bytes = [vault_bump];
    let vault_signer: &[&[u8]] = &[VAULT_SEED, config_key.as_ref(), &bump_bytes];

    vault::transfer_from_vault(vault_ai, owner_ai, amount, vault_signer)?;

    // Update state.
    position::sub_collateral(position_ai, amount)?;
    position::set_last_interact(position_ai, now)?;
    config::sub_total_collateral(config_ai, amount)?;

    Ok(())
}
