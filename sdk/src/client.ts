/**
 * High-level client for the p-peg protocol.
 *
 * Wraps PDA derivation, instruction building, and account fetching
 * into a single ergonomic interface. Usage:
 *
 *   const client = new PPegClient(connection, wallet);
 *   await client.initialize();
 *   await client.deposit(new BN(1_000_000_000)); // 1 SOL
 *   await client.mintPegged(new BN(50_000_000));  // 50 pUSD
 *   await client.spawnCreature();
 */

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Keypair,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import BN from "bn.js";
import {
  ProtocolConfig,
  Position,
  Creature,
  OracleState,
  PROGRAM_ID,
  InitializeParams,
  CONFIG_SIZE,
  POSITION_SIZE,
  CREATURE_SIZE,
  ORACLE_SIZE,
} from "./types";
import {
  deriveConfig,
  deriveVault,
  derivePosition,
  deriveCreature,
  deriveMintAuthority,
  deriveOracle,
  deriveAll,
} from "./pda";
import {
  createInitializeInstruction,
  createDepositInstruction,
  createWithdrawInstruction,
  createMintPeggedInstruction,
  createRedeemInstruction,
  createLiquidateInstruction,
  createSpawnCreatureInstruction,
  createEvolveInstruction,
  createRerollInstruction,
  createUpdateOracleInstruction,
  createUpdateConfigInstruction,
} from "./instructions";
import {
  fetchConfig,
  fetchPosition,
  fetchCreature,
  fetchOracle,
  fetchAllPositions,
  fetchAllCreatures,
} from "./accounts";
import * as creatures from "./creatures";

export interface Wallet {
  publicKey: PublicKey;
  signTransaction(tx: Transaction): Promise<Transaction>;
  signAllTransactions?(txs: Transaction[]): Promise<Transaction[]>;
}

export class PPegClient {
  readonly connection: Connection;
  readonly wallet: Wallet;
  readonly programId: PublicKey;

  // Cached PDAs (derived lazily).
  private _config?: PublicKey;
  private _vault?: PublicKey;
  private _oracle?: PublicKey;
  private _mintAuth?: PublicKey;
  private _position?: PublicKey;
  private _creature?: PublicKey;
  private _configState?: ProtocolConfig;

  constructor(
    connection: Connection,
    wallet: Wallet,
    programId: PublicKey = PROGRAM_ID
  ) {
    this.connection = connection;
    this.wallet = wallet;
    this.programId = programId;
  }

  // ---- PDA accessors -------------------------------------------------------

  get configAddress(): PublicKey {
    if (!this._config) {
      [this._config] = deriveConfig(this.wallet.publicKey, this.programId);
    }
    return this._config;
  }

  get vaultAddress(): PublicKey {
    if (!this._vault) {
      [this._vault] = deriveVault(this.configAddress, this.programId);
    }
    return this._vault;
  }

  get oracleAddress(): PublicKey {
    if (!this._oracle) {
      [this._oracle] = deriveOracle(this.configAddress, this.programId);
    }
    return this._oracle;
  }

  get mintAuthAddress(): PublicKey {
    if (!this._mintAuth) {
      [this._mintAuth] = deriveMintAuthority(this.configAddress, this.programId);
    }
    return this._mintAuth;
  }

  get positionAddress(): PublicKey {
    if (!this._position) {
      [this._position] = derivePosition(
        this.configAddress,
        this.wallet.publicKey,
        this.programId
      );
    }
    return this._position;
  }

  get creatureAddress(): PublicKey {
    if (!this._creature) {
      [this._creature] = deriveCreature(this.positionAddress, this.programId);
    }
    return this._creature;
  }

  // ---- Account fetchers ----------------------------------------------------

  async getConfig(): Promise<ProtocolConfig | null> {
    return fetchConfig(this.connection, this.configAddress);
  }

  async getPosition(): Promise<Position | null> {
    return fetchPosition(this.connection, this.positionAddress);
  }

  async getCreature(): Promise<Creature | null> {
    return fetchCreature(this.connection, this.creatureAddress);
  }

  async getOracle(): Promise<OracleState | null> {
    return fetchOracle(this.connection, this.oracleAddress);
  }

  async getAllPositions() {
    return fetchAllPositions(this.connection, this.programId);
  }

  async getAllCreatures() {
    return fetchAllCreatures(this.connection, this.programId);
  }

  // ---- Transaction builders ------------------------------------------------

  async initialize(
    peggedMint: PublicKey,
    params: InitializeParams = {}
  ): Promise<string> {
    const pdas = deriveAll(this.wallet.publicKey, this.programId);

    // Create PDA accounts.
    const createConfigIx = SystemProgram.createAccount({
      fromPubkey: this.wallet.publicKey,
      newAccountPubkey: pdas.config,
      space: CONFIG_SIZE,
      lamports: await this.connection.getMinimumBalanceForRentExemption(CONFIG_SIZE),
      programId: this.programId,
    });

    const createOracleIx = SystemProgram.createAccount({
      fromPubkey: this.wallet.publicKey,
      newAccountPubkey: pdas.oracle,
      space: ORACLE_SIZE,
      lamports: await this.connection.getMinimumBalanceForRentExemption(ORACLE_SIZE),
      programId: this.programId,
    });

    const initIx = createInitializeInstruction(
      this.wallet.publicKey,
      pdas.config,
      pdas.vault,
      pdas.oracle,
      peggedMint,
      params,
      this.programId
    );

    return this.sendTransaction([createConfigIx, createOracleIx, initIx]);
  }

