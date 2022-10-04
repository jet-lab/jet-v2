import { PoolAction } from ".."
import { MarginCluster, MarginConfig, MarginTokenConfig, getLatestConfig } from "./config"
import { Connection, PublicKey } from "@solana/web3.js"
import * as FlightLogs from "./flight-log.json"

interface TokenMintsList {
  tokenMint: PublicKey
  depositNoteMint: PublicKey
  loanNoteMint: PublicKey
}
type Mints = Record<string, TokenMintsList>

// We can rename this,
export interface FlightLog {
  id: number
  signature: string
  margin_account: string
  // we can ignore adapter for now?
  token1: string
  token1_amount: number
  token1_price: number
  token2: string
  token2_amount: number
  token2_price: number
  liquidator?: any // TODO: populate when structure for liquidator is set
  activity_type: string
  activity_timestamp: string
  activity_slot: number
  activity_value: number
  // don't need is_primary_activity
}

export interface AccountTransaction {
  timestamp: number | string
  blockDate: string
  blockTime: string
  signature: string
  sigIndex: number // Signature index that we used to find this transaction
  slot: number
  tradeAction: PoolAction
  tradeAmount: number
  tradeAmountInput?: number
  tokenSymbol: string
  tokenName: string
  tokenSymbolInput?: string
  tokenNameInput?: string
  tokenDecimals: number
}

export class MarginClient {
  static async getConfig(cluster: MarginCluster): Promise<MarginConfig> {
    if (typeof cluster === "string") {
      return await getLatestConfig(cluster)
    } else {
      return cluster
    }
  }

  static async setupAccountTx(config: MarginConfig, flightLog: FlightLog): Promise<AccountTransaction | null> {
    const accTransaction: Partial<AccountTransaction> = {}

    switch (flightLog.activity_type) {
      case "Deposit":
        accTransaction.tradeAction = "deposit"
        break
      case "Withdraw":
        accTransaction.tradeAction = "withdraw"
        break
      case "MarginBorrow":
        accTransaction.tradeAction = "borrow"
        break
      case "MarginRepay":
        accTransaction.tradeAction = "repay"
        break
      case "Repay":
        accTransaction.tradeAction = "repay"
        break
      case "MarginSwap":
        accTransaction.tradeAction = "swap"
        break
    }

    const inputTokenConfig = Object.values(config.tokens).find(config =>
      new PublicKey(flightLog.token1).equals(new PublicKey(config.mint))
    )

    const outputTokenConfig = Object.values(config.tokens).find(config =>
      new PublicKey(flightLog.token2).equals(new PublicKey(config.mint))
    )

    let token1 = inputTokenConfig as MarginTokenConfig
    let token2 = outputTokenConfig as MarginTokenConfig

    accTransaction.timestamp = flightLog.activity_timestamp
    accTransaction.blockDate = flightLog.activity_timestamp
    accTransaction.signature = flightLog.signature
    accTransaction.sigIndex = flightLog.id
    accTransaction.slot = flightLog.activity_slot
    accTransaction.tokenNameInput = token1.name
    accTransaction.tokenSymbolInput = token1.symbol
    accTransaction.tradeAmountInput = flightLog.token1_amount
    accTransaction.tokenName = token2.name
    accTransaction.tradeAmount = flightLog.token2_amount
    accTransaction.tokenSymbol = token2.symbol
    accTransaction.tokenDecimals = token1.decimals

    return accTransaction as AccountTransaction
  }

  static async getBlackBoxHistory(
    pubKey: PublicKey,
    cluster: MarginCluster,
    pageSize = 100
  ): Promise<AccountTransaction[]> {
    // URL:
    const flightLogURL = `https://blackbox.jetprotocol.io/margin/accounts/activity/${pubKey}`

    const response = await fetch(flightLogURL)
    const jetTransactions: FlightLog[] = await response.json()
    const config = await MarginClient.getConfig(cluster)

    // let page = 0
    // let processed = 0
    // while (processed < signatures.length) {
    //   const paginatedSignatures = signatures.slice(page * pageSize, (page + 1) * pageSize)
    //   const transactions = await provider.connection.getParsedTransactions(
    //     paginatedSignatures.map(s => s.signature),
    //     "confirmed"
    //   )
    //   const filteredTxs = MarginClient.filterTransactions(transactions, config)
    //   jetTransactions.push(...filteredTxs)
    //   page++
    //   processed += paginatedSignatures.length
    // }

    const parsedTransactions = await Promise.all(
      jetTransactions.map(async (t, idx) => await MarginClient.setupAccountTx(config, t))
    )
    const filteredParsedTransactions = parsedTransactions.filter(tx => !!tx) as AccountTransaction[]
    return filteredParsedTransactions.sort((a, b) => a.slot - b.slot)
  }
}
