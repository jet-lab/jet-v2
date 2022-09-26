import { Program, BN, Address } from "@project-serum/anchor";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { Orderbook } from "./orderbook";
import { JetBonds } from "./types";
export declare const OrderSideBorrow: {
    borrow: {};
};
export declare const OrderSideLend: {
    lend: {};
};
export declare type OrderSide = typeof OrderSideBorrow | typeof OrderSideLend;
export interface OrderParams {
    maxBondTicketQty: BN;
    maxUnderlyingTokenQty: BN;
    limitPrice: BN;
    matchLimit: BN;
    postOnly: boolean;
    postAllowed: boolean;
    autoStake: boolean;
}
/**
 * The raw struct as found on chain
 */
export interface BondManagerInfo {
    versionTag: BN;
    programAuthority: PublicKey;
    orderbookMarketState: PublicKey;
    eventQueue: PublicKey;
    asks: PublicKey;
    bids: PublicKey;
    underlyingTokenMint: PublicKey;
    underlyingTokenVault: PublicKey;
    bondTicketMint: PublicKey;
    claimsMint: PublicKey;
    collateralMint: PublicKey;
    underlyingOracle: PublicKey;
    ticketOracle: PublicKey;
    seed: number[];
    bump: number[];
    orderbookPaused: boolean;
    ticketsPaused: boolean;
    reserved: number[];
    duration: BN;
    nonce: BN;
}
export interface ClaimTicket {
    owner: PublicKey;
    bondManager: PublicKey;
    maturationTimestamp: BN;
    redeemable: BN;
}
/**
 * Class for loading and interacting with a BondMarket
 */
export declare class BondMarket {
    readonly addresses: {
        bondManager: PublicKey;
        orderbookMarketState: PublicKey;
        eventQueue: PublicKey;
        asks: PublicKey;
        bids: PublicKey;
        underlyingTokenMint: PublicKey;
        underlyingTokenVault: PublicKey;
        bondTicketMint: PublicKey;
        claimsMint: PublicKey;
        collateralMint: PublicKey;
        underlyingOracle: PublicKey;
        ticketOracle: PublicKey;
    };
    readonly info: BondManagerInfo;
    readonly program: Program<JetBonds>;
    private constructor();
    get address(): PublicKey;
    get provider(): import("@project-serum/anchor").Provider;
    /**
     * Loads the program state from on chain and returns a `BondMarket` client
     * class for interaction with the market
     *
     * @param program The anchor `JetBonds` program
     * @param address The address of the `bondManager` account
     * @returns
     */
    static load(program: Program<JetBonds>, address: Address): Promise<BondMarket>;
    exchangeTokensForTicketsIx(args: {
        amount: BN;
        user: Address;
        userTokenVault?: Address;
        userTokenVaultAuthority?: Address;
        userBondTicketVault?: Address;
    }): Promise<TransactionInstruction>;
    /**
     * Creates a `Lend` order instruction. Amount is underlying token lamports. Interest is basis points
     * @param maxBondTicketQty Maximum quantity of bond tickets to order fill
     * @param maxUnderlyingTokenQty Maximum quantity of underlying to lend
     * @param limitPrice limit price for matching orders
     * @param seed BN used to seed a `SplitTicket` intialization. (If auto_stake is enabled)
     * @param payer Payer for PDA initialization. Counted as `vaultAuthority` if not provided
     * @param vaultAuthority Authority over the token vault
     * @param ticketVault Ticket vault to receive matched funds
     * @param tokenVault Token vault containing funds for the order
     * @param matchLimit Maximum number of orders to match with
     * @param postOnly Only succeed if order did not match
     * @param postAllowed Post remaining unfilled as an order on the book
     * @param autoStake Automatically stake any matched bond tickets
     * @returns `TransactionInstruction`
     */
    lendOrderIx(args: {
        maxBondTicketQty: BN;
        maxUnderlyingTokenQty: BN;
        limitPrice: BN;
        seed: Uint8Array;
        payer: Address;
        vaultAuthority?: Address;
        ticketVault?: Address;
        tokenVault?: Address;
        matchLimit?: BN;
        postOnly?: boolean;
        postAllowed?: boolean;
        autoStake?: boolean;
    }): Promise<TransactionInstruction>;
    /**
     * Creates a `Borrow` order instruction. Amount is underlying token lamports. Interest is basis points
     * @param maxBondTicketQty Maximum quantity of bond tickets to order fill
     * @param maxUnderlyingTokenQty Maximum quantity of underlying to lend
     * @param limitPrice limit price for matching orders
     * @param vaultAuthority Authority over the token vault
     * @param ticketVault Ticket vault to receive matched funds
     * @param tokenVault Token vault containing funds for the order
     * @param matchLimit Maximum number of orders to match with
     * @param postOnly Only succeed if order did not match
     * @param postAllowed Post remaining unfilled as an order on the book
     * @param autoStake Automatically stake any matched bond tickets
     * @returns `TransactionInstruction`
     */
    sellTicketsOrderIx(args: {
        maxBondTicketQty: BN;
        maxUnderlyingTokenQty: BN;
        limitPrice: BN;
        vaultAuthority: Address;
        ticketVault?: Address;
        tokenVault?: Address;
        matchLimit?: BN;
        postOnly?: boolean;
        postAllowed?: boolean;
        autoStake?: boolean;
    }): Promise<TransactionInstruction>;
    cancelOrderIx(args: {
        orderId: BN;
        side: OrderSide;
        user: Address;
        userVault?: Address;
    }): Promise<TransactionInstruction>;
    stakeTicketsIx(args: {
        amount: BN;
        seed: Uint8Array;
        user: Address;
        ticketAccount?: Address;
    }): Promise<TransactionInstruction>;
    deriveTicketAddress(user: Address): Promise<PublicKey>;
    deriveClaimTicketKey(ticketHolder: Address, seed: Uint8Array): Promise<PublicKey>;
    fetchOrderbook(): Promise<Orderbook>;
}
/**
 * Builds order parameters for a given loan amount and interest rate
 *
 * @param amount amount to be lent or borrowed
 * @param interestRate desired interest rate, in basis points
 */
export declare const buildOrderAmount: (amount: BN, interestRate: BN) => {
    maxBondTicketQty: BN;
    maxUnderlyingTokenQty: BN;
    limitPrice: BN;
};
//# sourceMappingURL=bondMarket.d.ts.map