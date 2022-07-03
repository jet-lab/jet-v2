import { BN } from "@project-serum/anchor"
import { Number128 } from "./number128"

const POWERS_OF_TEN = [
  new BN(1),
  new BN(10),
  new BN(100),
  new BN(1_000),
  new BN(10_000),
  new BN(100_000),
  new BN(1_000_000),
  new BN(10_000_000),
  new BN(100_000_000),
  new BN(1_000_000_000),
  new BN(10_000_000_000),
  new BN(100_000_000_000),
  new BN(1_000_000_000_000)
]

export class Number192 {
  static readonly PRECISION = 15
  static readonly ONE = new BN(1_000_000_000_000_000)
  static readonly ZERO = Number128.ZERO
  static readonly U64_MAX = Number128.U64_MAX

  private constructor() {}

  static asU64(value: BN, exponent: number) {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = POWERS_OF_TEN[Math.abs(extraPrecision)]

    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = value.mul(precValue)
    } else {
      targetValue = value.div(precValue)
    }

    if (targetValue.gt(this.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(this.ZERO)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  static fromDecimal(value: BN, decimals: number, exponent: number) {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = POWERS_OF_TEN[Math.abs(extraPrecision)]

    let units: BN
    if (extraPrecision < 0) {
      return value.div(precValue)
    } else {
      return value.mul(precValue)
    }
  }
}
