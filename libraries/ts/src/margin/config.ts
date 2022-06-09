import { Address } from "@project-serum/anchor"
import MARGIN_CONFIG from "./config.json"

export type MarginTokens = "BTC" | "ETH" | "MSRM" | "SOL" | "SRM" | "USDC"
export type MarginOracles = "BTC_USD" | "ETH_USD" | "SOL_USD" | "SRM_USD"
export type MarginPools = "BTC" | "ETH" | "SOL" | "SRM"
export type MarginMarkets = "BTC_USDC" | "ETH_USDC" | "SOL_USDC" | "SRM_USDC"

export type MarginCluster = keyof typeof MARGIN_CONFIG | MarginConfig

export interface MarginConfig {
  controlProgramId: Address
  marginProgramId: Address
  marginPoolProgramId: Address
  marginSerumProgramId: Address
  marginSwapProgramId: Address
  metadataProgramId: Address
  pythProgramId: Address
  serumProgramId: Address
  serumReferralAuthority: Address
  splTokenFaucet: Address
  splTokenSwapProgramId: Address
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
  price: number
  confidence: number
  exponent: number
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
  symbol: string
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