  async deposit(amount: BN): Promise<string> {
    // Ensure position account exists.
    const ixs: TransactionInstruction[] = [];
    const posInfo = await this.connection.getAccountInfo(this.positionAddress);
    if (!posInfo) {
      ixs.push(
        SystemProgram.createAccount({
          fromPubkey: this.wallet.publicKey,
          newAccountPubkey: this.positionAddress,
          space: POSITION_SIZE,
          lamports: await this.connection.getMinimumBalanceForRentExemption(POSITION_SIZE),
          programId: this.programId,
        })
      );
    }

    ixs.push(
      createDepositInstruction(
        this.wallet.publicKey,
        this.positionAddress,
        this.configAddress,
        this.vaultAddress,
        { amount },
        this.programId
      )
    );

    return this.sendTransaction(ixs);
  }

  async withdraw(amount: BN): Promise<string> {
    const ix = createWithdrawInstruction(
      this.wallet.publicKey,
      this.positionAddress,
      this.configAddress,
      this.vaultAddress,
      this.oracleAddress,
      { amount },
      this.programId
    );
    return this.sendTransaction([ix]);
  }

  async mintPegged(amount: BN, peggedMint: PublicKey): Promise<string> {
    const ata = await getAssociatedTokenAddress(peggedMint, this.wallet.publicKey);

    const ixs: TransactionInstruction[] = [];

    // Create ATA if needed.
    const ataInfo = await this.connection.getAccountInfo(ata);
    if (!ataInfo) {
      ixs.push(
        createAssociatedTokenAccountInstruction(
          this.wallet.publicKey,
          ata,
          this.wallet.publicKey,
          peggedMint
        )
      );
    }

    ixs.push(
      createMintPeggedInstruction(
        this.wallet.publicKey,
        this.positionAddress,
        this.configAddress,
        this.oracleAddress,
        peggedMint,
        ata,
        this.mintAuthAddress,
        { amount },
        this.programId
      )
    );

    return this.sendTransaction(ixs);
  }

  async redeem(amount: BN, peggedMint: PublicKey): Promise<string> {
    const ata = await getAssociatedTokenAddress(peggedMint, this.wallet.publicKey);
    const ix = createRedeemInstruction(
      this.wallet.publicKey,
      this.positionAddress,
      this.configAddress,
      peggedMint,
      ata,
      { amount },
      this.programId
    );
    return this.sendTransaction([ix]);
  }

  async spawnCreature(): Promise<string> {
    // Create creature account.
    const ixs: TransactionInstruction[] = [];
    const creatureInfo = await this.connection.getAccountInfo(this.creatureAddress);
    if (!creatureInfo) {
      ixs.push(
        SystemProgram.createAccount({
          fromPubkey: this.wallet.publicKey,
          newAccountPubkey: this.creatureAddress,
          space: CREATURE_SIZE,
          lamports: await this.connection.getMinimumBalanceForRentExemption(CREATURE_SIZE),
          programId: this.programId,
        })
      );
    }

    ixs.push(
      createSpawnCreatureInstruction(
        this.wallet.publicKey,
        this.positionAddress,
        this.creatureAddress,
        this.configAddress,
        this.programId
      )
    );

    return this.sendTransaction(ixs);
  }

  async evolve(feedAmount: BN): Promise<string> {
    const ix = createEvolveInstruction(
      this.wallet.publicKey,
      this.positionAddress,
      this.creatureAddress,
      this.configAddress,
      this.vaultAddress,
      { feedAmount },
      this.programId
    );
    return this.sendTransaction([ix]);
  }

  async reroll(): Promise<string> {
    const ix = createRerollInstruction(
      this.wallet.publicKey,
      this.positionAddress,
      this.creatureAddress,
      this.configAddress,
      this.vaultAddress,
      this.programId
    );
    return this.sendTransaction([ix]);
  }

  // ---- Creature display helpers --------------------------------------------

  creatureTitle(c: Creature): string {
    return creatures.creatureTitle(c);
  }

  creatureDescription(c: Creature): string {
    return creatures.generateDescription(c);
  }

  creatureEffectivePower(c: Creature): number {
    return creatures.effectivePower(c);
  }

  creatureEvolutionProgress(c: Creature): number {
    return creatures.evolutionProgress(c);
  }

  // ---- Internal helpers ----------------------------------------------------

  private async sendTransaction(ixs: TransactionInstruction[]): Promise<string> {
    const tx = new Transaction().add(...ixs);
    tx.feePayer = this.wallet.publicKey;
    tx.recentBlockhash = (
      await this.connection.getLatestBlockhash()
    ).blockhash;

    const signed = await this.wallet.signTransaction(tx);
    const raw = signed.serialize();
    return this.connection.sendRawTransaction(raw, {
      skipPreflight: false,
      preflightCommitment: "confirmed",
    });
  }
}
