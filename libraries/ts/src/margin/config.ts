import { Address } from "@project-serum/anchor"
import MARGIN_CONFIG from "./config.json"

export type MarginTokens = "BTC" | "ETH" | "SOL" | "USDC"
export type MarginOracles = "BTC_USD" | "ETH_USD" | "SOL_USD"
export type MarginMarkets = "BTC_USDC" | "ETH_USDC" | "SOL_USDC"

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
  url: string
  tokens: Record<MarginTokens, MarginTokenConfig>
  oracles: Record<MarginOracles, MarginOracleConfig>
  markets: Record<MarginMarkets, MarginMarketConfig>
}

export interface MarginTokenConfig {
  symbol: MarginTokens
  decimals: number
  faucet?: Address
  faucetLimit?: number
  mint: Address
}

export interface MarginOracleConfig {
  symbol: string
  address: Address
}

export interface MarginMarketConfig {
  symbol: string
  market: Address
  baseMint: Address
  baseSymbol: string
  baseDecimals: number
  baseLotSize: number
  quoteMint: Address
  quoteSymbol: string
  quoteDecimals: number
  quoteLotSize: number
  requestQueue: Address
  eventQueue: Address
  bids: Address
  asks: Address
}
