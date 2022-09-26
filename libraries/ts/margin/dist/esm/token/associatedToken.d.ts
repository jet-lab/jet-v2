/// <reference types="node" />
import { BN, Address, AnchorProvider } from "@project-serum/anchor";
import { Mint, Account } from "@solana/spl-token";
import { Connection, PublicKey, TransactionInstruction, AccountInfo } from "@solana/web3.js";
import { TokenAmount } from "./tokenAmount";
export declare type TokenAddress = Address | TokenFormat;
export declare enum TokenFormat {
    /** The users associated token account will be used, and sol will be unwrapped. */
    unwrappedSol = 0,
    /** The users associated token account will be used, and sol will be wrapped. */
    wrappedSol = 1
}
export declare class AssociatedToken {
    address: PublicKey;
    info: Account | null;
    amount: TokenAmount;
    static readonly NATIVE_DECIMALS = 9;
    exists: boolean;
    /**
     * Get the address for the associated token account
     * @static
     * @param {Address} mint Token mint account
     * @param {Address} owner Owner of the new account
     * @returns {Promise<PublicKey>} Public key of the associated token account
     * @memberof AssociatedToken
     */
    static derive(mint: Address, owner: Address): PublicKey;
    /**
     * TODO:
     * @static
     * @param {Connection} connection
     * @param {Address} mint
     * @param {Address} owner
     * @param {number} decimals
     * @returns {(Promise<AssociatedToken>)}
     * @memberof AssociatedToken
     */
    static load({ connection, mint, owner, decimals }: {
        connection: Connection;
        mint: Address;
        owner: Address;
        decimals: number;
    }): Promise<AssociatedToken>;
    static exists(connection: Connection, mint: Address, owner: Address): Promise<boolean>;
    static existsAux(connection: Connection, mint: Address, owner: Address, address: Address): Promise<boolean>;
    static loadAux(connection: Connection, address: Address, decimals: number): Promise<AssociatedToken>;
    static zero(mint: Address, owner: Address, decimals: number): AssociatedToken;
    static zeroAux(address: Address, decimals: number): AssociatedToken;
    /** Loads multiple token accounts, loads wrapped SOL. Batches by 100 (RPC limit) */
    static loadMultiple({ connection, mints, decimals, owner }: {
        connection: Connection;
        mints: Address[];
        decimals: number | number[];
        owner: Address;
    }): Promise<AssociatedToken[]>;
    /**
     * Loads multiple associated token accounts by owner.
     * If the native mint is provided, loads the native SOL balance of the owner instead.
     * If a mints array is not provided, loads all associated token accounts and the SOL balance of the owner.
     * Batches by 100 (RPC limit) */
    static loadMultipleOrNative({ connection, owner, mints, decimals }: {
        connection: Connection;
        owner: Address;
        mints?: Address[];
        decimals?: number | number[];
    }): Promise<AssociatedToken[]>;
    /**
     * Loads multiple token accounts and their mints by address.
     * Batches by 100 (RPC limit) */
    static loadMultipleAux({ connection, addresses, decimals }: {
        connection: Connection;
        addresses: Address[];
        decimals?: number | number[];
    }): Promise<AssociatedToken[]>;
    /**
     * Fetch all the account info for multiple accounts specified by an array of public keys.
     *
     * @private
     * @static
     * @param {Connection} connection The connection used to fetch.
     * @param {PublicKey[]} publicKeys The accounts to fetch.
     * @param {number} [batchSize=100] The maximum batch size. Default 100, which is an RPC limit.
     * @return {(Promise<(AccountInfo<Buffer> | null)[]>)} An array of accounts returned in the same order as the publicKeys array
     * @memberof AssociatedToken
     */
    private static loadMultipleAccountsInfoBatched;
    /** TODO:
     * Get mint info
     * @static
     * @param {Provider} connection
     * @param {Address} mint
     * @returns {(Promise<Mint | undefined>)}
     * @memberof AssociatedToken
     */
    static loadMint(connection: Connection, mint: Address): Promise<Mint | undefined>;
    /**
     * Creates an instance of AssociatedToken.
     *
     * @param {PublicKey} address
     * @param {Account | null} info
     * @param {TokenAmount} amount
     * @memberof AssociatedToken
     */
    constructor(address: PublicKey, info: Account | null, amount: TokenAmount);
    /**
     * Decode a token account. From @solana/spl-token
     * @param {AccountInfo<Buffer>} inifo
     * @param {PublicKey} address
     * @returns
     */
    static decodeAccount(data: AccountInfo<Buffer> | null, address: Address, decimals: number): AssociatedToken;
    /**
     * Decode a mint account
     * @param {AccountInfo<Buffer>} info
     * @param {PublicKey} address
     * @returns {Mint}
     */
    static decodeMint(info: AccountInfo<Buffer>, address: PublicKey): Mint;
    /**
     * Decode a token account. From @solana/spl-token
     * @param {AccountInfo<Buffer>} inifo
     * @param {PublicKey} address
     * @returns
     */
    static decodeNative(info: AccountInfo<Buffer> | null, address: Address): AssociatedToken;
    /**
     * Returns true when the mint is native and the token account is actually the native wallet
     *
     * @static
     * @param {Address} owner
     * @param {Address} mint
     * @param {Address} tokenAccountOrNative
     * @return {boolean}
     * @memberof AssociatedToken
     */
    static isNative(owner: Address, mint: Address, tokenAccountOrNative: Address): boolean;
    /**
     * If the associated token account does not exist for this mint, add instruction to create the token account.If ATA exists, do nothing.
     * @static
     * @param {TransactionInstruction[]} instructions
     * @param {Provider} provider
     * @param {Address} owner
     * @param {Address} mint
     * @returns {Promise<PublicKey>} returns the public key of the token account
     * @memberof AssociatedToken
     */
    static withCreate(instructions: TransactionInstruction[], provider: AnchorProvider, owner: Address, mint: Address): Promise<PublicKey>;
    /**
     * If the token account does not exist, add instructions to create and initialize the token account. If the account exists do nothing.
     * @static
     * @param {TransactionInstruction[]} instructions
     * @param {Provider} provider
     * @param {Address} owner
     * @param {Address} mint
     * @returns {Promise<PublicKey>} returns the public key of the token account
     * @memberof AssociatedToken
     */
    static withCreateAux(instructions: TransactionInstruction[], provider: AnchorProvider, owner: Address, mint: Address, address: Address): Promise<void>;
    /**
     * Add close associated token account IX
     * @static
     * @param {TransactionInstruction[]} instructions
     * @param {Address} owner
     * @param {Address} mint
     * @param {Address} rentDestination
     * @memberof AssociatedToken
     */
    static withClose(instructions: TransactionInstruction[], owner: Address, mint: Address, rentDestination: Address): void;
    /** Wraps SOL in an associated token account. The account will only be created if it doesn't exist.
     * @param instructions
     * @param provider
     * @param {number} feesBuffer How much tokens should remain unwrapped to pay for fees
     */
    static withWrapNative(instructions: TransactionInstruction[], provider: AnchorProvider, feesBuffer: number): Promise<PublicKey>;
    /**
     * Unwraps all SOL in the associated token account.
     *
     * @param {TransactionInstruction[]} instructions
     * @param {owner} owner
     */
    static withUnwrapNative(instructions: TransactionInstruction[], owner: Address): void;
    /** Add wrap SOL IX
     * @param instructions
     * @param provider
     * @param mint
     * @param feesBuffer How much tokens should remain unwrapped to pay for fees
     */
    static withWrapIfNativeMint(instructions: TransactionInstruction[], provider: AnchorProvider, mint: Address, feesBuffer: number): Promise<PublicKey>;
    /**
     * Unwraps all SOL if the mint is native and the tokenAccount is the owner
     *
     * @param {TransactionInstruction[]} instructions
     * @param {owner} owner
     * @param {mint} mint
     * @param {tokenAccount} tokenAccountOrNative
     */
    static withUnwrapIfNativeMint(instructions: TransactionInstruction[], owner: Address, mint: Address): void;
    /**
     * Create the associated token account. Funds it if native.
     *
     * @static
     * @param {TransactionInstruction[]} instructions
     * @param {AnchorProvider} provider
     * @param {Address} mint
     * @param {number} feesBuffer How much tokens should remain unwrapped to pay for fees
     * @memberof AssociatedToken
     */
    static withCreateOrWrapIfNativeMint(instructions: TransactionInstruction[], provider: AnchorProvider, mint: Address, feesBuffer: number): Promise<PublicKey>;
    /**
     * Create the associated token account as a pre-instruction.
     * Unwraps sol as a post-instruction.
     *
     * @static
     * @param {TransactionInstruction[]} preInstructions
     * @param {TransactionInstruction[]} postInstructions
     * @param {AnchorProvider} provider
     * @param {Address} mint
     * @memberof AssociatedToken
     */
    static withCreateOrUnwrapIfNativeMint(preInstructions: TransactionInstruction[], postInstructions: TransactionInstruction[], provider: AnchorProvider, mint: Address): Promise<PublicKey>;
    static withBeginTransferFromSource({ instructions, provider, mint, feesBuffer, source }: {
        instructions: TransactionInstruction[];
        provider: AnchorProvider;
        mint: Address;
        feesBuffer: number;
        source: Address | TokenFormat;
    }): Promise<PublicKey>;
    static withBeginTransferToDestination({ instructions, provider, mint, destination }: {
        instructions: TransactionInstruction[];
        provider: AnchorProvider;
        mint: Address;
        destination: Address | TokenFormat;
    }): Promise<PublicKey>;
    /** Ends the transfer by unwraps the token if it is the native mint. */
    static withEndTransfer({ instructions, provider, mint, destination }: {
        instructions: TransactionInstruction[];
        provider: AnchorProvider;
        mint: Address;
        destination: Address | TokenFormat;
    }): void;
}
/**
 * Convert number to BN. This never throws for large numbers, unlike the BN constructor.
 * @param {Number} [number]
 * @returns {BN}
 */
export declare function numberToBn(number: number | null | undefined): BN;
/**
 * Convert BN to number. This never throws for large numbers, unlike BN.toNumber().
 * @param {BN} [bn]
 * @returns {number}
 */
export declare function bnToNumber(bn: BN | null | undefined): number;
/**
 * Convert BigInt (SPL Token) to BN. (Anchor)
 * @param {bigint} [bigInt]
 * @returns {BN}
 */
export declare const bigIntToBn: (bigInt: bigint | null | undefined) => BN;
export declare const bnToBigInt: (bn: BN | null | undefined) => bigint;
/** Convert BigInt (SPL Token) to BN. */
export declare const bigIntToNumber: (bigint: bigint | null | undefined) => number;
export declare function numberToBigInt(number: number | null | undefined): bigint;
//# sourceMappingURL=associatedToken.d.ts.map