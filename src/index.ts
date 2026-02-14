/**
 * Emblem Vault SDK — multi-chain wallet and trading client.
 *
 * Wraps the Emblem Vault REST API for wallet management,
 * token swaps, cross-chain bridges, and market data.
 */

// ---- Types ----

export enum Chain {
  SOLANA = "solana",
  ETHEREUM = "ethereum",
  BASE = "base",
  BSC = "bsc",
  POLYGON = "polygon",
  HEDERA = "hedera",
  BITCOIN = "bitcoin",
}

export interface EmblemConfig {
  apiKey: string;
  walletPassword: string;
  baseUrl?: string;
  defaultChain?: Chain;
  timeout?: number;
}

export interface TokenBalance {
  symbol: string;
  name: string;
  balance: number;
  usdValue: number;
  chain: Chain;
  mint: string;
  decimals: number;
  priceUsd: number;
  change24h: number;
}

export interface SwapRequest {
  chain: Chain;
  fromToken: string;
  toToken: string;
  amount: number;
  slippageBps?: number;
}

export interface SwapResult {
  success: boolean;
  txHash: string | null;
  fromToken: string;
  toToken: string;
  amountIn: number;
  amountOut: number;
  priceImpact: number;
  fee: number;
  error: string | null;
}

export interface BridgeRequest {
  fromChain: Chain;
  toChain: Chain;
  token: string;
  amount: number;
}

export interface BridgeResult {
  success: boolean;
  sourceTx: string | null;
  destinationTx: string | null;
  amountSent: number;
  amountReceived: number;
  fee: number;
  estimatedTime: number; // seconds
  status: "initiated" | "bridging" | "completed" | "failed";
  error: string | null;
}

export interface TokenInfo {
  symbol: string;
  name: string;
  chain: Chain;
  priceUsd: number;
  marketCap: number;
  volume24h: number;
  change24h: number;
  change7d: number;
  holders: number;
  contract: string;
  verified: boolean;
}

export interface LimitOrderRequest {
  chain: Chain;
  fromToken: string;
  toToken: string;
  amount: number;
  triggerPrice: number;
  direction: "above" | "below";
  expiresIn?: number; // seconds
}

export interface LimitOrderResult {
  orderId: string;
  status: "active" | "filled" | "cancelled" | "expired";
  createdAt: number;
  filledAt: number | null;
  txHash: string | null;
}

export interface WalletAddress {
  chain: Chain;
  address: string;
  isDefault: boolean;
}

export interface PortfolioSummary {
  totalUsd: number;
  chainBreakdown: Record<string, number>;
  topHoldings: TokenBalance[];
  change24h: number;
  change7d: number;
}

// ---- API Client ----

export class EmblemClient {
  private _config: Required<EmblemConfig>;
  private _cache: Map<string, { data: unknown; expires: number }> = new Map();

  constructor(config: EmblemConfig) {
    this._config = {
      apiKey: config.apiKey,
      walletPassword: config.walletPassword,
      baseUrl: config.baseUrl || "https://api.emblemvault.io/v1",
      defaultChain: config.defaultChain || Chain.SOLANA,
      timeout: config.timeout || 30000,
    };
  }

  // ---- Wallet ----

  async getBalances(chain?: Chain): Promise<TokenBalance[]> {
    const target = chain || this._config.defaultChain;
    return this._post<TokenBalance[]>("/wallet/balances", {
      chain: target,
    });
  }

  async getPortfolio(): Promise<PortfolioSummary> {
    const allBalances: TokenBalance[] = [];
    for (const c of Object.values(Chain)) {
      try {
        const balances = await this.getBalances(c as Chain);
        allBalances.push(...balances);
      } catch {
        // Chain might not be configured.
      }
    }

    const totalUsd = allBalances.reduce((sum, b) => sum + b.usdValue, 0);
    const chainBreakdown: Record<string, number> = {};
    for (const b of allBalances) {
      chainBreakdown[b.chain] = (chainBreakdown[b.chain] || 0) + b.usdValue;
    }
    const topHoldings = allBalances
      .filter((b) => b.usdValue > 0)
      .sort((a, b) => b.usdValue - a.usdValue)
      .slice(0, 10);

    return {
      totalUsd,
      chainBreakdown,
      topHoldings,
      change24h: 0, // Would compute from historical data.
      change7d: 0,
    };
  }

  async getWalletAddresses(): Promise<WalletAddress[]> {
    return this._get<WalletAddress[]>("/wallet/addresses");
  }

  async getAddress(chain?: Chain): Promise<string> {
    const addresses = await this.getWalletAddresses();
    const target = chain || this._config.defaultChain;
    const match = addresses.find((a) => a.chain === target);
    return match?.address || "";
  }

  // ---- Trading ----

  async swap(request: SwapRequest): Promise<SwapResult> {
    return this._post<SwapResult>("/swap", {
      chain: request.chain,
      from_token: request.fromToken,
      to_token: request.toToken,
      amount: request.amount.toString(),
      slippage_bps: request.slippageBps || 100,
    });
  }

