import { PublicKey } from "@solana/web3.js"
import assert from "assert"
import BN from "bn.js"
import { AccountPositionInfo, AdapterPositionFlags, PositionKind, PositionKindInfo } from "./state"
import { MarginAccount } from "./marginAccount"

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

  get value(): number {
    if (this.price) {
      return this.balance.toNumber() * 10 ** this.exponent * this.price.price
    } else {
      return NaN
    } 
  }

  /** The amount of tokens in the account */
  balance: BN

  /** The timestamp of the last balance update */
  balanceTimestamp: BN

  /** The current price/value of each token */
  price?: StorePriceInfo

  /** The kind of balance this position contains */
  kind: PositionKindInfo

  /** The exponent for the token value */
  exponent: number

  /** A weight on the value of this asset when counting collateral */
  valueModifier: number

  /** The max staleness for the account balance (seconds) */
  maxStaleness: BN

  /** Flags that are set by the adapter */
  flags: AdapterPositionFlags

  constructor({ info, price }: { info: AccountPositionInfo; price?: StorePriceInfo }) {
    this.info = info
    this.token = info.token
    this.address = info.address
    this.adapter = info.adapter
    this.balance = info.balance
    this.balanceTimestamp = info.balanceTimestamp
    this.price = price
    this.kind = info.kind
    this.exponent = info.exponent
    this.valueModifier = info.valueModifier / 100
    this.maxStaleness = info.maxStaleness
    this.flags = info.flags.flags
  }

  collateralValue(): number {
    assert(this.kind === PositionKind.Deposit || this.kind === PositionKind.AdapterCollateral)

    return this.valueModifier * this.value
  }

  requiredCollateralValue(setupLeverageFraction: number = 1): number {
    assert(this.kind === PositionKind.Claim)

    return this.value / (this.valueModifier * setupLeverageFraction)
  }

  requiredSetupCollateralValue(): number {
    return this.requiredCollateralValue(MarginAccount.SETUP_LEVERAGE_FRACTION)
  }
}
