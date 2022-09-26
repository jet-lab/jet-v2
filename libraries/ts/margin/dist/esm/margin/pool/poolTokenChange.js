import { BN } from "@project-serum/anchor";
export class PoolTokenChangeKind {
    constructor(kind) {
        this.kind = kind;
    }
    static setTo() {
        return new PoolTokenChangeKind({ setTo: {} });
    }
    static shiftBy() {
        return new PoolTokenChangeKind({ shiftBy: {} });
    }
    asParam() {
        return this.kind;
    }
    isShiftBy() {
        return "shiftBy" in this.kind;
    }
}
/**
 * TODO:
 * @export
 * @class TokenChange
 */
export class PoolTokenChange {
    /**
     * Creates an instance of Amount.
     * @param {PoolTokenChangeKind} changeKind
     * @param {BN} value
     * @memberof TokenChange
     */
    constructor(changeKind, value) {
        this.changeKind = changeKind;
        this.value = value;
    }
    /**
     * A `TokenChange` to be used to set the given token value
     * @static
     * @param {TokenAmount | BN | number} value
     * @returns {PoolToken}
     * @memberof TokenChange
     */
    static setTo(value) {
        return new PoolTokenChange(PoolTokenChangeKind.setTo(), typeof value === "object" && "lamports" in value ? value.lamports : new BN(value));
    }
    /**
     * A `TokenChange` to be used to shift the given token value
     * @static
     * @param {TokenAmount | BN| number} value
     * @returns {PoolTokenChange}
     * @memberof TokenChange
     */
    static shiftBy(value) {
        return new PoolTokenChange(PoolTokenChangeKind.shiftBy(), typeof value === "object" && "lamports" in value ? value.lamports : new BN(value));
    }
}
//# sourceMappingURL=poolTokenChange.js.map