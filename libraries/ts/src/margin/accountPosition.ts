import { PublicKey } from "@solana/web3.js"
import BN from "bn.js"
import { assert } from "console"
import { getTimestamp, Pool, TokenAmount } from ".."
import { Number128 } from "../"
import { AccountPositionInfo, AdapterPositionFlags, PositionKind, PositionKindInfo } from "./state"

export interface PriceInfo {
  value: TokenAmount
  exponent: number
  timestamp: BN
  isValid: number
}

export class AccountPosition {
  /** The raw account position deserialized by anchor */
  info: AccountPositionInfo

  /** The address of the token/mint of the asset */
  token: PublicKey

  /** The address of the account holding the tokens. */
  address: PublicKey

  /** The address of the adapter managing the asset */
  adapter: PublicKey

  /** The current value of this position, stored as a `Number128` with fixed precision. */
  value: BN

  /** The amount of tokens in the account */
  balance: BN

  /** The timestamp of the last balance update */
  balanceTimestamp: BN

  /** The current price/value of each token */
  price: PriceInfo

  /** The kind of balance this position contains */
  kind: PositionKindInfo

  get positionKind() {
    let kind = this.kind
    if ("NoValue" in kind) {
      return PositionKind.NoValue
    }
    if ("Deposit" in kind) {
      return PositionKind.Deposit
    }
    if ("Claim" in kind) {
      return PositionKind.Claim
    }
    throw new Error()
  }

  /** The exponent for the token value */
  exponent: number

  /** A weight on the value of this asset when counting collateral */
  valueModifier: number

  /** The max staleness for the account balance (seconds) */
  maxStaleness: BN

  /** Flags that are set by the adapter */
  flags: AdapterPositionFlags

  reserved: number[]

  constructor({ info, price }: { info: AccountPositionInfo; price?: PriceInfo }) {
    this.info = info
    this.token = info.token
    this.address = info.address
    this.adapter = info.adapter
    this.value = new BN(info.value, "le")
    this.balance = info.balance
    this.balanceTimestamp = info.balanceTimestamp
    this.price = {
      value: price?.value ?? TokenAmount.units(info.price.value, 0, info.price.exponent),
      exponent: price?.exponent ?? info.price.exponent,
      timestamp: price?.timestamp ?? info.price.timestamp,
      isValid: info.price.isValid
    }
    this.kind = info.kind
    this.exponent = info.exponent
    this.valueModifier = info.valueModifier
    this.maxStaleness = info.maxStaleness
    this.flags = new BN(info.flags as number[]).toNumber()
    this.reserved = info.reserved
    this.calculateValue()
  }

  calculateValue() {
    this.value = Number128.fromDecimal(this.balance, this.exponent).mul(
      Number128.fromDecimal(this.price.value.lamports, this.price.exponent)
    )
  }

  collateralValue() {
    assert(this.kind === PositionKind.Deposit)

    return Number128.fromDecimal(new BN(this.valueModifier), -2).mul(this.value)
  }

  requiredCollateralValue() {
    assert(this.kind === PositionKind.Claim)

    let modifier = Number128.fromDecimal(new BN(this.valueModifier), -2)

    if (modifier === Number128.ZERO) {
      console.log(`no leverage configured for claim ${this.token.toBase58()}`)
      return Number128.MAX
    } else {
      return this.value.div(modifier)
    }
  }

  setBalance(balance: BN) {
    this.balance = balance
    this.balanceTimestamp = getTimestamp()
    this.calculateValue()
  }

  setPrice(price: PriceInfo) {
    this.price = price
    this.calculateValue()
  }
}
