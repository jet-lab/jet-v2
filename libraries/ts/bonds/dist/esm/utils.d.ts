/// <reference types="node" />
import { Address } from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
export declare class DerivedAccount {
    address: PublicKey;
    bumpSeed: number;
    constructor(address: PublicKey, bumpSeed: number);
}
interface ToBytes {
    toBytes(): Uint8Array;
}
interface HasPublicKey {
    publicKey: PublicKey;
}
declare type DerivedAccountSeed = HasPublicKey | ToBytes | Uint8Array | string;
export declare function findDerivedAccount(seeds: DerivedAccountSeed[], programId: PublicKey): Promise<PublicKey>;
export declare const fetchData: (connection: Connection, address: Address) => Promise<Buffer>;
export declare const logAccounts: ({ ...accounts }: {
    [x: string]: any;
}) => void;
export {};
//# sourceMappingURL=utils.d.ts.map