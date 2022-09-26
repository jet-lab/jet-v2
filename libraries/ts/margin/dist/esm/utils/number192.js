import { BN } from "@project-serum/anchor";
import { bnToNumber, TokenAmount } from "../token";
export class Number192 {
    constructor(_bn) {
        this._bn = _bn;
    }
    toNumber() {
        return bnToNumber(this._bn) / 10 ** Number192.PRECISION;
    }
    toTokenAmount(decimals) {
        return TokenAmount.lamports(this.toBn(0), decimals);
    }
    /** Removes the fractional component from the number.*/
    toBn(exponent) {
        let extraPrecision = Number192.PRECISION + exponent;
        let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)));
        if (extraPrecision < 0) {
            return this._bn.mul(precValue);
        }
        else {
            return this._bn.div(precValue);
        }
    }
    /**
     * Convert this number to fit in a u64
     *
     * The precision of the number in the u64 is based on the
     * exponent provided.
     */
    toU64(exponent) {
        let targetValue = this.toBn(exponent);
        if (targetValue.gt(Number192.U64_MAX)) {
            throw new Error("cannot convert to u64 due to overflow");
        }
        if (targetValue.lt(Number192.ZERO._bn)) {
            throw new Error("cannot convert to u64 because value < 0");
        }
        return targetValue;
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
    toU64Ceil(exponent) {
        const extraPrecision = Number192.PRECISION + exponent;
        const precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)));
        const targetRounded = precValue.sub(new BN(1)).add(this._bn);
        let targetValue;
        if (extraPrecision < 0) {
            targetValue = targetRounded.mul(precValue);
        }
        else {
            targetValue = targetRounded.div(precValue);
        }
        if (targetValue.gt(Number192.U64_MAX)) {
            throw new Error("cannot convert to u64 due to overflow");
        }
        if (targetValue.lt(Number192.ZERO._bn)) {
            throw new Error("cannot convert to u64 because value < 0");
        }
        return targetValue;
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
    toU64Rounded(exponent) {
        let extraPrecision = Number192.PRECISION + exponent;
        let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)));
        let rounding;
        if (extraPrecision > 0) {
            // FIXME: This rounding appears broken https://github.com/jet-lab/program-libraries/blob/074afd601f4ec4ba7dd88ebd6bf2f6c871b29372/math/src/number.rs#L96
            rounding = new BN(1).mul(precValue).div(new BN(2));
        }
        else {
            rounding = Number192.ZERO._bn;
        }
        let targetRounded = rounding.add(this._bn);
        let targetValue;
        if (extraPrecision < 0) {
            targetValue = targetRounded.mul(precValue);
        }
        else {
            targetValue = targetRounded.div(precValue);
        }
        if (targetValue.gt(Number192.U64_MAX)) {
            throw new Error("cannot convert to u64 due to overflow");
        }
        if (targetValue.lt(Number192.ZERO._bn)) {
            throw new Error("cannot convert to u64 because value < 0");
        }
        return targetValue;
    }
    static fromDecimal(value, exponent) {
        let extraPrecision = Number192.PRECISION + exponent;
        let precValue = Number192.tenPow(new BN(Math.abs(extraPrecision)));
        if (extraPrecision < 0) {
            return new Number192(value.div(precValue));
        }
        else {
            return new Number192(value.mul(precValue));
        }
    }
    static fromBps(basisPoints) {
        return Number192.fromDecimal(new BN(basisPoints), Number192.BPS_EXPONENT);
    }
    static fromBits(bits) {
        return new Number192(new BN(bits, "le"));
    }
    static from(bn) {
        return new Number192(bn.mul(Number192.ONE._bn));
    }
    saturatingAdd(rhs) {
        return new Number192(Number192.clamp(this._bn.add(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn));
    }
    saturatingSub(rhs) {
        return new Number192(Number192.clamp(this._bn.sub(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn));
    }
    saturatingMul(rhs) {
        return new Number192(Number192.clamp(this._bn.mul(rhs._bn), Number192.ZERO._bn, Number192.MAX._bn));
    }
    static clamp(value, low, high) {
        return BN.max(BN.min(value, high), low);
    }
    add(rhs) {
        return new Number192(this._bn.add(rhs._bn));
    }
    sub(rhs) {
        return new Number192(this._bn.sub(rhs._bn));
    }
    mul(rhs) {
        return new Number192(this._bn.mul(rhs._bn).div(Number192.ONE._bn));
    }
    div(rhs) {
        return new Number192(this._bn.mul(Number192.ONE._bn).div(rhs._bn));
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
        return new Number192(BN.min(a._bn, b._bn));
    }
    static max(a, b) {
        return new Number192(BN.max(a._bn, b._bn));
    }
    static tenPow(exponent) {
        return new BN(10).pow(exponent);
    }
}
Number192.PRECISION = 15;
Number192.ZERO = new Number192(new BN(0));
Number192.ONE = new Number192(new BN(1000000000000000));
Number192.MAX = new Number192(new BN(new Array(24).fill(255)));
Number192.U64_MAX = new BN("18446744073709551615");
Number192.BPS_EXPONENT = -4;
//# sourceMappingURL=number192.js.map