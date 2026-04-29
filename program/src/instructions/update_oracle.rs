//! Update the oracle price feed.
//!
//! Only the protocol authority can update the price. In production
//! this would be replaced by a Pyth/Switchboard integration, but
//! the protocol is oracle-agnostic — it only reads price + confidence
//! from the oracle account.
//!
//! Accounts:
//!   0. [signer]    authority
//!   1. []          config PDA
//!   2. [writable]  oracle PDA
//!   3. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, oracle, vault};

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let authority = &accounts[0];
    let config_ai = &accounts[1];
    let oracle_ai = &accounts[2];
    let clock_ai = &accounts[3];

    if !authority.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    oracle::validate(oracle_ai)?;

    // Verify authority.
    let expected_auth = config::authority(config_ai)?;
    if expected_auth != *authority.key() {
        return Err(err(ERR_INVALID_AUTHORITY));
    }

    let price = read_u64(data, 0)?;
    let confidence = if data.len() >= 16 { read_u64(data, 8)? } else { 0 };

    if price == 0 {
        return Err(err(ERR_ORACLE_PRICE_ZERO));
    }

    let now = vault::current_timestamp(clock_ai)?;

    oracle::set_price(oracle_ai, price)?;
    oracle::set_confidence(oracle_ai, confidence)?;
    oracle::set_updated_at(oracle_ai, now)?;

    Ok(())
}
