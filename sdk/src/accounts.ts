/**
 * Account data deserialization.
 *
 * Each decoder reads raw account data (Buffer) and returns a typed
 * object. The layouts exactly mirror the Rust state definitions.
 */

import { PublicKey, Connection, AccountInfo } from "@solana/web3.js";
import BN from "bn.js";
import {
  ProtocolConfig,
  Position,
  Creature,
  OracleState,
  DISC_CONFIG,
  DISC_POSITION,
  DISC_CREATURE,
  DISC_ORACLE,
  PROGRAM_ID,
} from "./types";

// ---------------------------------------------------------------------------
// Read helpers
// ---------------------------------------------------------------------------

function readPubkey(buf: Buffer, offset: number): PublicKey {
  return new PublicKey(buf.subarray(offset, offset + 32));
}

function readU64(buf: Buffer, offset: number): BN {
  return new BN(buf.subarray(offset, offset + 8), "le");
}

function readU16(buf: Buffer, offset: number): number {
  return buf.readUInt16LE(offset);
}

function readU8(buf: Buffer, offset: number): number {
  return buf[offset];
}

function readBool(buf: Buffer, offset: number): boolean {
  return buf[offset] !== 0;
}

function checkDiscriminator(buf: Buffer, expected: Buffer): boolean {
  return buf.subarray(0, 8).equals(expected);
}

// ---------------------------------------------------------------------------
// Decoders
// ---------------------------------------------------------------------------

export function decodeConfig(data: Buffer): ProtocolConfig | null {
  if (data.length < 168 || !checkDiscriminator(data, DISC_CONFIG)) {
    return null;
  }

  return {
    authority: readPubkey(data, 8),
    vault: readPubkey(data, 40),
    peggedMint: readPubkey(data, 72),
    minCollateralRatio: readU64(data, 104),
    liquidationBonus: readU64(data, 112),
    spawnThreshold: readU64(data, 120),
    rerollFee: readU64(data, 128),
    protocolFeeBps: readU64(data, 136),
    totalCollateral: readU64(data, 144),
    totalMinted: readU64(data, 152),
    bump: readU8(data, 160),
  };
}

export function decodePosition(data: Buffer): Position | null {
  if (data.length < 112 || !checkDiscriminator(data, DISC_POSITION)) {
    return null;
  }

  return {
    owner: readPubkey(data, 8),
    collateral: readU64(data, 40),
    minted: readU64(data, 48),
    depositedAt: readU64(data, 56),
    lastInteract: readU64(data, 64),
    creature: readPubkey(data, 72),
    hasCreature: readBool(data, 104),
    bump: readU8(data, 105),
  };
}

export function decodeCreature(data: Buffer): Creature | null {
  if (data.length < 128 || !checkDiscriminator(data, DISC_CREATURE)) {
    return null;
  }

  return {
    owner: readPubkey(data, 8),
    position: readPubkey(data, 40),
    dna: readU64(data, 72),
    generation: readU16(data, 80),
    species: readU8(data, 82),
    element: readU8(data, 83),
    rarity: readU8(data, 84),
    mood: readU8(data, 85),
    power: readU16(data, 86),
    spawnedAt: readU64(data, 88),
    evolvedAt: readU64(data, 96),
    xp: readU64(data, 104),
    feeds: readU64(data, 112),
    bump: readU8(data, 120),
  };
}

export function decodeOracle(data: Buffer): OracleState | null {
  if (data.length < 40 || !checkDiscriminator(data, DISC_ORACLE)) {
    return null;
  }

  return {
    price: readU64(data, 8),
    confidence: readU64(data, 16),
    updatedAt: readU64(data, 24),
    bump: readU8(data, 32),
  };
}

// ---------------------------------------------------------------------------
// Fetchers — load and decode from RPC
// ---------------------------------------------------------------------------

export async function fetchConfig(
  connection: Connection,
  address: PublicKey
): Promise<ProtocolConfig | null> {
  const info = await connection.getAccountInfo(address);
  if (!info || !info.data) return null;
  return decodeConfig(Buffer.from(info.data));
}

export async function fetchPosition(
  connection: Connection,
  address: PublicKey
): Promise<Position | null> {
  const info = await connection.getAccountInfo(address);
  if (!info || !info.data) return null;
  return decodePosition(Buffer.from(info.data));
}

export async function fetchCreature(
  connection: Connection,
  address: PublicKey
): Promise<Creature | null> {
  const info = await connection.getAccountInfo(address);
  if (!info || !info.data) return null;
  return decodeCreature(Buffer.from(info.data));
}

export async function fetchOracle(
  connection: Connection,
  address: PublicKey
): Promise<OracleState | null> {
  const info = await connection.getAccountInfo(address);
  if (!info || !info.data) return null;
  return decodeOracle(Buffer.from(info.data));
}

/**
 * Fetch all position accounts for the protocol.
 *
 * Uses getProgramAccounts with a discriminator filter to find all
 * positions. WARNING: this can be expensive on mainnet.
 */
export async function fetchAllPositions(
  connection: Connection,
  programId: PublicKey = PROGRAM_ID
): Promise<{ pubkey: PublicKey; account: Position }[]> {
  const accounts = await connection.getProgramAccounts(programId, {
    filters: [
      { memcmp: { offset: 0, bytes: DISC_POSITION.toString("base64") } },
    ],
  });

  const results: { pubkey: PublicKey; account: Position }[] = [];
  for (const { pubkey, account } of accounts) {
    const decoded = decodePosition(Buffer.from(account.data));
    if (decoded) {
      results.push({ pubkey, account: decoded });
    }
  }
  return results;
}

/**
 * Fetch all creature accounts for the protocol.
 */
export async function fetchAllCreatures(
  connection: Connection,
  programId: PublicKey = PROGRAM_ID
): Promise<{ pubkey: PublicKey; account: Creature }[]> {
  const accounts = await connection.getProgramAccounts(programId, {
    filters: [
      { memcmp: { offset: 0, bytes: DISC_CREATURE.toString("base64") } },
    ],
  });

  const results: { pubkey: PublicKey; account: Creature }[] = [];
  for (const { pubkey, account } of accounts) {
    const decoded = decodeCreature(Buffer.from(account.data));
    if (decoded) {
      results.push({ pubkey, account: decoded });
    }
  }
  return results;
}
