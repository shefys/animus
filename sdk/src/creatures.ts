/**
 * Creature display helpers.
 *
 * Utilities for rendering creature information, computing derived
 * stats, and generating descriptive text from on-chain data.
 */

import BN from "bn.js";
import {
  Creature,
  SPECIES_NAMES,
  ELEMENT_NAMES,
  RARITY_NAMES,
  MOOD_NAMES,
  RARITY_COLORS,
  MAX_POWER,
  XP_PER_EVOLUTION,
  MAX_GENERATION,
  LAMPORTS_PER_SOL,
  BPS_DENOMINATOR,
} from "./types";

// ---------------------------------------------------------------------------
// Display formatting
// ---------------------------------------------------------------------------

/** Get the species display name for a creature. */
export function speciesName(creature: Creature): string {
  return SPECIES_NAMES[creature.species] ?? "Unknown";
}

/** Get the element display name. */
export function elementName(creature: Creature): string {
  return ELEMENT_NAMES[creature.element] ?? "Unknown";
}

/** Get the rarity display name. */
export function rarityName(creature: Creature): string {
  return RARITY_NAMES[creature.rarity] ?? "Unknown";
}

/** Get the mood display name. */
export function moodName(creature: Creature): string {
  return MOOD_NAMES[creature.mood] ?? "Unknown";
}

/** Get the rarity color (hex string). */
export function rarityColor(creature: Creature): string {
  return RARITY_COLORS[creature.rarity] ?? "#9ca3af";
}

/** Format the creature's full title: "Legendary Fire Drake (Gen 4)". */
export function creatureTitle(creature: Creature): string {
  const rarity = rarityName(creature);
  const element = elementName(creature);
  const species = speciesName(creature);
  const gen = creature.generation;
  return `${rarity} ${element} ${species} (Gen ${gen})`;
}

// ---------------------------------------------------------------------------
// Stat computation
// ---------------------------------------------------------------------------

/** Compute effective power including rarity multiplier. */
export function effectivePower(creature: Creature): number {
  const multiplier = 100 + creature.rarity * 15;
  return Math.min(Math.floor(creature.power * multiplier / 100), MAX_POWER);
}

/** Compute power as a percentage of max power. */
export function powerPercent(creature: Creature): number {
  return Math.round((effectivePower(creature) / MAX_POWER) * 100);
}

/** XP needed for the next evolution. */
export function xpToNextEvolution(creature: Creature): BN {
  if (creature.generation >= MAX_GENERATION) {
    return new BN(0);
  }
  const threshold = XP_PER_EVOLUTION.muln(creature.generation + 1);
  const remaining = threshold.sub(creature.xp);
  return remaining.isNeg() ? new BN(0) : remaining;
}

/** XP progress toward next evolution as a percentage (0–100). */
export function evolutionProgress(creature: Creature): number {
  if (creature.generation >= MAX_GENERATION) return 100;
  const threshold = XP_PER_EVOLUTION.muln(creature.generation + 1);
  if (threshold.isZero()) return 100;
  const progress = creature.xp.muln(100).div(threshold);
  return Math.min(progress.toNumber(), 100);
}

/** Whether the creature can evolve right now. */
export function canEvolve(creature: Creature): boolean {
  if (creature.generation >= MAX_GENERATION) return false;
  const threshold = XP_PER_EVOLUTION.muln(creature.generation + 1);
  return creature.xp.gte(threshold);
}

/** Compute the age of a creature in seconds. */
export function ageSeconds(creature: Creature, nowUnix: number): number {
  return Math.max(0, nowUnix - creature.spawnedAt.toNumber());
}

