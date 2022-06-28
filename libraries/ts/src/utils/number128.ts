import { BN } from "@project-serum/anchor"
import { unwatchFile } from "fs"

const PRECISION = 10

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

export class Number128 {
  static readonly ONE = new BN(10_000_000_000)
  static readonly ZERO = new BN(0)
  static readonly MAX = new BN("170141183460469231731687303715884105727")
  static readonly MIN = new BN("-170141183460469231731687303715884105728")

  private constructor() {}

  static asU64(value: BN, exponent: number) {
    let extraPrecision = PRECISION + exponent
    let precValue = POWERS_OF_TEN[Math.abs(extraPrecision)]

    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = value.mul(precValue)
    } else {
      targetValue = value.div(precValue)
    }

    if (targetValue.gt(this.MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(this.ZERO)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  static fromDecimal(value: BN, exponent: number) {
    let extraPrecision = PRECISION + exponent
    let precValue = POWERS_OF_TEN[Math.abs(extraPrecision)]

    if (extraPrecision < 0) {
      return value.div(precValue)
    } else {
      return value.mul(precValue)
    }
  }
}
