import { BN } from "@project-serum/anchor"
import { bnToNumber, TokenAmount } from "../token"

export class Number192 {
  static readonly PRECISION = 15
  static readonly ZERO = new Number192(new BN(0))
  static readonly ONE = new Number192(new BN(1_000_000_000_000_000))
  static readonly MAX = new Number192(new BN(new Array<number>(24).fill(255)))
  private static readonly U64_MAX = new BN("18446744073709551615")
  private static readonly BPS_EXPONENT = -4

  private constructor(private _bn: BN) {}

  toNumber() {
    return bnToNumber(this._bn) / 10 ** Number192.PRECISION
  }

  toTokenAmount(decimals: number) {
    return TokenAmount.lamports(this.toBn(0), decimals)
  }

  /** Removes the fractional component from the number.*/
  toBn(exponent: number): BN {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)))

    if (extraPrecision < 0) {
      return this._bn.mul(precValue)
    } else {
      return this._bn.div(precValue)
    }
  }

  /**
   * Convert this number to fit in a u64
   *
   * The precision of the number in the u64 is based on the
   * exponent provided.
   */
  toU64(exponent: number): BN {
    let targetValue = this.toBn(exponent)

    if (targetValue.gt(Number192.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(Number192.ZERO._bn)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  /**
   * Ceiling value of number, fit in a u64
   *
   * The precision of the number in the u64 is based on the
   * exponent provided.
   *
   * The result is rounded up to the nearest one, based on the
   * target precision.
   */
  toU64Ceil(exponent: number): BN {
    const extraPrecision = Number192.PRECISION + exponent
    const precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)))

    const targetRounded = precValue.sub(new BN(1)).add(this._bn)
    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = targetRounded.mul(precValue)
    } else {
      targetValue = targetRounded.div(precValue)
    }

    if (targetValue.gt(Number192.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(Number192.ZERO._bn)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  /**
   * Convert this number to fit in a u64
   *
   * The precision of the number in the u64 is based on the
   * exponent provided.
   *
   * The result is rounded to the nearest one, based on the
   * target precision.
   */
  toU64Rounded(exponent: number): BN {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)))

    let rounding: BN
    if (extraPrecision > 0) {
      // FIXME: This rounding appears broken https://github.com/jet-lab/program-libraries/blob/074afd601f4ec4ba7dd88ebd6bf2f6c871b29372/math/src/number.rs#L96
      rounding = new BN(1).mul(precValue).div(new BN(2))
    } else {
      rounding = Number192.ZERO._bn
    }

    let targetRounded = rounding.add(this._bn)
    let targetValue: BN
    if (extraPrecision < 0) {
      targetValue = targetRounded.mul(precValue)
    } else {
      targetValue = targetRounded.div(precValue)
    }

    if (targetValue.gt(Number192.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(Number192.ZERO._bn)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  static fromDecimal(value: BN, exponent: number): Number192 {
    let extraPrecision = Number192.PRECISION + exponent
    let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)))

    if (extraPrecision < 0) {
      return new Number192(value.div(precValue))
    } else {
      return new Number192(value.mul(precValue))
    }
  }

  static fromBps(basisPoints: number): Number192 {
    return Number192.fromDecimal(new BN(basisPoints), Number192.BPS_EXPONENT)
  }

  static fromBits(bits: number[]): Number192 {
    return new Number192(new BN(bits, "le"))
  }

  static from(bn: BN): Number192 {
    return new Number192(bn.mul(Number192.ONE._bn))
  }

  saturatingAdd(rhs: Number192): Number192 {
    return new Number192(Number192.clamp(this._bn.add(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn))
  }

  saturatingSub(rhs: Number192): Number192 {
    return new Number192(Number192.clamp(this._bn.sub(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn))
  }

  saturatingMul(rhs: Number192): Number192 {
    return new Number192(Number192.clamp(this._bn.mul(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn))
  }

  private static clamp(value: BN, low: BN, high: BN): BN {
    return BN.max(BN.min(value, high), low)
  }

  add(rhs: Number192): Number192 {
    return new Number192(this._bn.add(rhs._bn))
  }

  sub(rhs: Number192): Number192 {
    return new Number192(this._bn.sub(rhs._bn))
  }

  mul(rhs: Number192): Number192 {
    return new Number192(this._bn.mul(rhs._bn).div(Number192.ONE._bn))
  }

  div(rhs: Number192): Number192 {
    return new Number192(this._bn.mul(Number192.ONE._bn).div(rhs._bn))
  }

  public lt(b: Number192): boolean {
    return this._bn.lt(b._bn)
  }

  public gt(b: Number192): boolean {
    return this._bn.gt(b._bn)
  }

  public eq(b: Number192): boolean {
    return this._bn.eq(b._bn)
  }

  public isZero(): boolean {
    return this._bn.isZero()
  }

  static min(a: Number192, b: Number192): Number192 {
    return new Number192(BN.min(a._bn, b._bn))
  }

  static max(a: Number192, b: Number192): Number192 {
    return new Number192(BN.max(a._bn, b._bn))
  }

  static tenPow(exponent: BN) {
    return new BN(10).pow(exponent)
  }
}
