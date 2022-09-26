"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PoolTokenChange = exports.PoolTokenChangeKind = void 0;
const anchor_1 = require("@project-serum/anchor");
class PoolTokenChangeKind {
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
exports.PoolTokenChangeKind = PoolTokenChangeKind;
/**
 * TODO:
 * @export
 * @class TokenChange
 */
class PoolTokenChange {
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
        return new PoolTokenChange(PoolTokenChangeKind.setTo(), typeof value === "object" && "lamports" in value ? value.lamports : new anchor_1.BN(value));
    }
    /**
     * A `TokenChange` to be used to shift the given token value
     * @static
     * @param {TokenAmount | BN| number} value
     * @returns {PoolTokenChange}
     * @memberof TokenChange
     */
    static shiftBy(value) {
        return new PoolTokenChange(PoolTokenChangeKind.shiftBy(), typeof value === "object" && "lamports" in value ? value.lamports : new anchor_1.BN(value));
    }
}
exports.PoolTokenChange = PoolTokenChange;
//# sourceMappingURL=poolTokenChange.js.map