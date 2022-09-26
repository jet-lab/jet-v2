/// <reference types="node" />
import { Address } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
export declare type AccountSeed = {
    toBytes(): Uint8Array;
} | {
    publicKey: PublicKey;
} | Uint8Array | string | Buffer;
/**
 * Derive a PDA from the argued list of seeds.
 * @param {PublicKey} programId
 * @param {AccountSeed[]} seeds
 * @returns {Promise<PublicKey>}
 * @memberof JetClient
 */
export declare function findDerivedAccount(programId: Address, ...seeds: AccountSeed[]): PublicKey;
//# sourceMappingURL=pda.d.ts.map