import { Address } from "@project-serum/anchor";
export declare const MARGIN_CONFIG_URL = "https://storage.googleapis.com/jet-app-config/config.json";
export declare type MarginCluster = "localnet" | "devnet" | "mainnet-beta" | MarginConfig;
export interface MarginConfig {
    controlProgramId: Address;
    marginProgramId: Address;
    marginPoolProgramId: Address;
    marginSerumProgramId: Address;
    marginSwapProgramId: Address;
    metadataProgramId: Address;
    orcaSwapProgramId: Address;
    serumProgramId: Address;
    faucetProgramId?: Address;
    url: string;
    tokens: Record<string, MarginTokenConfig>;
    markets: Record<string, MarginMarketConfig>;
}
export interface MarginTokenConfig {
    symbol: string;
    name: string;
    decimals: number;
    precision: number;
    faucet?: Address;
    faucetLimit?: number;
    mint: Address;
}
export interface MarginMarketConfig {
    symbol: string;
    market: Address;
    baseMint: Address;
    baseDecimals: number;
    baseVault: Address;
    baseSymbol: string;
    quoteMint: Address;
    quoteDecimals: number;
    quoteVault: Address;
    quoteSymbol: string;
    requestQueue: Address;
    eventQueue: Address;
    bids: Address;
    asks: Address;
    quoteDustThreshold: number;
    baseLotSize: number;
    quoteLotSize: number;
    feeRateBps: number;
}
export declare function getLatestConfig(cluster: string): Promise<MarginConfig>;
//# sourceMappingURL=config.d.ts.map