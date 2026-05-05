//! Rarity distribution computation.
//!
//! Rarity is determined by a combination of a random seed (from DNA)
//! and the collateral size. Larger positions shift the probability
//! distribution toward rarer outcomes, but even small positions can
//! occasionally spawn a legendary creature.
//!
//! The distribution is tiered:
//!
//!   | rarity    | index | base probability | collateral shift       |
//!   |-----------|-------|------------------|------------------------|
//!   | common    |   0   | ~53%             | decreases with SOL     |
//!   | uncommon  |   1   | ~25%             | stable                 |
//!   | rare      |   2   | ~13%             | increases slightly     |
//!   | epic      |   3   | ~6%              | increases with SOL     |
//!   | legendary |   4   | ~2.5%            | increases with SOL     |
//!   | mythic    |   5   | ~0.5%            | requires ≥ 100 SOL     |
//!
//! The thresholds are expressed as values out of 64 (matching the
//! 6-bit rarity seed range 0–63).

/// Base rarity thresholds (cumulative, out of 64).
///
/// seed  0–33  → common     (34/64 = 53.1%)
/// seed 34–49  → uncommon   (16/64 = 25.0%)
/// seed 50–57  → rare       ( 8/64 = 12.5%)
/// seed 58–61  → epic       ( 4/64 =  6.3%)
/// seed 62–63  → legendary  ( 2/64 =  3.1%)
/// (mythic requires collateral ≥ 100 SOL and seed == 63)
const BASE_THRESHOLDS: [u8; 5] = [34, 50, 58, 62, 64];

/// Collateral tiers (in SOL) that shift the distribution.
///
/// At each tier, the thresholds shift downward, making rarer
/// outcomes more likely.
const TIER_SHIFTS: [(u64, u8); 5] = [
    (1, 0),     // < 1 SOL: no shift
    (5, 2),     // 5+ SOL: -2 on thresholds
    (10, 4),    // 10+ SOL: -4 on thresholds
    (50, 8),    // 50+ SOL: -8 on thresholds
    (100, 12),  // 100+ SOL: -12 on thresholds (mythic becomes possible)
];

/// Compute the rarity level from a seed value and SOL amount.
///
/// Returns 0–5 (common through mythic).
#[inline]
pub fn compute_rarity(seed: u8, sol_amount: u64) -> u8 {
    // Determine the shift based on collateral tier.
    let shift = determine_shift(sol_amount);

    // Apply shift to thresholds.
    let t0 = BASE_THRESHOLDS[0].saturating_sub(shift);
    let t1 = BASE_THRESHOLDS[1].saturating_sub(shift);
    let t2 = BASE_THRESHOLDS[2].saturating_sub(shift);
    let t3 = BASE_THRESHOLDS[3].saturating_sub(shift);

    // Mythic requires ≥ 100 SOL AND the highest seed value.
    if sol_amount >= 100 && seed >= 63u8.saturating_sub(shift / 2) {
        return 5; // mythic
    }

    if seed < t0 {
        0 // common
    } else if seed < t1 {
        1 // uncommon
    } else if seed < t2 {
        2 // rare
    } else if seed < t3 {
        3 // epic
    } else {
        4 // legendary
    }
}

/// Determine the threshold shift for a given SOL amount.
#[inline]
fn determine_shift(sol_amount: u64) -> u8 {
    let mut shift = 0u8;
    for &(threshold, s) in TIER_SHIFTS.iter() {
        if sol_amount >= threshold {
            shift = s;
        }
    }
    shift
}

/// Compute the probability of each rarity level for display purposes.
///
/// Returns an array of 6 probabilities (in basis points, totalling 10000).
/// Used by the SDK to show spawn odds to the user.
#[inline]
pub fn rarity_probabilities(sol_amount: u64) -> [u16; 6] {
    let shift = determine_shift(sol_amount);

    let t0 = BASE_THRESHOLDS[0].saturating_sub(shift) as u16;
    let t1 = BASE_THRESHOLDS[1].saturating_sub(shift) as u16;
    let t2 = BASE_THRESHOLDS[2].saturating_sub(shift) as u16;
    let t3 = BASE_THRESHOLDS[3].saturating_sub(shift) as u16;
    let t4 = 64u16; // total range

    let mythic_range = if sol_amount >= 100 {
        let mythic_thresh = 63u16.saturating_sub((shift / 2) as u16);
        t4.saturating_sub(mythic_thresh)
    } else {
        0
    };

    let legendary_range = t4.saturating_sub(t3).saturating_sub(mythic_range);
    let epic_range = t3.saturating_sub(t2);
    let rare_range = t2.saturating_sub(t1);
    let uncommon_range = t1.saturating_sub(t0);
    let common_range = t0;

    // Convert to basis points (multiply by 10000/64).
    let scale = |v: u16| -> u16 { ((v as u32) * 10000 / 64) as u16 };

    let mut probs = [
        scale(common_range),
        scale(uncommon_range),
        scale(rare_range),
        scale(epic_range),
        scale(legendary_range),
        scale(mythic_range),
    ];

    // Ensure they sum to 10000 (adjust common for rounding).
    let sum: u16 = probs.iter().sum();
    if sum < 10000 {
        probs[0] += 10000 - sum;
    } else if sum > 10000 {
        probs[0] = probs[0].saturating_sub(sum - 10000);
    }

    probs
}

/// Given a rarity level, return the XP multiplier in basis points.
///
/// Rarer creatures gain XP faster, making evolution easier for
/// lucky spawns. This compounds with the rarity upgrade on evolution.
///
///   common=10000, uncommon=11500, rare=13000,
///   epic=15000, legendary=18000, mythic=22000
pub const RARITY_XP_MULTIPLIERS: [u16; 6] = [
    10000, 11500, 13000, 15000, 18000, 22000,
];

/// Apply the rarity XP multiplier to a base XP amount.
#[inline]
pub fn apply_xp_multiplier(base_xp: u64, rarity_level: u8) -> u64 {
    let idx = (rarity_level as usize).min(5);
    let multiplier = RARITY_XP_MULTIPLIERS[idx] as u64;
    base_xp.saturating_mul(multiplier) / 10000
}
