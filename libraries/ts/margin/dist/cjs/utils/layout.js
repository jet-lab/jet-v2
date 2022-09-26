"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.i64Field = exports.SignedNumberField = exports.number192Field = exports.number128Field = exports.u64 = exports.NumberField = exports.PubkeyField = exports.pubkey = void 0;
const web3_js_1 = require("@solana/web3.js");
const bn_js_1 = __importDefault(require("bn.js"));
const BL = __importStar(require("@solana/buffer-layout"));
/**
 * Layout for a public key
 * @export
 * @param {string} [property]
 * @returns {PubkeyField}
 */
function pubkey(property) {
    return new PubkeyField(property);
}
exports.pubkey = pubkey;
/**
 * Layout for a public key
 * @export
 * @class PubkeyField
 * @extends {BL.Layout}
 */
class PubkeyField extends BL.Layout {
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
        return new web3_js_1.PublicKey(data);
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
exports.PubkeyField = PubkeyField;
/**
 * Layout for an arbitrary sized unsigned int
 * @export
 * @class NumberField
 * @extends {BL.Layout}
 */
class NumberField extends BL.Layout {
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
        return new bn_js_1.default(data, undefined, "le");
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
exports.NumberField = NumberField;
/**
 * Returns an unsigned number field that is 64 bits wide
 * @param property
 * @returns
 */
function u64(property) {
    return new NumberField(8, property);
}
exports.u64 = u64;
/**
 * Returns an unsigned number field that is 128 bts wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
function number128Field(property) {
    return new NumberField(16, property);
}
exports.number128Field = number128Field;
/**
 * Returns an unsigned number field that is 192 bits wide
 * @export
 * @param {string} [property]
 * @returns {NumberField}
 */
function number192Field(property) {
    return new NumberField(24, property);
}
exports.number192Field = number192Field;
/**
 * Layout for an arbitrary sized signed int
 * @export
 * @class SignedNumberField
 * @extends {BL.Layout}
 */
class SignedNumberField extends BL.Layout {
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
        return new bn_js_1.default(data, undefined, "le").fromTwos(this.span * 8);
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
exports.SignedNumberField = SignedNumberField;
/**
 * Returns a signed number field that is 8 bytes wide
 * @export
 * @param {string} [property]
 * @returns {SignedNumberField}
 */
function i64Field(property) {
    return new SignedNumberField(8, property);
}
exports.i64Field = i64Field;
//# sourceMappingURL=layout.js.map