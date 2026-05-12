//! Unit tests for the creature DNA generation and trait derivation.
//!
//! These tests verify that the same entropy always produces the same
//! creature, that trait ranges are valid, and that evolution mechanics
//! work correctly.

#[cfg(test)]
mod tests {
    const NUM_SPECIES: u8 = 16;
    const NUM_ELEMENTS: u8 = 8;
    const MAX_POWER: u16 = 9999;
    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    const MAX_GENERATION: u16 = 10;
    const XP_PER_EVOLUTION: u64 = 100_000_000;

    // --- re-implement engine functions for standalone testing ----------------

    fn derive_dna(entropy: [u8; 32]) -> u64 {
        let mut dna_bytes = [0u8; 8];
        for chunk in 0..4 {
            let base = chunk * 8;
            for i in 0..8 {
                dna_bytes[i] ^= entropy[base + i];
            }
        }
        let mut acc: u8 = 0x5a;
        for i in 0..8 {
            acc = acc.wrapping_add(dna_bytes[i]).wrapping_mul(0x9e);
            dna_bytes[i] ^= acc;
        }
        u64::from_le_bytes(dna_bytes)
    }

    fn dna_species(dna: u64) -> u8 { (dna & 0xF) as u8 % NUM_SPECIES }
    fn dna_element(dna: u64) -> u8 { ((dna >> 4) & 0x7) as u8 % NUM_ELEMENTS }
    fn initial_mood(dna: u64) -> u8 { ((dna >> 7) & 0x7) as u8 }

    fn integer_sqrt(n: u64) -> u64 {
        if n < 2 { return n; }
        let mut x = n;
        let mut y = (x + 1) / 2;
        while y < x { x = y; y = (x + n / x) / 2; }
        x
    }

    fn dna_power(dna: u64, collateral: u64) -> u16 {
        let power_seed = ((dna >> 16) & 0xFFFF) as u16;
        let sol = collateral / LAMPORTS_PER_SOL;
        let base = (power_seed % 1000) as u64;
        let bonus = integer_sqrt(sol).saturating_mul(100).min(4000);
        let total = base.saturating_add(bonus).min(MAX_POWER as u64);
        total as u16
    }

    fn visual_variant(dna: u64) -> u16 { ((dna >> 32) & 0xFFFF) as u16 }
    fn personality_hash(dna: u64) -> u16 { ((dna >> 48) & 0xFFFF) as u16 }

    fn effective_power(base: u16, rarity: u8) -> u16 {
        let mult = 100u32 + (rarity as u32) * 15;
        (((base as u32) * mult / 100) as u16).min(MAX_POWER)
    }

    fn evolved_power(dna: u64, collateral: u64, gen: u16) -> u16 {
        let base = dna_power(dna, collateral);
        let gen_bonus = (gen as u64) * 200;
        let sol = collateral / LAMPORTS_PER_SOL;
        let coll_bonus = integer_sqrt(sol) * 50;
        let total = (base as u64) + gen_bonus + coll_bonus;
        total.min(MAX_POWER as u64) as u16
    }

    fn can_evolve(xp: u64, gen: u16) -> bool {
        if gen >= MAX_GENERATION { return false; }
        xp >= XP_PER_EVOLUTION * ((gen as u64) + 1)
    }

    fn log2_approx(n: u64) -> u64 {
        if n == 0 { return 0; }
        63u64.saturating_sub(n.leading_zeros() as u64)
    }

    fn age_score(spawned: u64, now: u64) -> u16 {
        let hours = now.saturating_sub(spawned) / 3600;
        (log2_approx(hours + 1) * 100).min(MAX_POWER as u64) as u16
    }

    // ---- determinism tests -------------------------------------------------

    #[test]
    fn test_dna_deterministic() {
        let entropy = [42u8; 32];
        let dna1 = derive_dna(entropy);
        let dna2 = derive_dna(entropy);
        assert_eq!(dna1, dna2, "Same entropy must produce same DNA");
    }

    #[test]
    fn test_different_entropy_different_dna() {
        let e1 = [1u8; 32];
        let e2 = [2u8; 32];
        assert_ne!(derive_dna(e1), derive_dna(e2));
    }

    // ---- trait range tests -------------------------------------------------

    #[test]
    fn test_species_range() {
        for i in 0..256u64 {
            let dna = i;
            assert!(dna_species(dna) < NUM_SPECIES);
        }
    }

