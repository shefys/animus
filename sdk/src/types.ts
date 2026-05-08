/**
 * Type definitions for the p-peg protocol.
 *
 * These mirror the on-chain account layouts and instruction data
 * structures. All numeric fields are BN or number depending on
 * whether they can exceed Number.MAX_SAFE_INTEGER.
 */

import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

// ---------------------------------------------------------------------------
// Constants (must match program/src/constants.rs)
// ---------------------------------------------------------------------------

export const PROGRAM_ID = new PublicKey(
  "PPeg1111111111111111111111111111111111111111"
);

export const CONFIG_SEED = Buffer.from("config");
export const VAULT_SEED = Buffer.from("vault");
export const POSITION_SEED = Buffer.from("position");
export const CREATURE_SEED = Buffer.from("creature");
export const MINT_AUTH_SEED = Buffer.from("mint_auth");
export const ORACLE_SEED = Buffer.from("oracle");

export const BPS_DENOMINATOR = 10_000;
export const PRICE_DECIMALS = 1_000_000;
export const LAMPORTS_PER_SOL = 1_000_000_000;
export const PEGGED_DECIMALS = 6;

export const XP_PER_EVOLUTION = new BN(100_000_000);
export const MAX_GENERATION = 10;
export const MAX_POWER = 9999;

// Account discriminators.
export const DISC_CONFIG = Buffer.from([0x70, 0x70, 0x65, 0x67, 0x63, 0x6f, 0x6e, 0x66]);
export const DISC_POSITION = Buffer.from([0x70, 0x70, 0x65, 0x67, 0x70, 0x6f, 0x73, 0x6e]);
export const DISC_CREATURE = Buffer.from([0x70, 0x70, 0x65, 0x67, 0x63, 0x72, 0x65, 0x61]);
export const DISC_ORACLE = Buffer.from([0x70, 0x70, 0x65, 0x67, 0x6f, 0x72, 0x63, 0x6c]);

// Account sizes.
export const CONFIG_SIZE = 168;
export const POSITION_SIZE = 112;
export const CREATURE_SIZE = 128;
export const ORACLE_SIZE = 40;

// Instruction indices.
export enum Instruction {
  Initialize = 0,
  Deposit = 1,
  Withdraw = 2,
  MintPegged = 3,
  Redeem = 4,
  Liquidate = 5,
  SpawnCreature = 6,
  Evolve = 7,
  Reroll = 8,
  UpdateOracle = 9,
  UpdateConfig = 10,
}

// ---------------------------------------------------------------------------
// Account state types
// ---------------------------------------------------------------------------

export interface ProtocolConfig {
  authority: PublicKey;
  vault: PublicKey;
  peggedMint: PublicKey;
  minCollateralRatio: BN;
  liquidationBonus: BN;
  spawnThreshold: BN;
  rerollFee: BN;
  protocolFeeBps: BN;
  totalCollateral: BN;
  totalMinted: BN;
  bump: number;
}

export interface Position {
  owner: PublicKey;
  collateral: BN;
  minted: BN;
  depositedAt: BN;
  lastInteract: BN;
  creature: PublicKey;
  hasCreature: boolean;
  bump: number;
}

export interface Creature {
  owner: PublicKey;
  position: PublicKey;
  dna: BN;
  generation: number;
  species: number;
  element: number;
  rarity: number;
  mood: number;
  power: number;
  spawnedAt: BN;
  evolvedAt: BN;
  xp: BN;
  feeds: BN;
  bump: number;
}

export interface OracleState {
  price: BN;
  confidence: BN;
  updatedAt: BN;
  bump: number;
}

// ---------------------------------------------------------------------------
// Creature trait enums
// ---------------------------------------------------------------------------

export const SPECIES_NAMES = [
  "Gremlin", "Imp", "Sprite", "Wisp",
  "Drake", "Basilisk", "Golem", "Wraith",
  "Phoenix", "Leviathan", "Chimera", "Hydra",
  "Kraken", "Behemoth", "Wyrm", "Archon",
] as const;

export const ELEMENT_NAMES = [
  "Fire", "Water", "Earth", "Air",
  "Shadow", "Light", "Chaos", "Void",
] as const;

export const RARITY_NAMES = [
  "Common", "Uncommon", "Rare",
  "Epic", "Legendary", "Mythic",
] as const;

export const MOOD_NAMES = [
  "Idle", "Curious", "Hungry", "Playful",
  "Aggressive", "Sleepy", "Proud", "Feral",
] as const;

export const RARITY_COLORS: Record<number, string> = {
  0: "#9ca3af", // common — gray
  1: "#22c55e", // uncommon — green
  2: "#3b82f6", // rare — blue
  3: "#a855f7", // epic — purple
  4: "#f59e0b", // legendary — amber
  5: "#ef4444", // mythic — red
};

export type SpeciesName = (typeof SPECIES_NAMES)[number];
export type ElementName = (typeof ELEMENT_NAMES)[number];
export type RarityName = (typeof RARITY_NAMES)[number];
export type MoodName = (typeof MOOD_NAMES)[number];

// ---------------------------------------------------------------------------
// Instruction parameter types
// ---------------------------------------------------------------------------

export interface InitializeParams {
  minCollateralRatio?: BN;
  liquidationBonus?: BN;
  spawnThreshold?: BN;
  rerollFee?: BN;
  protocolFeeBps?: BN;
}

export interface DepositParams {
  amount: BN;
}

export interface WithdrawParams {
  amount: BN;
}

export interface MintPeggedParams {
  amount: BN;
}

export interface RedeemParams {
  amount: BN;
}

export interface LiquidateParams {
  repayAmount: BN;
}

export interface EvolveParams {
  feedAmount: BN;
}

export interface UpdateOracleParams {
  price: BN;
  confidence?: BN;
}

export interface UpdateConfigParams {
  fieldIndex: number;
  newValue: BN;
}
