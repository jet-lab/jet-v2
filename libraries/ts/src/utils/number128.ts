import { BN } from "@project-serum/anchor"

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
  static readonly PRECISION = 10
  static readonly ONE = new BN(10_000_000_000)
  static readonly ZERO = new BN(0)
  static readonly MAX = new BN("340_282_366_920_938_463_463_374_607_431_768_211_455".replaceAll("_", ""))
  static readonly U64_MAX = new BN("18_446_744_073_709_551_615".replaceAll("_", ""))

  private constructor() {}

  static asU64(value: BN, exponent: number) {
    let extraPrecision = Number128.PRECISION + exponent
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

  static fromDecimal(value: BN, exponent: number) {
    let extraPrecision = Number128.PRECISION + exponent
    let precValue = POWERS_OF_TEN[Math.abs(extraPrecision)]

    if (extraPrecision < 0) {
      return value.div(precValue)
    } else {
      return value.mul(precValue)
    }
  }
}
