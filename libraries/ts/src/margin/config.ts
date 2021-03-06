import { Address } from "@project-serum/anchor"
import MARGIN_CONFIG from "./config.json"

export type MarginTokens = "BTC" | "ETH" | "MSRM" | "SOL" | "SRM" | "USDC"
export type MarginOracles = "BTC_USD" | "ETH_USD" | "SOL_USD" | "SRM_USD"
export type MarginPools = "BTC" | "ETH" | "SOL" | "SRM" | "USDC"
export type MarginMarkets = "BTC_USDC" | "ETH_USDC"

export type MarginCluster = keyof typeof MARGIN_CONFIG | MarginConfig

export interface MarginConfig {
  controlProgramId: Address
  marginProgramId: Address
  marginPoolProgramId: Address
  marginSerumProgramId: Address
  marginSwapProgramId: Address
  metadataProgramId: Address
  orcaSwapProgramId: Address
  pythProgramId: Address
  serumProgramId: Address
  serumReferralAuthority: Address
  splTokenFaucet?: Address
  url: string
  tokens: Record<MarginTokens, MarginTokenConfig>
  oracles: Record<MarginOracles, MarginOracleConfig>
  pools: Record<MarginPools, MarginPoolConfig>
  markets: Record<MarginMarkets, MarginMarketConfig>
}

export interface MarginTokenConfig {
  symbol: MarginTokens
  name: string
  decimals: number
  precision: number
  faucet?: Address
  faucetLimit?: number
  mint: Address
}

export interface MarginOracleConfig {
  symbol: string
  address: Address
  product: Address
}

export interface MarginPoolConfig {
  symbol: MarginPools
  name: string
  tokenMint: Address
  oracle: Address
  product: Address
  feesVault: Address
}

export interface MarginMarketConfig {
  symbol: MarginMarkets
  market: Address
  baseMint: Address
  baseDecimals: number
  baseVault: Address
  baseSymbol: MarginTokens
  quoteMint: Address
  quoteDecimals: number
  quoteVault: Address
  quoteSymbol: MarginTokens
  requestQueue: Address
  eventQueue: Address
  bids: Address
  asks: Address
  quoteDustThreshold: number
  baseLotSize: number
  quoteLotSize: number
  feeRateBps: number
}
