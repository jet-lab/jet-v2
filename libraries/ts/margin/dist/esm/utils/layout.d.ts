import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import * as BL from "@solana/buffer-layout";
/**
 * Layout for a public key
 * @export
 * @param {string} [property]
 * @returns {PubkeyField}
 */
export declare function pubkey(property?: string): PubkeyField;
/**
 * Layout for a public key
 * @export
 * @class PubkeyField
 * @extends {BL.Layout}
 */
export declare class PubkeyField extends BL.Layout<PublicKey> {
    /**
     * Creates an instance of PubkeyField.
     * @param {string} [property]
     * @memberof PubkeyField
     */
    constructor(property?: string);
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {PublicKey}
     * @memberof PubkeyField
     */
    decode(b: Uint8Array, offset?: number): PublicKey;
    /**
     * TODO:
     * @param {PublicKey} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof PubkeyField
     */
    encode(src: PublicKey, b: Uint8Array, offset?: number): number;
}
/**
 * Layout for an arbitrary sized unsigned int
 * @export
 * @class NumberField
 * @extends {BL.Layout}
 */
export declare class NumberField extends BL.Layout<BN> {
    /**
     * Creates an instance of NumberField which decodes to a BN.
     * @param span The number of bytes in the number
     * @param property Field name within in a struct
     */
    constructor(span: number, property?: string);
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {BN}
     * @memberof NumberField
     */
    decode(b: Uint8Array, offset?: number): BN;
    /**
     * TODO:
     * @param {BN} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof NumberField
     */
    encode(src: BN, b: Uint8Array, offset?: number): number;
}
/**
 * Returns an unsigned number field that is 64 bits wide
 * @param property
 * @returns
 */
export declare function u64(property?: string): NumberField;
/**
 * Returns an unsigned number field that is 128 bts wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
export declare function number128Field(property?: string): NumberField;
/**
 * Returns an unsigned number field that is 192 bits wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
export declare function number192Field(property?: string): NumberField;
/**
 * Layout for an arbitrary sized signed int
 * @export
 * @class SignedNumberField
 * @extends {BL.Layout}
 */
export declare class SignedNumberField extends BL.Layout<BN> {
    /**
     * Creates an instance of SignedNumberField.
     * @param {number} span
     * @param {string} [property]
     * @memberof SignedNumberField
     */
    constructor(span: number, property?: string);
    /**
     * TODO:
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {BN}
     * @memberof SignedNumberField
     */
    decode(b: Uint8Array, offset?: number): BN;
    /**
     * TODO:
     * @param {BN} src
     * @param {Uint8Array} b
     * @param {number} [offset]
     * @returns {number}
     * @memberof SignedNumberField
     */
    encode(src: BN, b: Uint8Array, offset?: number): number;
}
/**
 * Returns a signed number field that is 8 bytes wide
 * @export
 * @param {string} [property]
 * @returns {SignedNumberField}
 */
export declare function i64Field(property?: string): SignedNumberField;
//# sourceMappingURL=layout.d.ts.map