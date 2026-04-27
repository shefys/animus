//! Initialize the p-peg protocol.
//!
//! Creates the config PDA, vault PDA, oracle PDA, and sets protocol
//! parameters. The pegged token mint must already exist and its mint
//! authority must be the mint_auth PDA derived from this program.
//!
//! Accounts:
//!   0. [signer]    authority — protocol admin
//!   1. [writable]  config PDA
//!   2. [writable]  vault PDA (system account)
//!   3. [writable]  oracle PDA
//!   4. []          pegged_mint — the pUSD mint
//!   5. []          system_program
//!   6. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::error::{err, ERR_ACCOUNT_NOT_SIGNER, ERR_ALREADY_INITIALIZED};
use crate::instruction::read_u64;
use crate::state::{config, oracle};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    // --- unpack accounts ----------------------------------------------------

    let authority = &accounts[0];
    let config_ai = &accounts[1];
    let vault_ai = &accounts[2];
    let oracle_ai = &accounts[3];
    let mint_ai = &accounts[4];
    let _system = &accounts[5];

    // --- guards -------------------------------------------------------------

    if !authority.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    // Verify config is not already initialized (check if data is all zeros).
    {
        let data_ref = config_ai.try_borrow_data()
            .map_err(|_| err(crate::error::ERR_INVALID_ACCOUNT_DATA))?;
        if data_ref.len() >= 8 && data_ref[0..8] == DISC_CONFIG {
            return Err(err(ERR_ALREADY_INITIALIZED));
        }
    }

    // --- read instruction data ----------------------------------------------

    let min_ratio = if data.len() >= 8 { read_u64(data, 0)? } else { DEFAULT_MIN_COLLATERAL_RATIO };
    let liq_bonus = if data.len() >= 16 { read_u64(data, 8)? } else { DEFAULT_LIQUIDATION_BONUS };
    let spawn_thresh = if data.len() >= 24 { read_u64(data, 16)? } else { DEFAULT_SPAWN_THRESHOLD };
    let reroll_fee = if data.len() >= 32 { read_u64(data, 24)? } else { DEFAULT_REROLL_FEE };
    let protocol_fee = if data.len() >= 40 { read_u64(data, 32)? } else { DEFAULT_PROTOCOL_FEE_BPS };

    // --- derive PDAs and verify ---------------------------------------------

    let config_seeds: &[&[u8]] = &[CONFIG_SEED, authority.key().as_ref()];
    let (expected_config, config_bump) =
        pinocchio::pubkey::find_program_address(config_seeds, program_id);
    if *config_ai.key() != expected_config {
        return Err(err(crate::error::ERR_INVALID_PDA));
    }

    let vault_seeds: &[&[u8]] = &[VAULT_SEED, config_ai.key().as_ref()];
    let (expected_vault, _vault_bump) =
        pinocchio::pubkey::find_program_address(vault_seeds, program_id);
    if *vault_ai.key() != expected_vault {
        return Err(err(crate::error::ERR_INVALID_PDA));
    }

    let oracle_seeds: &[&[u8]] = &[ORACLE_SEED, config_ai.key().as_ref()];
    let (expected_oracle, oracle_bump) =
        pinocchio::pubkey::find_program_address(oracle_seeds, program_id);
    if *oracle_ai.key() != expected_oracle {
        return Err(err(crate::error::ERR_INVALID_PDA));
    }

    // --- write config state -------------------------------------------------

    config::write_discriminator(config_ai)?;
    config::set_authority(config_ai, authority.key())?;
    config::set_vault(config_ai, vault_ai.key())?;
    config::set_pegged_mint(config_ai, mint_ai.key())?;
    config::set_min_collateral_ratio(config_ai, min_ratio)?;
    config::set_liquidation_bonus(config_ai, liq_bonus)?;
    config::set_spawn_threshold(config_ai, spawn_thresh)?;
    config::set_reroll_fee(config_ai, reroll_fee)?;
    config::set_protocol_fee_bps(config_ai, protocol_fee)?;
    config::set_total_collateral(config_ai, 0)?;
    config::set_total_minted(config_ai, 0)?;
    config::set_bump(config_ai, config_bump)?;

    // --- write oracle state -------------------------------------------------

    oracle::write_discriminator(oracle_ai)?;
    oracle::set_price(oracle_ai, 0)?;
    oracle::set_confidence(oracle_ai, 0)?;
    oracle::set_updated_at(oracle_ai, 0)?;
    oracle::set_bump(oracle_ai, oracle_bump)?;

    Ok(())
}
