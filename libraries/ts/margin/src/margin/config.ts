import { Address } from "@project-serum/anchor"
import axios from "axios"

export const MARGIN_CONFIG_URL = "https://storage.googleapis.com/jet-app-config/config.json"
export type MarginCluster = "localnet" | "devnet" | "mainnet-beta" | MarginConfig

export interface MarginConfig {
  bondsProgramId?: string
  metadataAccount: string
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
  airspaces: AirspaceConfig[]
}

export interface AirspaceConfig {
  name: string
  tokens: string[]
  bondMarkets: Record<string, BondMarketConfig>
}

export interface BondMarketConfig {
  symbol: string
  bondManager: Address
  version: number
  duration: number
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

export async function getLatestConfig(cluster: string): Promise<MarginConfig> {
  let response = await axios.get(MARGIN_CONFIG_URL)
  return response.data[cluster]
}
