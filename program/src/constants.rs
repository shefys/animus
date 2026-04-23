//! Protocol-wide constants.
//!
//! Every magic number in the program lives here. If a value appears in
//! more than one instruction handler it MUST be defined here so that
//! future parameter changes only touch one file.

/// Seed prefix for the protocol config PDA.
pub const CONFIG_SEED: &[u8] = b"config";

/// Seed prefix for the collateral vault PDA.
pub const VAULT_SEED: &[u8] = b"vault";

/// Seed prefix for individual user positions.
pub const POSITION_SEED: &[u8] = b"position";

/// Seed prefix for creature accounts.
pub const CREATURE_SEED: &[u8] = b"creature";

/// Seed prefix for the pegged token mint authority.
pub const MINT_AUTH_SEED: &[u8] = b"mint_auth";

/// Seed prefix for the oracle price feed PDA.
pub const ORACLE_SEED: &[u8] = b"oracle";

// ---------------------------------------------------------------------------
// Account sizes (bytes)
// ---------------------------------------------------------------------------

/// 8 (discriminator) + Config fields.
pub const CONFIG_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 7;
// authority(32) + vault(32) + mint(32) + min_collateral_ratio(8)
// + liquidation_bonus(8) + spawn_threshold(8) + reroll_fee(8)
// + protocol_fee_bps(8) + total_collateral(8) + total_minted(8)
// + bump(1) + _pad(7)

/// 8 (discriminator) + Position fields.
pub const POSITION_SIZE: usize = 8 + 32 + 8 + 8 + 8 + 8 + 32 + 1 + 1 + 6;
// owner(32) + collateral(8) + minted(8) + deposited_at(8)
// + last_interact(8) + creature(32) + has_creature(1) + bump(1) + _pad(6)

/// 8 (discriminator) + Creature fields.
pub const CREATURE_SIZE: usize = 8 + 32 + 32 + 8 + 2 + 1 + 1 + 1 + 1 + 2 + 8 + 8 + 8 + 8 + 8;
// owner(32) + position(32) + dna(8) + generation(2) + species(1)
// + element(1) + rarity(1) + mood(1) + power(2) + spawned_at(8)
// + evolved_at(8) + xp(8) + feeds(8) + bump(8 — actually 1+7 pad)

/// 8 (discriminator) + Oracle fields.
pub const ORACLE_SIZE: usize = 8 + 8 + 8 + 8 + 1 + 7;
// price(8) + confidence(8) + updated_at(8) + bump(1) + _pad(7)

// ---------------------------------------------------------------------------
// Protocol defaults
// ---------------------------------------------------------------------------

/// Minimum collateral ratio in basis points. 15000 = 150%.
pub const DEFAULT_MIN_COLLATERAL_RATIO: u64 = 15000;

/// Liquidation bonus in basis points. 500 = 5%.
pub const DEFAULT_LIQUIDATION_BONUS: u64 = 500;

/// Minimum lamport collateral to spawn a creature (0.5 SOL).
pub const DEFAULT_SPAWN_THRESHOLD: u64 = 500_000_000;

/// Reroll fee in lamports (0.01 SOL).
pub const DEFAULT_REROLL_FEE: u64 = 10_000_000;

/// Protocol fee on minting in basis points. 30 = 0.3%.
pub const DEFAULT_PROTOCOL_FEE_BPS: u64 = 30;

/// Basis point denominator.
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Price precision — oracle prices are stored with 6 decimals.
/// A price of 150_000_000 means $150.000000.
pub const PRICE_DECIMALS: u64 = 1_000_000;

/// pUSD has 6 decimals like USDC.
pub const PEGGED_DECIMALS: u8 = 6;

/// Lamports per SOL.
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Account discriminators — first 8 bytes of every PDA
// ---------------------------------------------------------------------------

pub const DISC_CONFIG: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x63, 0x6f, 0x6e, 0x66]; // "ppegconf"
pub const DISC_POSITION: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x70, 0x6f, 0x73, 0x6e]; // "ppegposn"
pub const DISC_CREATURE: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x63, 0x72, 0x65, 0x61]; // "ppegcrea"
pub const DISC_ORACLE: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x6f, 0x72, 0x63, 0x6c]; // "ppegorcl"

// ---------------------------------------------------------------------------
// Creature constants
// ---------------------------------------------------------------------------

/// Number of distinct species.
pub const NUM_SPECIES: u8 = 16;

/// Number of distinct elements.
pub const NUM_ELEMENTS: u8 = 8;

/// Maximum power level.
pub const MAX_POWER: u16 = 9999;

/// XP required per evolution stage.
pub const XP_PER_EVOLUTION: u64 = 100_000_000; // 0.1 SOL worth of cumulative deposits

/// Maximum generation (evolutions).
pub const MAX_GENERATION: u16 = 10;
