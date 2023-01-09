import { BN } from "@project-serum/anchor"
import { bnToNumber, TokenAmount } from "../token"

export class Number128 {
  static readonly PRECISION = 15
  static readonly ONE = new Number128(new BN(1_000_000_000_000_000))
  static readonly ZERO = new Number128(new BN(0))
  static readonly MAX = new Number128(new BN("340282366920938463463374607431768211455"))
  private static readonly U64_MAX = new BN("18446744073709551615")
  private static readonly BPS_EXPONENT = -4

  constructor(private _bn: BN) {}

  toNumber() {
    return bnToNumber(this._bn) / 10 ** Number128.PRECISION
  }

  toTokenAmount(decimals: number) {
    return TokenAmount.lamports(this.toBn(0), decimals)
  }

  /** Removes the fractional component from the number.*/
  toBn(exponent: number): BN {
    let extraPrecision = Number128.PRECISION + exponent
    let precValue = Number128.tenPow(new BN(Math.abs(extraPrecision)))

    if (extraPrecision < 0) {
      return this._bn.mul(precValue)
    } else {
      return this._bn.div(precValue)
    }
  }

  /** Removes the fractional component from the number. Throws if the number is not within the range of a u64. */
  toU64(exponent: number): BN {
    const targetValue = this.toBn(exponent)

    if (targetValue.gt(Number128.U64_MAX)) {
      throw new Error("cannot convert to u64 due to overflow")
    }

    if (targetValue.lt(Number128.ZERO._bn)) {
      throw new Error("cannot convert to u64 because value < 0")
    }

    return targetValue
  }

  static fromDecimal(value: BN, exponent: number): Number128 {
    let extraPrecision = Number128.PRECISION + exponent
    let precValue = Number128.tenPow(new BN(Math.abs(extraPrecision)))

    if (extraPrecision < 0) {
      return new Number128(value.div(precValue))
    } else {
      return new Number128(value.mul(precValue))
    }
  }

  /** Convert from basis points */
  static fromBps(basisPoints: BN): Number128 {
    return this.fromDecimal(basisPoints, this.BPS_EXPONENT)
  }

  static fromBits(bits: number[]): Number128 {
    return new Number128(new BN(bits, "le"))
  }

  static from(bn: BN): Number128 {
    return new Number128(bn.mul(Number128.ONE._bn))
  }

  saturatingAdd(rhs: Number128): Number128 {
    return new Number128(Number128.clamp(this._bn.add(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn))
  }

  saturatingSub(rhs: Number128): Number128 {
    return new Number128(Number128.clamp(this._bn.sub(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn))
  }

  saturatingMul(rhs: Number128): Number128 {
    return new Number128(Number128.clamp(this._bn.mul(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn))
  }

  private static clamp(value: BN, low: BN, high: BN): BN {
    return BN.max(BN.min(value, high), low)
  }

  add(rhs: Number128): Number128 {
    return new Number128(this._bn.add(rhs._bn))
  }

  sub(rhs: Number128): Number128 {
    return new Number128(this._bn.sub(rhs._bn))
  }

  mul(rhs: Number128): Number128 {
    return new Number128(this._bn.mul(rhs._bn).div(Number128.ONE._bn))
  }

  div(rhs: Number128): Number128 {
    return new Number128(this._bn.mul(Number128.ONE._bn).div(rhs._bn))
  }

  public lt(b: Number128): boolean {
    return this._bn.lt(b._bn)
  }

  public gt(b: Number128): boolean {
    return this._bn.gt(b._bn)
  }

  public eq(b: Number128): boolean {
    return this._bn.eq(b._bn)
  }

  public isZero(): boolean {
    return this._bn.isZero()
  }

  static min(a: Number128, b: Number128): Number128 {
    return new Number128(BN.min(a._bn, b._bn))
  }

  static max(a: Number128, b: Number128): Number128 {
    return new Number128(BN.max(a._bn, b._bn))
  }

  static tenPow(exponent: BN) {
    return new BN(10).pow(exponent)
  }
}
