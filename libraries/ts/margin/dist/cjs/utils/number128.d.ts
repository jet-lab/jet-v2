import { BN } from "@project-serum/anchor";
import { TokenAmount } from "../token";
export declare class Number128 {
    private _bn;
    static readonly PRECISION = 10;
    static readonly ONE: Number128;
    static readonly ZERO: Number128;
    static readonly MAX: Number128;
    private static readonly U64_MAX;
    private static readonly BPS_EXPONENT;
    constructor(_bn: BN);
    toNumber(): number;
    toTokenAmount(decimals: number): TokenAmount;
    /** Removes the fractional component from the number.*/
    toBn(exponent: number): BN;
    /** Removes the fractional component from the number. Throws if the number is not within the range of a u64. */
    toU64(exponent: number): BN;
    static fromDecimal(value: BN, exponent: number): Number128;
    /** Convert from basis points */
    static fromBps(basisPoints: BN): Number128;
    static fromBits(bits: number[]): Number128;
    static from(bn: BN): Number128;
    saturatingAdd(rhs: Number128): Number128;
    saturatingSub(rhs: Number128): Number128;
    saturatingMul(rhs: Number128): Number128;
    private static clamp;
    add(rhs: Number128): Number128;
    sub(rhs: Number128): Number128;
    mul(rhs: Number128): Number128;
    div(rhs: Number128): Number128;
    lt(b: Number128): boolean;
    gt(b: Number128): boolean;
    eq(b: Number128): boolean;
    isZero(): boolean;
    static min(a: Number128, b: Number128): Number128;
    static max(a: Number128, b: Number128): Number128;
    static tenPow(exponent: BN): BN;
}
//# sourceMappingURL=number128.d.ts.map