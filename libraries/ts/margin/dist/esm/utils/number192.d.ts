import { BN } from "@project-serum/anchor";
import { TokenAmount } from "../token";
export declare class Number192 {
    private _bn;
    static readonly PRECISION = 15;
    static readonly ZERO: Number192;
    static readonly ONE: Number192;
    static readonly MAX: Number192;
    private static readonly U64_MAX;
    private static readonly BPS_EXPONENT;
    private constructor();
    toNumber(): number;
    toTokenAmount(decimals: number): TokenAmount;
    /** Removes the fractional component from the number.*/
    toBn(exponent: number): BN;
    /**
     * Convert this number to fit in a u64
     *
     * The precision of the number in the u64 is based on the
     * exponent provided.
     */
    toU64(exponent: number): BN;
    /**
     * Ceiling value of number, fit in a u64
     *
     * The precision of the number in the u64 is based on the
     * exponent provided.
     *
     * The result is rounded up to the nearest one, based on the
     * target precision.
     */
    toU64Ceil(exponent: number): BN;
    /**
     * Convert this number to fit in a u64
     *
     * The precision of the number in the u64 is based on the
     * exponent provided.
     *
     * The result is rounded to the nearest one, based on the
     * target precision.
     */
    toU64Rounded(exponent: number): BN;
    static fromDecimal(value: BN, exponent: number): Number192;
    static fromBps(basisPoints: number): Number192;
    static fromBits(bits: number[]): Number192;
    static from(bn: BN): Number192;
    saturatingAdd(rhs: Number192): Number192;
    saturatingSub(rhs: Number192): Number192;
    saturatingMul(rhs: Number192): Number192;
    private static clamp;
    add(rhs: Number192): Number192;
    sub(rhs: Number192): Number192;
    mul(rhs: Number192): Number192;
    div(rhs: Number192): Number192;
    lt(b: Number192): boolean;
    gt(b: Number192): boolean;
    eq(b: Number192): boolean;
    isZero(): boolean;
    static min(a: Number192, b: Number192): Number192;
    static max(a: Number192, b: Number192): Number192;
    static tenPow(exponent: BN): BN;
}
//# sourceMappingURL=number192.d.ts.map