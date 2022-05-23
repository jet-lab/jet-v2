import { Program } from "@project-serum/anchor"
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata } from ".."
import JET_CONFIG from "../margin/config.json"
import Provider, { AnchorProvider, Wallet } from "@project-serum/anchor/dist/cjs/provider"
import {
  JetControl,
  JetControlIdl,
  JetMarginIdl,
  JetMarginPoolIdl,
  JetMarginSerumIdl,
  JetMarginSwapIdl,
  JetMetadataIdl
} from "../types"
import { MarginCluster, MarginConfig } from "./config"
import { Connection } from "@solana/web3.js"

export interface MarginPrograms {
  config: MarginConfig
  connection: Connection
  control: Program<JetControl>
  margin: Program<JetMargin>
  marginPool: Program<JetMarginPool>
  marginSerum: Program<JetMarginSerum>
  marginSwap: Program<JetMarginSwap>
  metadata: Program<JetMetadata>
}

export class MarginClient {
  static getPrograms(provider: AnchorProvider, cluster: MarginCluster): MarginPrograms {
    const config = MarginClient.getConfig(cluster)

    const programs: MarginPrograms = {
      config,
      connection: provider.connection,

      control: new Program(JetControlIdl, config.controlProgramId, provider),
      margin: new Program(JetMarginIdl, config.marginProgramId, provider),
      marginPool: new Program(JetMarginPoolIdl, config.marginPoolProgramId, provider),
      marginSerum: new Program(JetMarginSerumIdl, config.marginSerumProgramId, provider),
      marginSwap: new Program(JetMarginSwapIdl, config.marginSwapProgramId, provider),
      metadata: new Program(JetMetadataIdl, config.metadataProgramId, provider)
    }

    return programs
  }

  static getConfig(cluster: MarginCluster): MarginConfig {
    if (typeof cluster === "string") {
      // FIXME: Suble differences between configs as faucet and faucetLimit are sometimes undefined.
      // Remove `as MarginConfig` when there is an interface for the configs
      return JET_CONFIG[cluster] as MarginConfig
    } else {
      return cluster
    }
  }
}
