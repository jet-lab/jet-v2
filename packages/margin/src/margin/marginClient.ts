import { Program, AnchorProvider } from "@project-serum/anchor"
import {
  JetAirspace,
  JetAirspaceIdl,
  JetControl,
  JetControlIdl,
  JetMargin,
  JetMarginIdl,
  JetMarginPool,
  JetMarginPoolIdl,
  JetMarginSerum,
  JetMarginSerumIdl,
  JetMarginSwap,
  JetMarginSwapIdl,
  JetMetadata,
  JetMetadataIdl
} from "../types"
import { MarginCluster, MarginConfig, getLatestConfig } from "./config"
import {
  Connection,
  ParsedTransactionWithMeta,
  PublicKey,
} from "@solana/web3.js"
import axios from "axios"
import { Pool } from "./pool"

export interface MarginPrograms {
  airspace: Program<JetAirspace>
  config: MarginConfig
  connection: Connection
  control: Program<JetControl>
  margin: Program<JetMargin>
  marginPool: Program<JetMarginPool>
  marginSerum: Program<JetMarginSerum>
  marginSwap: Program<JetMarginSwap>
  metadata: Program<JetMetadata>
}

export interface FlightLog {
  id: number
  signature: string
  adapter: string
  adapter_identifier: string
  margin_account: string
  token1: string
  token1_amount: number
  token1_price: number
  // Empty string if there is no token2
  token2: string
  token2_amount: number
  token2_price: number
  liquidator?: string
  activity_type: string
  activity_timestamp: string // TODO: convert to epoch millis
  activity_slot: number
  activity_value: number
  is_primary_activity: boolean

  // Data that is enriched in the UI
  token1_name: string
  token2_name: string
  token1_symbol: string
  token2_symbol: string
  timestamp: number
}

export class MarginClient {
  static getPrograms(provider: AnchorProvider, config: MarginConfig): MarginPrograms {
    const programs: MarginPrograms = {
      config,
      connection: provider.connection,

      airspace: new Program(JetAirspaceIdl, config.airspaceProgramId, provider),
      control: new Program(JetControlIdl, config.controlProgramId, provider),
      margin: new Program(JetMarginIdl, config.marginProgramId, provider),
      marginPool: new Program(JetMarginPoolIdl, config.marginPoolProgramId, provider),
      marginSerum: new Program(JetMarginSerumIdl, config.marginSerumProgramId, provider),
      marginSwap: new Program(JetMarginSwapIdl, config.marginSwapProgramId, provider),
      metadata: new Program(JetMetadataIdl, config.metadataProgramId, provider)
    }

    return programs
  }

  static async getConfig(cluster: MarginCluster): Promise<MarginConfig> {
    if (typeof cluster === "string") {
      return await getLatestConfig(cluster)
    } else {
      return cluster
    }
  }

  static filterTransactions(
    transactions: (ParsedTransactionWithMeta | null)[],
    config: MarginConfig
  ): ParsedTransactionWithMeta[] {
    return transactions.filter(t => {
      if (t?.meta?.logMessages?.some(tx => tx.includes(config.marginPoolProgramId.toString()))) {
        return true
      } else {
        return false
      }
    }) as ParsedTransactionWithMeta[]
  }

  // Blackbox history on mainnet only
  static getBlackboxTx(pools: Record<string, Pool>, flightLog: FlightLog) {
    const token1Pool = Object.values(pools).find(pool => pool.tokenMint.toBase58() == flightLog.token1);
    const token2Pool = Object.values(pools).find(pool => pool.tokenMint.toBase58() == flightLog.token2);

    flightLog.token1_name = token1Pool?.name || "";
    flightLog.token2_name = token2Pool?.name || "";
    flightLog.token1_symbol = token1Pool?.symbol || "";
    flightLog.token2_symbol = token2Pool?.symbol || "";

    // Blackbox is currently stripping timezones, fix that locally until it's updated
    flightLog.activity_timestamp = `${flightLog.activity_timestamp}Z`
    flightLog.timestamp = new Date(flightLog.activity_timestamp).valueOf() / 1000
  }

  static async getBlackBoxHistory(pubKey: PublicKey, cluster: MarginCluster, pools: Record<string, Pool>): Promise<FlightLog[]> {
    const url =
      cluster === "mainnet-beta"
        ? process.env.REACT_APP_DATA_API
        : cluster === "devnet"
          ? process.env.REACT_APP_DEV_DATA_API
          : cluster === "localnet"
            ? process.env.REACT_APP_LOCAL_DATA_API
            : ""
    const flightLogURL = `${url}/margin/accounts/activity/${pubKey}`

    const response = await axios.get(flightLogURL)
    const jetTransactions: FlightLog[] = await response.data

    jetTransactions.map((t) => MarginClient.getBlackboxTx(pools, t))
    const filteredParsedTransactions = jetTransactions.filter(tx => !!tx) as FlightLog[]
    return filteredParsedTransactions.sort((a, b) => a.activity_slot - b.activity_slot)
  }
}
