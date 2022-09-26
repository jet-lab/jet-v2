import { BN } from "@project-serum/anchor";
import { TokenAmount } from "../../";
export declare class PoolTokenChangeKind {
    private kind;
    constructor(kind: PoolTokenChangeKindType);
    static setTo(): PoolTokenChangeKind;
    static shiftBy(): PoolTokenChangeKind;
    asParam(): PoolTokenChangeKindType;
    isShiftBy(): boolean;
}
export declare type PoolTokenChangeKindType = {
    setTo: {};
} | {
    shiftBy: {};
};
/**
 * TODO:
 * @export
 * @class TokenChange
 */
export declare class PoolTokenChange {
    changeKind: PoolTokenChangeKind;
    value: BN;
    /**
     * Creates an instance of Amount.
     * @param {PoolTokenChangeKind} changeKind
     * @param {BN} value
     * @memberof TokenChange
     */
    constructor(changeKind: PoolTokenChangeKind, value: BN);
    /**
     * A `TokenChange` to be used to set the given token value
     * @static
     * @param {TokenAmount | BN | number} value
     * @returns {PoolToken}
     * @memberof TokenChange
     */
    static setTo(value: TokenAmount | BN | number): PoolTokenChange;
    /**
     * A `TokenChange` to be used to shift the given token value
     * @static
     * @param {TokenAmount | BN| number} value
     * @returns {PoolTokenChange}
     * @memberof TokenChange
     */
    static shiftBy(value: TokenAmount | BN | number): PoolTokenChange;
}
//# sourceMappingURL=poolTokenChange.d.ts.map