/** Format age as a human-readable string. */
export function ageDisplay(creature: Creature, nowUnix: number): string {
  const secs = ageSeconds(creature, nowUnix);
  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const mins = Math.floor((secs % 3600) / 60);

  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

/** Compute an age score (logarithmic). Older creatures score higher. */
export function ageScore(creature: Creature, nowUnix: number): number {
  const secs = ageSeconds(creature, nowUnix);
  const hours = secs / 3600;
  return Math.min(Math.floor(Math.log2(hours + 1) * 100), MAX_POWER);
}

// ---------------------------------------------------------------------------
// DNA bit-field extraction (mirrors the Rust engine)
// ---------------------------------------------------------------------------

/** Extract visual variant seed from DNA (bits 32–47). */
export function visualVariant(dna: BN): number {
  return dna.shrn(32).and(new BN(0xFFFF)).toNumber();
}

/** Extract personality hash from DNA (bits 48–63). */
export function personalityHash(dna: BN): number {
  return dna.shrn(48).and(new BN(0xFFFF)).toNumber();
}

// ---------------------------------------------------------------------------
// Rarity probability display
// ---------------------------------------------------------------------------

const BASE_THRESHOLDS = [34, 50, 58, 62, 64];
const TIER_SHIFTS: [number, number][] = [
  [1, 0], [5, 2], [10, 4], [50, 8], [100, 12],
];

/** Compute the rarity probability distribution for a given SOL amount. */
export function rarityProbabilities(solAmount: number): number[] {
  let shift = 0;
  for (const [threshold, s] of TIER_SHIFTS) {
    if (solAmount >= threshold) shift = s;
  }

  const t0 = Math.max(0, BASE_THRESHOLDS[0] - shift);
  const t1 = Math.max(0, BASE_THRESHOLDS[1] - shift);
  const t2 = Math.max(0, BASE_THRESHOLDS[2] - shift);
  const t3 = Math.max(0, BASE_THRESHOLDS[3] - shift);

  const mythicRange = solAmount >= 100
    ? Math.max(0, 64 - (63 - Math.floor(shift / 2)))
    : 0;

  const scale = (v: number) => Math.round(v * 10000 / 64);

  const probs = [
    scale(t0),
    scale(t1 - t0),
    scale(t2 - t1),
    scale(t3 - t2),
    scale(64 - t3 - mythicRange),
    scale(mythicRange),
  ];

  // Adjust for rounding.
  const sum = probs.reduce((a, b) => a + b, 0);
  probs[0] += 10000 - sum;

  return probs;
}

/** Format rarity probabilities as a display string. */
export function rarityOddsDisplay(solAmount: number): string {
  const probs = rarityProbabilities(solAmount);
  return RARITY_NAMES
    .map((name, i) => `${name}: ${(probs[i] / 100).toFixed(1)}%`)
    .join(", ");
}

// ---------------------------------------------------------------------------
// Creature description generator
// ---------------------------------------------------------------------------

const ADJECTIVES = [
  "ancient", "restless", "cunning", "volatile", "patient",
  "fierce", "enigmatic", "radiant", "phantom", "primordial",
  "stoic", "mercurial", "venomous", "spectral", "arcane",
  "feral",
];

const ORIGINS = [
  "born from pure collateral entropy",
  "forged in the protocol's first block",
  "spawned from a whale's conviction",
  "emerged from the oracle's shadow",
  "crystallized at the peg boundary",
  "summoned by overcollateralization",
  "manifested from vault resonance",
  "awakened by a liquidation shock",
];

/** Generate a unique description for a creature based on its DNA. */
export function generateDescription(creature: Creature): string {
  const phash = personalityHash(creature.dna);
  const vvar = visualVariant(creature.dna);

  const adj = ADJECTIVES[phash % ADJECTIVES.length];
  const origin = ORIGINS[vvar % ORIGINS.length];

  const species = speciesName(creature);
  const element = elementName(creature).toLowerCase();
  const rarity = rarityName(creature).toLowerCase();

  const genText = creature.generation > 0
    ? ` It has evolved ${creature.generation} time${creature.generation > 1 ? "s" : ""}, growing stronger with each feeding.`
    : " It awaits its first evolution.";

  return `A ${adj} ${rarity} ${species} of the ${element} affinity, ${origin}.${genText}`;
}
