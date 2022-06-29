import { Program, AnchorProvider, BN } from "@project-serum/anchor"
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata, TokenAmount } from ".."
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
import { MarginCluster, MarginConfig, MarginTokenConfig } from "./config"
import { ConfirmedSignatureInfo, Connection, PublicKey, TransactionResponse } from "@solana/web3.js"

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
    const responses = await Promise.all(
      signatures.map(sig => provider.connection.getTransaction(sig.signature, { commitment: "confirmed" }))
    )
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

  static getTransactionData(
    transaction: TransactionResponse,
    mints: Mints,
    config: MarginConfig,
    sigIndex: number
  ): TransactionLog | null {
    if (!transaction.meta?.logMessages || !transaction.blockTime) {
      return null
    }

    const instructions = ["repay", "borrow", "deposit", "withdraw"]
    let tradeAction = ""
    for (let i = 0; i < instructions.length; i++) {
      if (transaction.meta?.logMessages?.some(log => log.toLowerCase().includes(instructions[i]))) {
        tradeAction = instructions[i]
        break
      }
    }

    if (!tradeAction || !transaction.meta?.postTokenBalances || !transaction.meta?.preTokenBalances) {
      return null
    }

    const log: Partial<TransactionLog> = {
      tradeAction
    }
    for (let i = 0; i < transaction.meta.preTokenBalances?.length; i++) {
      const pre = transaction.meta.preTokenBalances[i]
      const matchingPost = transaction.meta.postTokenBalances?.find(post => post.mint === pre.mint)
      if (matchingPost && matchingPost.uiTokenAmount.amount !== pre.uiTokenAmount.amount) {
        let token: MarginTokenConfig | null = null

        for (let j = 0; j < Object.entries(mints).length; j++) {
          const tokenAbbrev = Object.entries(mints)[j][0]
          const tokenMints = Object.entries(mints)[j][1]
          if (
            Object.values(tokenMints)
              .map((t: PublicKey) => t.toBase58())
              .includes(matchingPost.mint)
          ) {
            token = config.tokens[tokenAbbrev] as MarginTokenConfig
            if (
              token.symbol === "SOL" &&
              (tradeAction === "withdraw" || tradeAction === "borrow") &&
              matchingPost.uiTokenAmount.amount === "0"
            ) {
              break
            }
            const postAmount = new BN(matchingPost.uiTokenAmount.amount)
            const preAmount = new BN(pre.uiTokenAmount.amount)

            log.tokenAbbrev = token.symbol
            log.tokenDecimals = token.decimals
            log.tradeAmount = new TokenAmount(postAmount.sub(preAmount).abs(), token.decimals)

            const dateTime = new Date(transaction.blockTime * 1000)
            log.blockDate = dateTime.toLocaleDateString()
            log.time = dateTime.toLocaleTimeString("en-US", { hour12: false })
            log.sigIndex = sigIndex ? sigIndex : 0
            return log as TransactionLog
          }
        }
      }
    }
    return null
  }

  static async getFlightLogs(
    provider: AnchorProvider,
    pubKey: PublicKey,
    mints: Mints,
    cluster: MarginCluster
  ): Promise<TransactionLog[]> {
    const config = MarginClient.getConfig(cluster)
    const signatures = await provider.connection.getSignaturesForAddress(pubKey, undefined, "confirmed")
    const transactions = await MarginClient.getTransactionsFromSignatures(provider, signatures)
    const jetTransactions = MarginClient.filterTransactions(transactions, config)
    const parsedTransactions = jetTransactions
      .map((t, idx) => MarginClient.getTransactionData(t, mints, config, idx))
      .filter(tx => !!tx) as TransactionLog[]
    return parsedTransactions.sort((a, b) => a.sigIndex - b.sigIndex)
  }
}
