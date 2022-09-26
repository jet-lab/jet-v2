import { BN } from "@project-serum/anchor";
import { bnToNumber, TokenAmount } from "../token";
export class Number128 {
    constructor(_bn) {
        this._bn = _bn;
    }
    toNumber() {
        return bnToNumber(this._bn) / 10 ** Number128.PRECISION;
    }
    toTokenAmount(decimals) {
        return TokenAmount.lamports(this.toBn(0), decimals);
    }
    /** Removes the fractional component from the number.*/
    toBn(exponent) {
        let extraPrecision = Number128.PRECISION + exponent;
        let precValue = Number128.tenPow(new BN(Math.abs(extraPrecision)));
        if (extraPrecision < 0) {
            return this._bn.mul(precValue);
        }
        else {
            return this._bn.div(precValue);
        }
    }
    /** Removes the fractional component from the number. Throws if the number is not within the range of a u64. */
    toU64(exponent) {
        const targetValue = this.toBn(exponent);
        if (targetValue.gt(Number128.U64_MAX)) {
            throw new Error("cannot convert to u64 due to overflow");
        }
        if (targetValue.lt(Number128.ZERO._bn)) {
            throw new Error("cannot convert to u64 because value < 0");
        }
        return targetValue;
    }
    static fromDecimal(value, exponent) {
        let extraPrecision = Number128.PRECISION + exponent;
        let precValue = Number128.tenPow(new BN(Math.abs(extraPrecision)));
        if (extraPrecision < 0) {
            return new Number128(value.div(precValue));
        }
        else {
            return new Number128(value.mul(precValue));
        }
    }
    /** Convert from basis points */
    static fromBps(basisPoints) {
        return this.fromDecimal(basisPoints, this.BPS_EXPONENT);
    }
    static fromBits(bits) {
        return new Number128(new BN(bits, "le"));
    }
    static from(bn) {
        return new Number128(bn.mul(Number128.ONE._bn));
    }
    saturatingAdd(rhs) {
        return new Number128(Number128.clamp(this._bn.add(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn));
    }
    saturatingSub(rhs) {
        return new Number128(Number128.clamp(this._bn.sub(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn));
    }
    saturatingMul(rhs) {
        return new Number128(Number128.clamp(this._bn.mul(rhs._bn), Number128.ZERO._bn, Number128.MAX._bn));
    }
    static clamp(value, low, high) {
        return BN.max(BN.min(value, high), low);
    }
    add(rhs) {
        return new Number128(this._bn.add(rhs._bn));
    }
    sub(rhs) {
        return new Number128(this._bn.sub(rhs._bn));
    }
    mul(rhs) {
        return new Number128(this._bn.mul(rhs._bn).div(Number128.ONE._bn));
    }
    div(rhs) {
        return new Number128(this._bn.mul(Number128.ONE._bn).div(rhs._bn));
    }
    lt(b) {
        return this._bn.lt(b._bn);
    }
    gt(b) {
        return this._bn.gt(b._bn);
    }
    eq(b) {
        return this._bn.eq(b._bn);
    }
    isZero() {
        return this._bn.isZero();
    }
    static min(a, b) {
        return new Number128(BN.min(a._bn, b._bn));
    }
    static max(a, b) {
        return new Number128(BN.max(a._bn, b._bn));
    }
    static tenPow(exponent) {
        return new BN(10).pow(exponent);
    }
}
Number128.PRECISION = 10;
Number128.ONE = new Number128(new BN(10000000000));
Number128.ZERO = new Number128(new BN(0));
Number128.MAX = new Number128(new BN("340282366920938463463374607431768211455"));
Number128.U64_MAX = new BN("18446744073709551615");
Number128.BPS_EXPONENT = -4;
//# sourceMappingURL=number128.js.map