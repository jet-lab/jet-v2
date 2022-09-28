import { Address } from "@project-serum/anchor"
import axios from "axios"

export const MARGIN_CONFIG_URL = "https://storage.googleapis.com/jet-app-config/config.json"
export type MarginCluster = "localnet" | "devnet" | "mainnet-beta" | MarginConfig

export interface MarginConfig {
  bondsProgramId: Address
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
  bondMarkets: MarginBondMarketConfig[]
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

export interface MarginBondMarketConfig {
  symbol: string
  bondManager: Address
  versionTag: number
  airspace: Address
  orderbookMarketState: Address
  eventQueue: Address
  asks: Address
  bids: Address
  underlyingTokenMint: Address
  underlyingTokenVault: Address
  bondTicketMint: Address
  claimsMint: Address
  collateralMint: Address
  underlyingOracle: Address
  ticketOracle: Address
  seed: number[]
  orderbookPaused: boolean
  ticketsPaused: boolean
  duration: number
}

export async function getLatestConfig(cluster: string): Promise<MarginConfig> {
  let response = await axios.get(MARGIN_CONFIG_URL)
  return (await response.data)[cluster]
}
