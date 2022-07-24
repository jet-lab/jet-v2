import { Address } from "@project-serum/anchor"
import MARGIN_CONFIG from "./config.json"

export type MarginCluster = keyof typeof MARGIN_CONFIG | MarginConfig

export interface MarginConfig {
  controlProgramId: Address
  marginProgramId: Address
  marginPoolProgramId: Address
  marginSerumProgramId: Address
  marginSwapProgramId: Address
  metadataProgramId: Address
  orcaSwapProgramId: Address
  serumProgramId: Address
  faucetProgramId?: Address
  url: string
  tokens: Record<string, MarginTokenConfig>
  markets: Record<string, MarginMarketConfig>
}

export interface MarginTokenConfig {
  symbol: string
  name: string
  decimals: number
  precision: number
  faucet?: Address
  faucetLimit?: number
  mint: Address
}

export interface MarginMarketConfig {
  symbol: string
  market: Address
  baseMint: Address
  baseDecimals: number
  baseVault: Address
  baseSymbol: string
  quoteMint: Address
  quoteDecimals: number
  quoteVault: Address
  quoteSymbol: string
  requestQueue: Address
  eventQueue: Address
  bids: Address
  asks: Address
  quoteDustThreshold: number
  baseLotSize: number
  quoteLotSize: number
  feeRateBps: number
}
