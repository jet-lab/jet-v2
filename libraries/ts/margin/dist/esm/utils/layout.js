import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import * as BL from "@solana/buffer-layout";
/**
 * Layout for a public key
 * @export
 * @param {string} [property]
 * @returns {PubkeyField}
 */
export function pubkey(property) {
    return new PubkeyField(property);
}
/**
 * Layout for a public key
 * @export
 * @class PubkeyField
 * @extends {BL.Layout}
 */
export class PubkeyField extends BL.Layout {
    /**
     * Creates an instance of PubkeyField.
     * @param {string} [property]
     * @memberof PubkeyField
     */
    constructor(property) {
        super(32, property);
    }
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {PublicKey}
     * @memberof PubkeyField
     */
    decode(b, offset) {
        const start = offset ?? 0;
        const data = b.slice(start, start + this.span);
        return new PublicKey(data);
    }
    /**
     * TODO:
     * @param {PublicKey} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof PubkeyField
     */
    encode(src, b, offset) {
        const start = offset ?? 0;
        b.set(src.toBytes(), start);
        return this.span;
    }
}
/**
 * Layout for an arbitrary sized unsigned int
 * @export
 * @class NumberField
 * @extends {BL.Layout}
 */
export class NumberField extends BL.Layout {
    /**
     * Creates an instance of NumberField which decodes to a BN.
     * @param span The number of bytes in the number
     * @param property Field name within in a struct
     */
    constructor(span, property) {
        super(span, property);
    }
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {BN}
     * @memberof NumberField
     */
    decode(b, offset) {
        const start = offset ?? 0;
        const data = b.slice(start, start + this.span);
        return new BN(data, undefined, "le");
    }
    /**
     * TODO:
     * @param {BN} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof NumberField
     */
    encode(src, b, offset) {
        const start = offset ?? 0;
        b.set(src.toArray("le"), start);
        return this.span;
    }
}
/**
 * Returns an unsigned number field that is 64 bits wide
 * @param property
 * @returns
 */
export function u64(property) {
    return new NumberField(8, property);
}
/**
 * Returns an unsigned number field that is 128 bts wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
export function number128Field(property) {
    return new NumberField(16, property);
}
/**
 * Returns an unsigned number field that is 192 bits wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
export function number192Field(property) {
    return new NumberField(24, property);
}
/**
 * Layout for an arbitrary sized signed int
 * @export
 * @class SignedNumberField
 * @extends {BL.Layout}
 */
export class SignedNumberField extends BL.Layout {
    /**
     * Creates an instance of SignedNumberField.
     * @param {number} span
     * @param {string} [property]
     * @memberof SignedNumberField
     */
    constructor(span, property) {
        super(span, property);
    }
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {BN}
     * @memberof SignedNumberField
     */
    decode(b, offset) {
        const start = offset == undefined ? 0 : offset;
        const data = b.slice(start, start + this.span);
        return new BN(data, undefined, "le").fromTwos(this.span * 8);
    }
    /**
     * TODO:
     * @param {BN} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof SignedNumberField
     */
    encode(src, b, offset) {
        const start = offset == undefined ? 0 : offset;
        b.set(src.toTwos(this.span * 8).toArray("le"), start);
        return this.span;
    }
}
/**
 * Returns a signed number field that is 8 bytes wide
 * @export
 * @param {string} [property]
 * @returns {SignedNumberField}
 */
export function i64Field(property) {
    return new SignedNumberField(8, property);
}
//# sourceMappingURL=layout.js.map