  async getSwapQuote(request: SwapRequest): Promise<{
    estimatedOut: number;
    priceImpact: number;
    fee: number;
    route: string[];
  }> {
    return this._post("/swap/quote", {
      chain: request.chain,
      from_token: request.fromToken,
      to_token: request.toToken,
      amount: request.amount.toString(),
    });
  }

  async createLimitOrder(request: LimitOrderRequest): Promise<LimitOrderResult> {
    return this._post<LimitOrderResult>("/orders/limit", {
      chain: request.chain,
      from_token: request.fromToken,
      to_token: request.toToken,
      amount: request.amount.toString(),
      trigger_price: request.triggerPrice.toString(),
      direction: request.direction,
      expires_in: request.expiresIn || 86400,
    });
  }

  async getOpenOrders(): Promise<LimitOrderResult[]> {
    return this._get<LimitOrderResult[]>("/orders/open");
  }

  async cancelOrder(orderId: string): Promise<{ success: boolean }> {
    return this._post("/orders/cancel", { order_id: orderId });
  }

  // ---- Bridge ----

  async bridge(request: BridgeRequest): Promise<BridgeResult> {
    return this._post<BridgeResult>("/bridge", {
      from_chain: request.fromChain,
      to_chain: request.toChain,
      token: request.token,
      amount: request.amount.toString(),
    });
  }

  async getBridgeEstimate(request: BridgeRequest): Promise<{
    estimatedReceived: number;
    fee: number;
    estimatedTime: number;
    supported: boolean;
  }> {
    return this._post("/bridge/estimate", {
      from_chain: request.fromChain,
      to_chain: request.toChain,
      token: request.token,
      amount: request.amount.toString(),
    });
  }

  // ---- Market Data ----

  async getPrice(symbol: string, chain?: Chain): Promise<number> {
    const cacheKey = `price:${symbol}:${chain || "any"}`;
    const cached = this._getCached<number>(cacheKey);
    if (cached !== null) return cached;

    const result = await this._get<{ price: number }>(
      `/market/price?symbol=${symbol}&chain=${chain || ""}`
    );
    this._setCache(cacheKey, result.price, 30);
    return result.price;
  }

  async getTokenInfo(symbol: string, chain?: Chain): Promise<TokenInfo | null> {
    try {
      return await this._get<TokenInfo>(
        `/market/token?symbol=${symbol}&chain=${chain || ""}`
      );
    } catch {
      return null;
    }
  }

  async searchTokens(query: string, chain?: Chain): Promise<TokenInfo[]> {
    return this._get<TokenInfo[]>(
      `/market/search?q=${encodeURIComponent(query)}&chain=${chain || ""}`
    );
  }

  async getTrending(chain?: Chain, limit: number = 10): Promise<TokenInfo[]> {
    return this._get<TokenInfo[]>(
      `/market/trending?chain=${chain || ""}&limit=${limit}`
    );
  }

  // ---- Utilities ----

  async checkTokenSafety(contract: string, chain: Chain): Promise<{
    isHoneypot: boolean;
    liquidityLocked: boolean;
    ownershipRenounced: boolean;
    topHolderPct: number;
    riskLevel: "low" | "medium" | "high" | "critical";
    warnings: string[];
  }> {
    return this._post("/market/safety", {
      contract,
      chain,
    });
  }

  async getTransactionHistory(
    chain?: Chain,
    limit: number = 20
  ): Promise<{
    txHash: string;
    type: string;
    chain: Chain;
    amount: number;
    token: string;
    timestamp: number;
  }[]> {
    return this._get(`/wallet/history?chain=${chain || ""}&limit=${limit}`);
  }

  // ---- HTTP Methods ----

  private async _get<T>(path: string): Promise<T> {
    const url = `${this._config.baseUrl}${path}`;
    const response = await fetch(url, {
      method: "GET",
      headers: this._headers(),
      signal: AbortSignal.timeout(this._config.timeout),
    });
    if (!response.ok) {
      throw new Error(`Emblem API error: ${response.status} ${response.statusText}`);
    }
    return response.json() as Promise<T>;
  }

  private async _post<T>(path: string, body: Record<string, unknown>): Promise<T> {
    const url = `${this._config.baseUrl}${path}`;
    const response = await fetch(url, {
      method: "POST",
      headers: this._headers(),
      body: JSON.stringify({
        ...body,
        password: this._config.walletPassword,
      }),
      signal: AbortSignal.timeout(this._config.timeout),
    });
    if (!response.ok) {
      throw new Error(`Emblem API error: ${response.status} ${response.statusText}`);
    }
    return response.json() as Promise<T>;
  }

  private _headers(): Record<string, string> {
    return {
      "Authorization": `Bearer ${this._config.apiKey}`,
      "Content-Type": "application/json",
      "X-Client": "emblem-vault-sdk/0.3.0",
    };
  }

  private _getCached<T>(key: string): T | null {
    const entry = this._cache.get(key);
    if (entry && Date.now() < entry.expires) {
      return entry.data as T;
    }
    this._cache.delete(key);
    return null;
  }

  private _setCache(key: string, data: unknown, ttlSeconds: number): void {
    this._cache.set(key, {
      data,
      expires: Date.now() + ttlSeconds * 1000,
    });
  }
}

export default EmblemClient;
