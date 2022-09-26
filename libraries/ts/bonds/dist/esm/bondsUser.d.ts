import { MarginAccount } from "@jet-lab/margin";
import { Address, BN } from "@project-serum/anchor";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { BondMarket, ClaimTicket } from "./bondMarket";
/**  The underlying token */
export declare const AssetKindToken: {
    underlyingToken: {};
};
/** The bond tickets */
export declare const AssetKindTicket: {
    bondTicket: {};
};
/** Bond tickets or their underlying token */
export declare type AssetKind = typeof AssetKindTicket | typeof AssetKindToken;
/** MarginUser account as found on-chain */
export interface MarginUserInfo {
    user: PublicKey;
    bondManager: PublicKey;
    eventAdapter: PublicKey;
    bondTicketsStored: BN;
    underlyingTokenStored: BN;
    outstandingObligations: BN;
    debt: DebtInfo;
    claims: PublicKey;
    nonce: BN;
}
export interface DebtInfo {
    pending: BN;
    committed: BN;
    pastDue: BN;
}
/**
 * A class for user level interaction with the bonds orderbook.
 *
 * Allows placing orders
 */
export declare class BondsUser {
    readonly bondMarket: BondMarket;
    readonly user: Address;
    readonly marginAccount?: MarginAccount;
    readonly borrowerAccount?: MarginUserInfo;
    readonly addresses: {
        ticketAccount: Address;
        marginAccount?: Address;
        borrowerAccount?: Address;
        claims?: Address;
    };
    private constructor();
    get provider(): import("@project-serum/anchor").Provider;
    /**
     *
     * @param market The `BondMarket` this user account belongs to
     * @param user the pubkey of the signer that interacts with the market
     * @returns BondsUser
     */
    static load(market: BondMarket, user: Address): Promise<BondsUser>;
    /**
     *
     * Loads a `BondsUser` given a margin account and `BondMarket`
     *
     * @param market the bond market this BondsUser is derived from
     * @param marginAccount the marginAccount
     * @returns `BondsUser`
     */
    static loadWithMarginAccount(market: BondMarket, marginAccount: MarginAccount): Promise<BondsUser>;
    borrowOrderIx(args: {
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
    exchangeTokensForTicketsIx(amount: BN): Promise<TransactionInstruction>;
    loadClaimTicket(seed: Uint8Array): Promise<ClaimTicket>;
    /**
     *
     * @param payer Payer for pda initialization
     * @param tokenAddress Address to recieve settled tokens
     * @param ticketAddress Address to recieve settled tickets
     * @returns
     */
    initializeMarginUser(payer: Address, tokenAddress?: Address, ticketAddress?: Address): Promise<TransactionInstruction>;
    static deriveMarginUser(bondMarket: BondMarket, marginAccountAddress: Address): Promise<PublicKey>;
    static deriveObligation(borrowerAccount: Address, seed: Uint8Array, programId: Address): Promise<PublicKey>;
}
//# sourceMappingURL=bondsUser.d.ts.map