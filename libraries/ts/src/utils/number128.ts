import { BN } from "@project-serum/anchor"

export class Number128 {
  static readonly PRECISION = 10
  static readonly ONE = new BN(10_000_000_000)
  static readonly ZERO = new BN(0)
  static readonly MAX = new BN("340282366920938463463374607431768211455")
  static readonly U64_MAX = new BN("18446744073709551615")
  static readonly BPS_EXPONENT = -4
  static readonly POWERS_OF_TEN = [
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

  private constructor() {}

  /** Removes the fractional component from the number.*/
  static asBn(value: BN, exponent: number) {
    let extraPrecision = Number128.PRECISION + exponent
    let precValue = Number128.POWERS_OF_TEN[Math.abs(extraPrecision)]

    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = value.mul(precValue)
    } else {
      targetValue = value.div(precValue)
    }
    return targetValue
  }

  /** Removes the fractional component from the number. Throws if the number is not within the range of a u64. */
  static asU64(value: BN, exponent: number) {
    const targetValue = Number128.asBn(value, exponent)

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
    let precValue = Number128.POWERS_OF_TEN[Math.abs(extraPrecision)]

    if (extraPrecision < 0) {
      return value.div(precValue)
    } else {
      return value.mul(precValue)
    }
  }

  /** Convert from basis points */
  static fromBps(basisPoints: BN) {
    return this.fromDecimal(basisPoints, this.BPS_EXPONENT)
  }
}
