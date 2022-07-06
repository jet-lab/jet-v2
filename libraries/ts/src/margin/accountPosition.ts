import { PublicKey } from "@solana/web3.js"
import assert from "assert"
import BN from "bn.js"
import { bnToNumber, getTimestamp } from ".."
import { Number128 } from "../"
import { AccountPositionInfo, AdapterPositionFlags, PositionKind, PositionKindInfo } from "./state"

export interface PriceInfo {
  value: BN
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
  valueRaw: BN

  get value(): number {
    return bnToNumber(Number128.asU64(this.valueRaw, -5)) / 100000
  }

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
  valueModifier: BN

  /** The max staleness for the account balance (seconds) */
  maxStaleness: BN

  /** Flags that are set by the adapter */
  flags: AdapterPositionFlags

  constructor({ info, price }: { info: AccountPositionInfo; price?: PriceInfo }) {
    this.info = info
    this.token = info.token
    this.address = info.address
    this.adapter = info.adapter
    this.valueRaw = new BN(info.value, "le")
    this.balance = info.balance
    this.balanceTimestamp = info.balanceTimestamp
    this.price = {
      value: price?.value ?? info.price.value,
      exponent: price?.exponent ?? info.price.exponent,
      timestamp: price?.timestamp ?? info.price.timestamp,
      isValid: price ? Number(price.isValid) : info.price.isValid
    }
    this.kind = info.kind
    this.exponent = info.exponent
    this.valueModifier = Number128.fromDecimal(new BN(info.valueModifier), -2)
    this.maxStaleness = info.maxStaleness
    this.flags = new BN(info.flags as number[]).toNumber()
    this.calculateValue()
  }

  calculateValue() {
    this.valueRaw = Number128.fromDecimal(this.balance, this.exponent)
      .mul(Number128.fromDecimal(this.price.value, this.price.exponent))
      .div(Number128.ONE)
  }

  collateralValue() {
    assert(this.kind === PositionKind.Deposit)

    return this.valueModifier.mul(this.valueRaw).div(Number128.ONE)
  }

  requiredCollateralValue() {
    assert(this.kind === PositionKind.Claim)

    if (this.valueModifier.eq(Number128.ZERO)) {
      console.log(`no leverage configured for claim ${this.token.toBase58()}`)
      return Number128.MAX
    } else {
      return this.valueRaw.mul(Number128.ONE).div(this.valueModifier)
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
