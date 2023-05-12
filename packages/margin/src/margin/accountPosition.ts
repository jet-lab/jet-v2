import { PublicKey } from "@solana/web3.js"
import assert from "assert"
import BN from "bn.js"
import { AccountPositionInfo, AdapterPositionFlags, PositionKind, PositionKindInfo } from "./state"
import { Number128, getTimestamp } from "../utils"

export interface PriceInfo {
  value: BN
  exponent: number
  timestamp: BN
  isValid: number
}

export interface StorePriceInfo {
  price: number;
  ema: number;
  confidence: number;
  timestamp: Date;
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

  /** The current value of this position. */
  valueRaw: Number128

  get value(): number {
    return this.valueRaw.toNumber()
  }

  /** The amount of tokens in the account */
  balance: BN

  /** The timestamp of the last balance update */
  balanceTimestamp: BN

  /** The current price/value of each token */
  priceRaw: PriceInfo

  get price(): Number128 {
    return Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent)
  }

  /** The kind of balance this position contains */
  kind: PositionKindInfo

  /** The exponent for the token value */
  exponent: number

  /** A weight on the value of this asset when counting collateral */
  valueModifier: Number128

  /** The max staleness for the account balance (seconds) */
  maxStaleness: BN

  /** Flags that are set by the adapter */
  flags: AdapterPositionFlags

  constructor({ info, price }: { info: AccountPositionInfo; price?: PriceInfo }) {
    this.info = info
    this.token = info.token
    this.address = info.address
    this.adapter = info.adapter
    this.valueRaw = Number128.fromBits(info.value)
    this.balance = info.balance
    this.balanceTimestamp = info.balanceTimestamp
    this.priceRaw = {
      value: price?.value ?? info.price.value,
      exponent: price?.exponent ?? info.price.exponent,
      timestamp: price?.timestamp ?? info.price.timestamp,
      isValid: price ? Number(price.isValid) : info.price.isValid
    }
    this.kind = info.kind
    this.exponent = info.exponent
    this.valueModifier = Number128.fromDecimal(new BN(info.valueModifier), -2)
    this.maxStaleness = info.maxStaleness
    this.flags = info.flags.flags
    this.calculateValue()
  }

  calculateValue(): void {
    this.valueRaw = Number128.fromDecimal(this.balance, this.exponent).mul(
      Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent)
    )
  }

  collateralValue(): Number128 {
    assert(this.kind === PositionKind.Deposit || this.kind === PositionKind.AdapterCollateral)

    return this.valueModifier.mul(this.valueRaw)
  }

  requiredCollateralValue(setupLeverageFraction: Number128 = Number128.ONE): Number128 {
    assert(this.kind === PositionKind.Claim)

    if (this.valueModifier.eq(Number128.ZERO)) {
      console.log(`no leverage configured for claim ${this.token.toBase58()}`)
      return Number128.MAX
    } else {
      return this.valueRaw.div(this.valueModifier).div(setupLeverageFraction)
    }
  }

  setBalance(balance: BN) {
    this.balance = balance
    this.balanceTimestamp = getTimestamp()
    this.calculateValue()
  }

  setPrice(price: PriceInfo) {
    this.priceRaw = price
    this.calculateValue()
  }
}
