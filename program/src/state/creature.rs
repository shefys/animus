//! On-chain creature account.
//!
//! Every creature is deterministically derived from its DNA — a u64
//! computed from position parameters and slot entropy at spawn time.
//! The DNA is split into bit-fields to produce species, element,
//! base rarity, and initial power. These fields are stored explicitly
//! so clients can read traits without re-deriving.
//!
//! Creatures evolve through feeding (adding collateral to the parent
//! position). Each evolution bumps the generation counter and mutates
//! the creature's traits according to the engine rules.
//!
//! Layout:
//!
//!   offset  size  field
//!   ------  ----  -----
//!    0       8    discriminator
//!    8      32    owner pubkey
//!   40      32    position pubkey
//!   72       8    dna (u64)
//!   80       2    generation (u16)
//!   82       1    species (u8, 0–15)
//!   83       1    element (u8, 0–7)
//!   84       1    rarity (u8, 0–5: common/uncommon/rare/epic/legend/mythic)
//!   85       1    mood (u8, 0–7)
//!   86       2    power (u16, 0–9999)
//!   88       8    spawned_at (unix timestamp)
//!   96       8    evolved_at (unix timestamp)
//!  104       8    xp (cumulative lamports fed)
//!  112       8    feeds (number of feed events)
//!  120       1    bump
//!  121       7    _reserved
//!  ------  ----
//!  total   128

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::constants::DISC_CREATURE;
use crate::error::{err, ERR_INVALID_DISCRIMINATOR, ERR_INVALID_ACCOUNT_DATA, ERR_ARITHMETIC_OVERFLOW};

const CREATURE_ACCOUNT_SIZE: usize = 128;

const OFF_DISC: usize = 0;
const OFF_OWNER: usize = 8;
const OFF_POSITION: usize = 40;
const OFF_DNA: usize = 72;
const OFF_GENERATION: usize = 80;
const OFF_SPECIES: usize = 82;
const OFF_ELEMENT: usize = 83;
const OFF_RARITY: usize = 84;
const OFF_MOOD: usize = 85;
const OFF_POWER: usize = 86;
const OFF_SPAWNED_AT: usize = 88;
const OFF_EVOLVED_AT: usize = 96;
const OFF_XP: usize = 104;
const OFF_FEEDS: usize = 112;
const OFF_BUMP: usize = 120;

pub const ACCOUNT_SIZE: usize = CREATURE_ACCOUNT_SIZE;

#[inline]
pub fn validate(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    if data.len() < CREATURE_ACCOUNT_SIZE {
        return Err(err(ERR_INVALID_ACCOUNT_DATA));
    }
    if data[OFF_DISC..OFF_DISC + 8] != DISC_CREATURE {
        return Err(err(ERR_INVALID_DISCRIMINATOR));
    }
    Ok(())
}

// ---- readers ---------------------------------------------------------------

macro_rules! creature_read_pubkey {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<Pubkey, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            let mut buf = [0u8; 32];
            buf.copy_from_slice(&data[$off..$off + 32]);
            Ok(Pubkey::from(buf))
        }
    };
}

macro_rules! creature_read_u64 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<u64, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&data[$off..$off + 8]);
            Ok(u64::from_le_bytes(buf))
        }
    };
}

macro_rules! creature_read_u16 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<u16, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            Ok(u16::from_le_bytes([data[$off], data[$off + 1]]))
        }
    };
}

macro_rules! creature_read_u8 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<u8, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            Ok(data[$off])
        }
    };
}

creature_read_pubkey!(owner, OFF_OWNER);
creature_read_pubkey!(position, OFF_POSITION);
creature_read_u64!(dna, OFF_DNA);
creature_read_u16!(generation, OFF_GENERATION);
creature_read_u8!(species, OFF_SPECIES);
creature_read_u8!(element, OFF_ELEMENT);
creature_read_u8!(rarity, OFF_RARITY);
creature_read_u8!(mood, OFF_MOOD);
creature_read_u16!(power, OFF_POWER);
creature_read_u64!(spawned_at, OFF_SPAWNED_AT);
creature_read_u64!(evolved_at, OFF_EVOLVED_AT);
creature_read_u64!(xp, OFF_XP);
creature_read_u64!(feeds, OFF_FEEDS);
creature_read_u8!(bump, OFF_BUMP);

// ---- writers ---------------------------------------------------------------

#[inline]
pub fn write_discriminator(account: &AccountInfo) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_DISC..OFF_DISC + 8].copy_from_slice(&DISC_CREATURE);
    Ok(())
}

macro_rules! creature_write_pubkey {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: &Pubkey) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$off..$off + 32].copy_from_slice(val.as_ref());
            Ok(())
        }
    };
}

macro_rules! creature_write_u64 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$off..$off + 8].copy_from_slice(&val.to_le_bytes());
            Ok(())
        }
    };
}

macro_rules! creature_write_u16 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: u16) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$off..$off + 2].copy_from_slice(&val.to_le_bytes());
            Ok(())
        }
    };
}

macro_rules! creature_write_u8 {
    ($name:ident, $off:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: u8) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$off] = val;
            Ok(())
        }
    };
}

creature_write_pubkey!(set_owner, OFF_OWNER);
creature_write_pubkey!(set_position, OFF_POSITION);
creature_write_u64!(set_dna, OFF_DNA);
creature_write_u16!(set_generation, OFF_GENERATION);
creature_write_u8!(set_species, OFF_SPECIES);
creature_write_u8!(set_element, OFF_ELEMENT);
creature_write_u8!(set_rarity, OFF_RARITY);
creature_write_u8!(set_mood, OFF_MOOD);
creature_write_u16!(set_power, OFF_POWER);
creature_write_u64!(set_spawned_at, OFF_SPAWNED_AT);
creature_write_u64!(set_evolved_at, OFF_EVOLVED_AT);
creature_write_u64!(set_xp, OFF_XP);
creature_write_u64!(set_feeds, OFF_FEEDS);
creature_write_u8!(set_bump, OFF_BUMP);

// ---- atomic updaters -------------------------------------------------------

#[inline]
pub fn add_xp(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_XP..OFF_XP + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(delta).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_XP..OFF_XP + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

#[inline]
pub fn increment_feeds(account: &AccountInfo) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_FEEDS..OFF_FEEDS + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(1).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_FEEDS..OFF_FEEDS + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

#[inline]
pub fn increment_generation(account: &AccountInfo) -> Result<u16, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let current = u16::from_le_bytes([data[OFF_GENERATION], data[OFF_GENERATION + 1]]);
    let new = current.checked_add(1).ok_or(err(ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_GENERATION..OFF_GENERATION + 2].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

// ---- trait names (for logging / SDK) ----------------------------------------

pub const SPECIES_NAMES: [&str; 16] = [
    "gremlin",    "imp",        "sprite",     "wisp",
    "drake",      "basilisk",   "golem",      "wraith",
    "phoenix",    "leviathan",  "chimera",    "hydra",
    "kraken",     "behemoth",   "wyrm",       "archon",
];

pub const ELEMENT_NAMES: [&str; 8] = [
    "fire", "water", "earth", "air", "shadow", "light", "chaos", "void",
];

pub const RARITY_NAMES: [&str; 6] = [
    "common", "uncommon", "rare", "epic", "legendary", "mythic",
];

pub const MOOD_NAMES: [&str; 8] = [
    "idle", "curious", "hungry", "playful", "aggressive", "sleepy", "proud", "feral",
];
