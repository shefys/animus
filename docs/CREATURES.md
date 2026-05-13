# Creature Engine

On-chain creatures that live and die with your DeFi positions.

## DNA

Every creature has a 64-bit DNA value computed deterministically from on-chain entropy at spawn time. The inputs are:

- Position pubkey (32 bytes)
- Owner pubkey (32 bytes)
- Current unix timestamp (8 bytes)
- Slot hashes sysvar (first 32 bytes)

These are XORed together and mixed through a 4-round cascade function. The result is folded from 32 bytes down to 8 bytes (the DNA), with an additional mixing pass to spread entropy.

## Trait Derivation

DNA is split into bit-fields:

| Bits   | Size    | Trait            | Range        |
|--------|---------|------------------|--------------|
| 0–3    | 4 bits  | Species          | 0–15         |
| 4–6    | 3 bits  | Element          | 0–7          |
| 7–9    | 3 bits  | Base mood        | 0–7          |
| 10–15  | 6 bits  | Rarity seed      | 0–63         |
| 16–31  | 16 bits | Power seed       | 0–65535      |
| 32–47  | 16 bits | Visual variant   | 0–65535      |
| 48–63  | 16 bits | Personality hash | 0–65535      |

### Species (16 types)

Gremlin, Imp, Sprite, Wisp, Drake, Basilisk, Golem, Wraith, Phoenix, Leviathan, Chimera, Hydra, Kraken, Behemoth, Wyrm, Archon.

### Elements (8 types)

Fire, Water, Earth, Air, Shadow, Light, Chaos, Void.

### Rarity

Rarity is not purely random. It depends on both the DNA rarity seed (0–63) AND the position's collateral at spawn time. Larger positions shift the probability distribution toward rarer outcomes.

Base thresholds (out of 64): Common < 34, Uncommon < 50, Rare < 58, Epic < 62, Legendary < 64. For positions ≥ 100 SOL, Mythic becomes possible.

Collateral tiers shift all thresholds down: 5 SOL (-2), 10 SOL (-4), 50 SOL (-8), 100 SOL (-12).

### Power

Base power comes from the DNA power seed (mod 1000, giving 0–999). Collateral adds a bonus: `sqrt(SOL_amount) * 100`, capped at 4000. Total is capped at 9999.

Effective power includes a rarity multiplier: Common 1.00x, Uncommon 1.15x, Rare 1.30x, Epic 1.45x, Legendary 1.60x, Mythic 1.75x.

## Evolution

Creatures evolve by accumulating XP. XP is gained by feeding — depositing additional collateral through the Evolve instruction. The amount of collateral deposited equals the XP gained.

Evolution thresholds increase linearly: Gen 0→1 requires 0.1 SOL, Gen 1→2 requires 0.2 SOL, etc. Maximum generation is 10.

Each evolution:
- Bumps the generation counter
- Recalculates power: `base + (gen * 200) + sqrt(SOL) * 50`
- May increase rarity (every 3 generations, +1 rarity level)
- Shifts mood (even gens → "proud", odd gens → cycle)

## Creature Death

Creatures die when their parent position is liquidated. The creature account data is zeroed — this is permanent. There is no way to revive a dead creature.

This is the core emotional incentive: keep your position healthy or lose your creature forever.

## Reroll

If you don't like your creature, you can reroll for a new one. This:
1. Deducts a reroll fee from your position's collateral
2. Generates new DNA from fresh entropy (new timestamp + old DNA + slot hashes)
3. Resets the creature to generation 0 with 0 XP

You keep the creature account (same PDA), but all traits are recomputed. This is a gamble — you might get something rarer, or you might get a common gremlin.
