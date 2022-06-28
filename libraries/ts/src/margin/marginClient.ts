import { Program, AnchorProvider } from "@project-serum/anchor"
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata } from ".."
import JET_CONFIG from "../margin/config.json"
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
import { ConfirmedSignatureInfo, Connection, PublicKey, TokenAmount, TransactionResponse } from "@solana/web3.js"

interface TokenMintsList {
  tokenMint: PublicKey
  depositNoteMint: PublicKey
  loanNoteMint: PublicKey
}
type Mints = Record<string, TokenMintsList>

interface TransactionLog {
  blockDate: string
  time: string
  signature: string
  sigIndex: number //signature index that we used to find this transaction
  tradeAction: string
  tradeAmount: TokenAmount
  tokenAbbrev: string
  tokenDecimals: number
  tokenPrice: number
}

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
      return JET_CONFIG[cluster] as MarginConfig
    } else {
      return cluster
    }
  }

  static async getTransactionsFromSignatures(
    provider: AnchorProvider,
    signatures: ConfirmedSignatureInfo[]
  ): Promise<TransactionResponse[]> {
    const responses = await Promise.all(signatures.map(sig => provider.connection.getTransaction(sig.signature)))
    return responses.filter(res => res !== null) as TransactionResponse[]
  }

  static filterTransactions(transactions: TransactionResponse[], config: MarginConfig) {
    return transactions.filter(t => {
      if (t.meta?.logMessages?.some(log => log.includes(config.marginPoolProgramId.toString()))) {
        return true
      } else {
        return false
      }
    })
  }

  static getTransactionData(transaction: TransactionResponse, mints: Mints): TransactionLog | null {
    const instructions = ["repay", "borrow", "deposit", "withdraw"]
    let tradeAction = ""
    instructions.map(element => {
      if (transaction.meta?.logMessages?.some(log => log.includes(element))) {
        tradeAction = element
      }
    })
    if (!tradeAction) return null
    console.log(tradeAction)
    return {
      tradeAction
    } as TransactionLog
  }

  static async getFlightLogs(provider: AnchorProvider, pubKey: PublicKey, mints: Mints, cluster: MarginCluster) {
    const config = MarginClient.getConfig(cluster)
    const signatures = await provider.connection.getSignaturesForAddress(pubKey)
    const transactions = await MarginClient.getTransactionsFromSignatures(provider, signatures)
    const jetTransactions = MarginClient.filterTransactions(transactions, config)
    console.log(jetTransactions)
    return jetTransactions
  }
}
