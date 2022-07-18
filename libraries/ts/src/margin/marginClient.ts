import { NATIVE_MINT } from "@solana/spl-token"
import { Program, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata, TokenAmount, PoolAction } from ".."
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

type TxAndSig = {
  details: TransactionResponse
  sig: ConfirmedSignatureInfo
}

export interface AccountTransaction {
  timestamp: number
  blockDate: string
  blockTime: string
  signature: string
  sigIndex: number // Signature index that we used to find this transaction
  tradeAction: PoolAction
  tradeAmount: TokenAmount
  tokenSymbol: string
  tokenName: string
  tokenDecimals: number
  fromAccount?: PublicKey // In the case of a transfer between accounts
  toAccount?: PublicKey // In the case of a transfer between accounts
  status: "error" | "success"
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

  static async getSingleTransaction(provider: AnchorProvider, sig: ConfirmedSignatureInfo): Promise<TxAndSig | null> {
    const details = await provider.connection.getTransaction(sig.signature, { commitment: "confirmed" })
    if (details) {
      return {
        details,
        sig
      }
    } else {
      return null
    }
  }

  static async getTransactionsFromSignatures(
    provider: AnchorProvider,
    signatures: ConfirmedSignatureInfo[]
  ): Promise<TxAndSig[]> {
    const responses = await Promise.all(signatures.map(sig => MarginClient.getSingleTransaction(provider, sig)))
    return responses.filter(res => res !== null) as TxAndSig[]
  }

  static filterTransactions(transactions: TxAndSig[], config: MarginConfig) {
    return transactions.filter(t => {
      if (t.details?.meta?.logMessages?.some(tx => tx.includes(config.marginPoolProgramId.toString()))) {
        return true
      } else {
        return false
      }
    })
  }

  static getTransactionData(
    txAndSig: TxAndSig,
    mints: Mints,
    config: MarginConfig,
    sigIndex: number
  ): AccountTransaction | null {
    const transaction = txAndSig.details
    if (!transaction.meta?.logMessages || !transaction.blockTime) {
      return null
    }

    const instructions = {
      deposit: "Instruction: Deposit",
      withdraw: "Instruction: Withdraw",
      borrow: "Instruction: MarginBorrow",
      "margin repay": "Instruction: MarginRepay",
      repay: "Instruction: Repay"
    }
    let tradeAction = ""

    // Check to see if logMessage string contains relevant instruction
    // If it does, set tradeAction to that element
    const isTradeInstruction = (logLine: string) => {
      for (const action of Object.keys(instructions)) {
        if (logLine.includes(instructions[action])) {
          tradeAction = action
          return true
        }
      }
    }

    if (
      txAndSig.sig.signature ===
      "3qRRjLtXNPtUGXS7tkEtm3pFe13jp11WzbF9FPuA7oV9GEnYJGBFiNwwfCfAtm9YEuszUPBdT7Bg65GFFXG3Hj3t"
    ) {
      console.log(txAndSig)
    }

    // Check each logMessage string for instruction
    // Break after finding the first logMessage for which above is true
    for (let i = 0; i < transaction.meta.logMessages.length; i++) {
      if (isTradeInstruction(transaction.meta?.logMessages[i])) {
        break
      }
    }

    if (!tradeAction || !transaction.meta?.postTokenBalances || !transaction.meta?.preTokenBalances) {
      return null
    }

    const tx: Partial<AccountTransaction> = {
      tradeAction
    } as { tradeAction: PoolAction }
    for (let i = 0; i < transaction.meta.preTokenBalances?.length; i++) {
      const pre = transaction.meta.preTokenBalances[i]
      const matchingPost = transaction.meta.postTokenBalances?.find(
        post => post.mint === pre.mint && post.owner === pre.owner
      )
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
              translateAddress(token.mint).equals(NATIVE_MINT) &&
              (tradeAction === "withdraw" || tradeAction === "borrow") &&
              matchingPost.uiTokenAmount.amount === "0"
            ) {
              break
            }
            const postAmount = new BN(matchingPost.uiTokenAmount.amount)
            const preAmount = new BN(pre.uiTokenAmount.amount)

            tx.tokenSymbol = token.symbol
            tx.tokenName = token.name
            tx.tokenDecimals = token.decimals
            tx.tradeAmount = TokenAmount.lamports(postAmount.sub(preAmount).abs(), token.decimals)

            const dateTime = new Date(transaction.blockTime * 1000)
            tx.timestamp = transaction.blockTime
            tx.blockDate = dateTime.toLocaleDateString()
            tx.blockTime = dateTime.toLocaleTimeString("en-US", { hour12: false })
            tx.sigIndex = sigIndex ? sigIndex : 0
            tx.signature = txAndSig.sig.signature
            tx.status = txAndSig.details.meta?.err ? "error" : "success"
            return tx as AccountTransaction
          }
        }
      }
    }
    return null
  }

  static async getTransactionHistory(
    provider: AnchorProvider,
    pubKey: PublicKey,
    mints: Mints,
    cluster: MarginCluster
  ): Promise<AccountTransaction[]> {
    const config = MarginClient.getConfig(cluster)
    const signatures = await provider.connection.getSignaturesForAddress(pubKey, undefined, "confirmed")
    const transactions = await MarginClient.getTransactionsFromSignatures(provider, signatures)
    const jetTransactions = MarginClient.filterTransactions(transactions, config)
    const parsedTransactions = jetTransactions
      .map((t, idx) => MarginClient.getTransactionData(t, mints, config, idx))
      .filter(tx => !!tx) as AccountTransaction[]
    return parsedTransactions.sort((a, b) => a.sigIndex - b.sigIndex)
  }
}
