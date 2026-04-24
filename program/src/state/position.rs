//! User position account.
//!
//! Each user has at most one position per protocol instance. The
//! position tracks how much SOL collateral they deposited and how
//! much pUSD they minted against it.
//!
//! Layout:
//!
//!   offset  size  field
//!   ------  ----  -----
//!    0       8    discriminator
//!    8      32    owner pubkey
//!   40       8    collateral (lamports)
//!   48       8    minted (pUSD raw units)
//!   56       8    deposited_at (unix timestamp)
//!   64       8    last_interact (unix timestamp)
//!   72      32    creature pubkey (zero if none)
//!  104       1    has_creature (bool)
//!  105       1    bump
//!  106       6    _reserved
//!  ------  ----
//!  total   112

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::constants::{POSITION_SIZE, DISC_POSITION};
use crate::error::{err, ERR_INVALID_DISCRIMINATOR, ERR_INVALID_ACCOUNT_DATA, ERR_ARITHMETIC_OVERFLOW};

const OFF_DISC: usize = 0;
const OFF_OWNER: usize = 8;
const OFF_COLLATERAL: usize = 40;
const OFF_MINTED: usize = 48;
const OFF_DEPOSITED_AT: usize = 56;
const OFF_LAST_INTERACT: usize = 64;
const OFF_CREATURE: usize = 72;
const OFF_HAS_CREATURE: usize = 104;
const OFF_BUMP: usize = 105;

#[inline]
pub fn validate(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    if data.len() < POSITION_SIZE {
        return Err(err(ERR_INVALID_ACCOUNT_DATA));
    }
    if data[OFF_DISC..OFF_DISC + 8] != DISC_POSITION {
        return Err(err(ERR_INVALID_DISCRIMINATOR));
    }
    Ok(())
}

// --- readers ----------------------------------------------------------------

#[inline]
pub fn owner(account: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[OFF_OWNER..OFF_OWNER + 32]);
    Ok(Pubkey::from(buf))
}

#[inline]
pub fn collateral(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_COLLATERAL..OFF_COLLATERAL + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn minted(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_MINTED..OFF_MINTED + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn deposited_at(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_DEPOSITED_AT..OFF_DEPOSITED_AT + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn last_interact(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_LAST_INTERACT..OFF_LAST_INTERACT + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn creature(account: &AccountInfo) -> Result<Pubkey, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[OFF_CREATURE..OFF_CREATURE + 32]);
    Ok(Pubkey::from(buf))
}

#[inline]
pub fn has_creature(account: &AccountInfo) -> Result<bool, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    Ok(data[OFF_HAS_CREATURE] != 0)
}

#[inline]
pub fn bump(account: &AccountInfo) -> Result<u8, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    Ok(data[OFF_BUMP])
}

// --- writers ----------------------------------------------------------------

#[inline]
pub fn write_discriminator(account: &AccountInfo) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_DISC..OFF_DISC + 8].copy_from_slice(&DISC_POSITION);
    Ok(())
}

#[inline]
pub fn set_owner(account: &AccountInfo, val: &Pubkey) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_OWNER..OFF_OWNER + 32].copy_from_slice(val.as_ref());
    Ok(())
}

#[inline]
pub fn set_collateral(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_COLLATERAL..OFF_COLLATERAL + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_minted(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_MINTED..OFF_MINTED + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_deposited_at(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_DEPOSITED_AT..OFF_DEPOSITED_AT + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_last_interact(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_LAST_INTERACT..OFF_LAST_INTERACT + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_creature(account: &AccountInfo, val: &Pubkey) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_CREATURE..OFF_CREATURE + 32].copy_from_slice(val.as_ref());
    Ok(())
}

#[inline]
pub fn set_has_creature(account: &AccountInfo, val: bool) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_HAS_CREATURE] = val as u8;
    Ok(())
}

#[inline]
pub fn set_bump(account: &AccountInfo, val: u8) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_BUMP] = val;
    Ok(())
}

// --- atomic updaters --------------------------------------------------------

#[inline]
pub fn add_collateral(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_COLLATERAL..OFF_COLLATERAL + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(delta).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_COLLATERAL..OFF_COLLATERAL + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

#[inline]
pub fn sub_collateral(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_COLLATERAL..OFF_COLLATERAL + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_sub(delta).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_COLLATERAL..OFF_COLLATERAL + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

#[inline]
pub fn add_minted(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_MINTED..OFF_MINTED + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(delta).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_MINTED..OFF_MINTED + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

#[inline]
pub fn sub_minted(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_MINTED..OFF_MINTED + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_sub(delta).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_MINTED..OFF_MINTED + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}
