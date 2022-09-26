"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.findDerivedAccount = void 0;
const anchor_1 = require("@project-serum/anchor");
const pubkey_1 = require("@project-serum/anchor/dist/cjs/utils/pubkey");
const bs58_1 = __importDefault(require("bs58"));
/**
 * Derive a PDA from the argued list of seeds.
 * @param {PublicKey} programId
 * @param {AccountSeed[]} seeds
 * @returns {Promise<PublicKey>}
 * @memberof JetClient
 */
function findDerivedAccount(programId, ...seeds) {
    const seedBytes = seeds.map(s => {
        if (typeof s == "string") {
            const pubkeyBytes = bs58_1.default.decodeUnsafe(s);
            if (!pubkeyBytes || pubkeyBytes.length !== 32) {
                return Buffer.from(s);
            }
            else {
                return (0, anchor_1.translateAddress)(s).toBytes();
            }
        }
        else if ("publicKey" in s) {
            return s.publicKey.toBytes();
        }
        else if ("toBytes" in s) {
            return s.toBytes();
        }
        else {
            return s;
        }
    });
    const [address] = (0, pubkey_1.findProgramAddressSync)(seedBytes, (0, anchor_1.translateAddress)(programId));
    return address;
}
exports.findDerivedAccount = findDerivedAccount;
//# sourceMappingURL=pda.js.map