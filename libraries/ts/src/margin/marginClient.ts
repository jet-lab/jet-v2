import { getAccount, NATIVE_MINT } from "@solana/spl-token"
import { Program, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata, TokenAmount, PoolAction } from ".."
import {
  JetControl,
  JetControlIdl,
  JetMarginIdl,
  JetMarginPoolIdl,
  JetMarginSerumIdl,
  JetMarginSwapIdl,
  JetMetadataIdl
} from "../types"
import { MarginCluster, MarginConfig, MarginTokenConfig, getLatestConfig } from "./config"
import {
  ConfirmedSignatureInfo,
  Connection,
  ParsedTransactionWithMeta,
  PublicKey,
  TransactionResponse,
  ParsedInstruction,
  ParsedInnerInstruction,
  PartiallyDecodedInstruction
} from "@solana/web3.js"

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
  tradeAmountInput?: TokenAmount
  tokenSymbol: string
  tokenName: string
  tokenSymbolInput?: string
  tokenNameInput?: string
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
  static getPrograms(provider: AnchorProvider, config: MarginConfig): MarginPrograms {
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

  static async getConfig(cluster: MarginCluster): Promise<MarginConfig> {
    if (typeof cluster === "string") {
      return await getLatestConfig(cluster)
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

  static async getTransactionData(
    parsedTx: ParsedTransactionWithMeta,
    mints: Mints,
    config: MarginConfig,
    sigIndex: number,
    provider: AnchorProvider
  ): Promise<AccountTransaction | null> {
    if (!parsedTx.meta?.logMessages || !parsedTx.blockTime) {
      return null
    }

    const instructions = {
      deposit: "Instruction: Deposit",
      withdraw: "Instruction: Withdraw",
      borrow: "Instruction: MarginBorrow",
      "margin repay": "Instruction: MarginRepay",
      repay: "Instruction: Repay",
      swap: "Instruction: MarginSwap"
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

    const setupAccountTx = (token, amount, parsedTx, amountIn?, tokenIn?) => {
      tx.tokenSymbol = token.symbol
      tx.tokenName = token.name
      tx.tokenDecimals = token.decimals
      tx.tradeAmount = TokenAmount.lamports(amount, token.decimals)

      // tokenIn applies if the trade type is a swap
      // For the input token only
      // Default is the output token
      if (tokenIn) {
        tx.tokenSymbolInput = tokenIn.symbol
        tx.tokenNameInput = tokenIn.name
        tx.tradeAmountInput = TokenAmount.lamports(amountIn, tokenIn.decimals)
      }

      const dateTime = new Date(parsedTx.blockTime * 1000)
      tx.timestamp = parsedTx.blockTime
      tx.blockDate = dateTime.toLocaleDateString()
      tx.blockTime = dateTime.toLocaleTimeString("en-US", { hour12: false })
      tx.sigIndex = sigIndex ? sigIndex : 0
      tx.signature = parsedTx.transaction.signatures[0]
      tx.status = parsedTx.meta?.err ? "error" : "success"
      return tx as AccountTransaction
    }

    // Check each logMessage string for instruction
    for (let i = 0; i < parsedTx.meta.logMessages.length; i++) {
      if (isTradeInstruction(parsedTx.meta?.logMessages[i])) {
        // Break after finding the first logMessage for which above is true
        break
      }
    }

    if (!tradeAction || !parsedTx.meta?.postTokenBalances || !parsedTx.meta?.preTokenBalances) {
      return null
    }

    const tx: Partial<AccountTransaction> = {
      tradeAction
    } as { tradeAction: PoolAction }
    for (let i = 0; i < parsedTx.meta.preTokenBalances?.length; i++) {
      const pre = parsedTx.meta.preTokenBalances[i]
      const matchingPost = parsedTx.meta.postTokenBalances?.find(
        post => post.mint === pre.mint && post.owner === pre.owner
      )
      if (matchingPost && matchingPost.uiTokenAmount.amount !== pre.uiTokenAmount.amount) {
        let token: MarginTokenConfig | null = null
        let tokenIn: MarginTokenConfig | null = null

        const ixs = parsedTx.meta.innerInstructions
        const parsedIxnArray: ParsedInstruction[] = []
        let amount = new BN(0)
        let amountIn = new BN(0)

        ixs?.forEach((ix: ParsedInnerInstruction) => {
          ix.instructions.forEach((inst: ParsedInstruction | PartiallyDecodedInstruction) => {
            if ("parsed" in inst) {
              if (inst.parsed && inst.parsed.type === "transfer" && inst?.parsed.info.amount !== "0") {
                parsedIxnArray.push(inst)
                // Default amount is the value of the final parsed instruction
                amount = new BN(inst.parsed.info.amount)
              }
            }
          })
        })
        // If trade action is swap, set up input amount as well
        // Get value of amount in the first parsed instruction
        if (tradeAction === "swap" && parsedIxnArray[0]) {
          amountIn = new BN(parsedIxnArray[0].parsed.info.amount)
        }

        // if we could not find a token transfer, default to token values changes
        if (amount.eq(new BN(0))) {
          const postAmount = new BN(matchingPost.uiTokenAmount.amount)
          const preAmount = new BN(pre.uiTokenAmount.amount)
          amount = postAmount.sub(preAmount).abs()
        }

        for (let j = 0; j < Object.entries(mints).length; j++) {
          const tokenAbbrev = Object.entries(mints)[j][0]
          const tokenMints = Object.entries(mints)[j][1]
          if (
            Object.values(tokenMints)
              .map((t: PublicKey) => t.toBase58())
              .includes(matchingPost.mint)
          ) {
            if (tradeAction === "swap") {
              // If trade action is swap,
              // Set up correct target mint
              const transferIxs: ParsedInstruction[] = []
              ixs?.forEach((ix: ParsedInnerInstruction) => {
                ix.instructions.forEach((inst: ParsedInstruction | PartiallyDecodedInstruction) => {
                  if ("parsed" in inst) {
                    if (inst.parsed && inst.parsed.type === "transfer") {
                      transferIxs.push(inst)
                    }
                  }
                })
              })
              const firstTransferIxSource: string = transferIxs[0].parsed.info.source
              const finalTransferIxSource: string = transferIxs[transferIxs.length - 1].parsed.info.source
              const firstMint = await getAccount(provider.connection, new PublicKey(firstTransferIxSource))
              const sourceAccountMint = await getAccount(provider.connection, new PublicKey(finalTransferIxSource))
              const tokenConfig = Object.values(config.tokens).find(config =>
                sourceAccountMint.mint.equals(new PublicKey(config.mint))
              )
              const firstTokenConfig = Object.values(config.tokens).find(config =>
                firstMint.mint.equals(new PublicKey(config.mint))
              )
              token = tokenConfig as MarginTokenConfig
              tokenIn = config.tokens[tokenAbbrev] as MarginTokenConfig
            } else {
              token = config.tokens[tokenAbbrev] as MarginTokenConfig
            }
            if (
              translateAddress(token.mint).equals(NATIVE_MINT) &&
              (tradeAction === "withdraw" || tradeAction === "borrow") &&
              matchingPost.uiTokenAmount.amount === "0"
            ) {
              break
            }
            return setupAccountTx(token, amount, parsedTx, amountIn, tokenIn)
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
    cluster: MarginCluster,
    pageSize = 100
  ): Promise<AccountTransaction[]> {
    const config = await MarginClient.getConfig(cluster)
    const signatures = await provider.connection.getSignaturesForAddress(pubKey, undefined, "confirmed")
    const jetTransactions: ParsedTransactionWithMeta[] = []
    let page = 0
    let processed = 0
    while (processed < signatures.length) {
      const paginatedSignatures = signatures.slice(page * pageSize, (page + 1) * pageSize)
      const transactions = await provider.connection.getParsedTransactions(
        paginatedSignatures.map(s => s.signature),
        "confirmed"
      )
      const filteredTxs = MarginClient.filterTransactions(transactions, config)
      jetTransactions.push(...filteredTxs)
      page++
      processed += paginatedSignatures.length
    }

    const parsedTransactions = await Promise.all(
      jetTransactions.map(async (t, idx) => await MarginClient.getTransactionData(t, mints, config, idx, provider))
    )
    const filteredParsedTransactions = parsedTransactions.filter(tx => !!tx) as AccountTransaction[]
    return filteredParsedTransactions.sort((a, b) => a.sigIndex - b.sigIndex)
  }
}
