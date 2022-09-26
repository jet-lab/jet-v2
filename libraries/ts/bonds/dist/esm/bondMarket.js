import { BN } from "@project-serum/anchor";
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram, } from "@solana/web3.js";
import { build_order_amount_deprecated } from "../wasm-utils/pkg";
import { Orderbook } from "./orderbook";
import { fetchData, findDerivedAccount } from "./utils";
export const OrderSideBorrow = { borrow: {} };
export const OrderSideLend = { lend: {} };
/**
 * Class for loading and interacting with a BondMarket
 */
export class BondMarket {
    constructor(bondManager, program, info) {
        this.addresses = {
            ...info,
            bondManager,
        };
        this.program = program;
        this.info = info;
    }
    get address() {
        return this.addresses.bondManager;
    }
    get provider() {
        return this.program.provider;
    }
    /**
     * Loads the program state from on chain and returns a `BondMarket` client
     * class for interaction with the market
     *
     * @param program The anchor `JetBonds` program
     * @param address The address of the `bondManager` account
     * @returns
     */
    static async load(program, address) {
        let data = await fetchData(program.provider.connection, address);
        let info = program.coder.accounts.decode("BondManager", data);
        return new BondMarket(new PublicKey(address), program, info);
    }
    async exchangeTokensForTicketsIx(args) {
        let authority = args.userTokenVaultAuthority ?? args.user;
        authority = new PublicKey(authority);
        const tokenVault = args.userTokenVault ??
            (await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, authority));
        const ticketVault = args.userBondTicketVault ??
            (await getAssociatedTokenAddress(this.addresses.bondTicketMint, authority));
        return await this.program.methods
            .exchangeTokens(args.amount)
            .accounts({
            ...this.addresses,
            userBondTicketVault: new PublicKey(ticketVault),
            userUnderlyingTokenVault: new PublicKey(tokenVault),
            userAuthority: args.user,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
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
    async lendOrderIx(args) {
        let params = {
            maxBondTicketQty: args.maxBondTicketQty,
            maxUnderlyingTokenQty: args.maxUnderlyingTokenQty,
            limitPrice: args.limitPrice,
            matchLimit: args.matchLimit ?? new BN(100),
            postOnly: args.postOnly ?? false,
            postAllowed: args.postAllowed ?? true,
            autoStake: args.autoStake ?? true,
        };
        const authority = args.vaultAuthority ?? args.payer;
        const ticketVault = args.ticketVault ??
            (await getAssociatedTokenAddress(this.info.bondTicketMint, new PublicKey(authority)));
        const tokenVault = args.tokenVault ??
            (await getAssociatedTokenAddress(this.info.underlyingTokenMint, new PublicKey(authority)));
        const splitTicket = await findDerivedAccount(["split_ticket", authority, Buffer.from(args.seed)], this.program.programId);
        return await this.program.methods
            .lendOrder(params, Buffer.from(args.seed))
            .accounts({
            ...this.addresses,
            user: authority,
            userTicketVault: ticketVault,
            userTokenVault: tokenVault,
            splitTicket: splitTicket,
            payer: args.payer,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
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
    async sellTicketsOrderIx(args) {
        let params = {
            maxBondTicketQty: args.maxBondTicketQty,
            maxUnderlyingTokenQty: args.maxUnderlyingTokenQty,
            limitPrice: args.limitPrice,
            matchLimit: args.matchLimit ?? new BN(100),
            postOnly: args.postOnly ?? false,
            postAllowed: args.postAllowed ?? true,
            autoStake: args.autoStake ?? true,
        };
        const ticketVault = args.ticketVault ??
            (await getAssociatedTokenAddress(this.info.bondTicketMint, new PublicKey(args.vaultAuthority)));
        const tokenVault = args.tokenVault ??
            (await getAssociatedTokenAddress(this.info.underlyingTokenMint, new PublicKey(args.vaultAuthority)));
        return await this.program.methods
            .sellTicketsOrder(params)
            .accounts({
            ...this.addresses,
            user: args.vaultAuthority,
            userTicketVault: ticketVault,
            userTokenVault: tokenVault,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    async cancelOrderIx(args) {
        const userVault = args.userVault ?? args.side === OrderSideBorrow
            ? await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, new PublicKey(args.user))
            : await getAssociatedTokenAddress(this.addresses.bondTicketMint, new PublicKey(args.user));
        const marketAccount = args.side === OrderSideBorrow
            ? this.addresses.underlyingTokenVault
            : this.addresses.bondTicketMint;
        return await this.program.methods
            .cancelOrder(args.orderId)
            .accounts({
            ...this.addresses,
            user: args.user,
            userVault,
            marketAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    async stakeTicketsIx(args) {
        const claimTicket = await this.deriveClaimTicketKey(args.user, args.seed);
        const bondTicketTokenAccount = args.ticketAccount ??
            (await getAssociatedTokenAddress(this.addresses.bondTicketMint, new PublicKey(args.user)));
        return await this.program.methods
            .stakeBondTickets({
            amount: args.amount,
            ticketSeed: Buffer.from(args.seed),
        })
            .accounts({
            ...this.addresses,
            claimTicket,
            bondTicketTokenAccount,
            ticketHolder: args.user,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        })
            .instruction();
    }
    async deriveTicketAddress(user) {
        return await getAssociatedTokenAddress(this.addresses.bondTicketMint, new PublicKey(user));
    }
    async deriveClaimTicketKey(ticketHolder, seed) {
        return await findDerivedAccount(["claim_ticket", this.address, new PublicKey(ticketHolder), seed], this.program.programId);
    }
    async fetchOrderbook() {
        return await Orderbook.load(this);
    }
}
/**
 * Builds order parameters for a given loan amount and interest rate
 *
 * @param amount amount to be lent or borrowed
 * @param interestRate desired interest rate, in basis points
 */
export const buildOrderAmount = (amount, interestRate) => {
    let orderAmount = build_order_amount_deprecated(BigInt(amount.toString()), BigInt(interestRate.toString()));
    return {
        maxBondTicketQty: new BN(orderAmount.base.toString()),
        maxUnderlyingTokenQty: new BN(orderAmount.quote.toString()),
        limitPrice: new BN(orderAmount.price.toString()),
    };
};
//# sourceMappingURL=bondMarket.js.map