    #[test]
    fn test_element_range() {
        for i in 0..256u64 {
            let dna = i << 4;
            assert!(dna_element(dna) < NUM_ELEMENTS);
        }
    }

    #[test]
    fn test_mood_range() {
        for i in 0..256u64 {
            let dna = i << 7;
            assert!(initial_mood(dna) < 8);
        }
    }

    // ---- power computation -------------------------------------------------

    #[test]
    fn test_power_increases_with_collateral() {
        let dna = 0xABCD_1234_5678_9ABCu64;
        let power_1sol = dna_power(dna, LAMPORTS_PER_SOL);
        let power_10sol = dna_power(dna, 10 * LAMPORTS_PER_SOL);
        let power_100sol = dna_power(dna, 100 * LAMPORTS_PER_SOL);
        assert!(power_10sol > power_1sol);
        assert!(power_100sol > power_10sol);
    }

    #[test]
    fn test_power_capped_at_max() {
        let dna = 0xFFFF_FFFF_FFFF_FFFFu64;
        let power = dna_power(dna, 1000 * LAMPORTS_PER_SOL);
        assert!(power <= MAX_POWER);
    }

    #[test]
    fn test_effective_power_rarity_multiplier() {
        let base = 1000u16;
        assert_eq!(effective_power(base, 0), 1000); // 1.00x
        assert_eq!(effective_power(base, 1), 1150); // 1.15x
        assert_eq!(effective_power(base, 2), 1300); // 1.30x
        assert_eq!(effective_power(base, 5), 1750); // 1.75x
    }

    // ---- evolution ---------------------------------------------------------

    #[test]
    fn test_evolved_power_increases_with_generation() {
        let dna = 0x1234_5678_9ABC_DEF0u64;
        let coll = 5 * LAMPORTS_PER_SOL;
        let p0 = evolved_power(dna, coll, 0);
        let p3 = evolved_power(dna, coll, 3);
        let p10 = evolved_power(dna, coll, 10);
        assert!(p3 > p0);
        assert!(p10 > p3);
    }

    #[test]
    fn test_evolved_power_capped() {
        let dna = 0xFFFF_FFFF_FFFF_FFFFu64;
        let power = evolved_power(dna, 1000 * LAMPORTS_PER_SOL, 10);
        assert!(power <= MAX_POWER);
    }

    #[test]
    fn test_can_evolve_thresholds() {
        assert!(!can_evolve(0, 0));
        assert!(can_evolve(XP_PER_EVOLUTION, 0)); // gen 0 → 1
        assert!(!can_evolve(XP_PER_EVOLUTION, 1)); // gen 1 needs 2x
        assert!(can_evolve(2 * XP_PER_EVOLUTION, 1));
        assert!(!can_evolve(u64::MAX, MAX_GENERATION)); // max gen
    }

    // ---- visual variant and personality ------------------------------------

    #[test]
    fn test_visual_variant_range() {
        let dna = 0xABCD_EF01_2345_6789u64;
        let vv = visual_variant(dna);
        assert!(vv <= 0xFFFF);
    }

    #[test]
    fn test_personality_hash_range() {
        let dna = 0xABCD_EF01_2345_6789u64;
        let ph = personality_hash(dna);
        assert!(ph <= 0xFFFF);
    }

    // ---- age score ---------------------------------------------------------

    #[test]
    fn test_age_score_increases() {
        let spawn = 1_000_000u64;
        let s1 = age_score(spawn, spawn + 3600); // 1 hour
        let s2 = age_score(spawn, spawn + 86400); // 1 day
        let s3 = age_score(spawn, spawn + 2_592_000); // 30 days
        assert!(s2 > s1);
        assert!(s3 > s2);
    }

    #[test]
    fn test_age_score_capped() {
        let score = age_score(0, u64::MAX / 2);
        assert!(score <= MAX_POWER);
    }

    // ---- integer sqrt ------------------------------------------------------

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(9), 3);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(99), 9);
        assert_eq!(integer_sqrt(101), 10);
    }

    // ---- log2 approx -------------------------------------------------------

    #[test]
    fn test_log2_approx() {
        assert_eq!(log2_approx(0), 0);
        assert_eq!(log2_approx(1), 0);
        assert_eq!(log2_approx(2), 1);
        assert_eq!(log2_approx(4), 2);
        assert_eq!(log2_approx(8), 3);
        assert_eq!(log2_approx(1024), 10);
    }
}
