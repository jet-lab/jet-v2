import { BN } from "@project-serum/anchor"
import { Number128 } from "./number128"

export class Number192 {
  static readonly PRECISION = 15
  static readonly ONE = new BN(1_000_000_000_000_000)
  static readonly ZERO = Number128.ZERO
  static readonly U64_MAX = Number128.U64_MAX
  static readonly POWERS_OF_TEN = Number128.POWERS_OF_TEN

  private constructor() {}

  /** Removes the fractional component from the number. */
  static asBn(value: BN, exponent: number) {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.POWERS_OF_TEN[Math.abs(extraPrecision)]

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
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.POWERS_OF_TEN[Math.abs(extraPrecision)]

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

  static asU64Rounded(value: BN, exponent: number) {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.POWERS_OF_TEN[Math.abs(extraPrecision)]

    let rounding: BN
    if (extraPrecision > 0) {
      // FIXME: This rounding appears broken https://github.com/jet-lab/program-libraries/blob/074afd601f4ec4ba7dd88ebd6bf2f6c871b29372/math/src/number.rs#L96
      rounding = new BN(1).mul(precValue).div(new BN(2))
    } else {
      rounding = Number192.ZERO
    }

    let targetRounded = rounding.add(value)
    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = targetRounded.mul(precValue)
    } else {
      targetValue = targetRounded.div(precValue)
    }

    if (targetValue.gt(this.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    return targetValue
  }

  static from(value: BN) {
    return value.mul(Number192.ONE)
  }

  static fromDecimal(value: BN, exponent: number) {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.POWERS_OF_TEN[Math.abs(extraPrecision)]

    if (extraPrecision < 0) {
      return value.div(precValue)
    } else {
      return value.mul(precValue)
    }
  }
}
