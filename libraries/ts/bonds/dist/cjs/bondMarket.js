"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildOrderAmount = exports.BondMarket = exports.OrderSideLend = exports.OrderSideBorrow = void 0;
const anchor_1 = require("@project-serum/anchor");
const spl_token_1 = require("@solana/spl-token");
const web3_js_1 = require("@solana/web3.js");
const pkg_1 = require("../wasm-utils/pkg");
const orderbook_1 = require("./orderbook");
const utils_1 = require("./utils");
exports.OrderSideBorrow = { borrow: {} };
exports.OrderSideLend = { lend: {} };
/**
 * Class for loading and interacting with a BondMarket
 */
class BondMarket {
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
        let data = await (0, utils_1.fetchData)(program.provider.connection, address);
        let info = program.coder.accounts.decode("BondManager", data);
        return new BondMarket(new web3_js_1.PublicKey(address), program, info);
    }
    async exchangeTokensForTicketsIx(args) {
        let authority = args.userTokenVaultAuthority ?? args.user;
        authority = new web3_js_1.PublicKey(authority);
        const tokenVault = args.userTokenVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.underlyingTokenMint, authority));
        const ticketVault = args.userBondTicketVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.bondTicketMint, authority));
        return await this.program.methods
            .exchangeTokens(args.amount)
            .accounts({
            ...this.addresses,
            userBondTicketVault: new web3_js_1.PublicKey(ticketVault),
            userUnderlyingTokenVault: new web3_js_1.PublicKey(tokenVault),
            userAuthority: args.user,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
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
            matchLimit: args.matchLimit ?? new anchor_1.BN(100),
            postOnly: args.postOnly ?? false,
            postAllowed: args.postAllowed ?? true,
            autoStake: args.autoStake ?? true,
        };
        const authority = args.vaultAuthority ?? args.payer;
        const ticketVault = args.ticketVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.info.bondTicketMint, new web3_js_1.PublicKey(authority)));
        const tokenVault = args.tokenVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.info.underlyingTokenMint, new web3_js_1.PublicKey(authority)));
        const splitTicket = await (0, utils_1.findDerivedAccount)(["split_ticket", authority, Buffer.from(args.seed)], this.program.programId);
        return await this.program.methods
            .lendOrder(params, Buffer.from(args.seed))
            .accounts({
            ...this.addresses,
            user: authority,
            userTicketVault: ticketVault,
            userTokenVault: tokenVault,
            splitTicket: splitTicket,
            payer: args.payer,
            systemProgram: web3_js_1.SystemProgram.programId,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
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
            matchLimit: args.matchLimit ?? new anchor_1.BN(100),
            postOnly: args.postOnly ?? false,
            postAllowed: args.postAllowed ?? true,
            autoStake: args.autoStake ?? true,
        };
        const ticketVault = args.ticketVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.info.bondTicketMint, new web3_js_1.PublicKey(args.vaultAuthority)));
        const tokenVault = args.tokenVault ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.info.underlyingTokenMint, new web3_js_1.PublicKey(args.vaultAuthority)));
        return await this.program.methods
            .sellTicketsOrder(params)
            .accounts({
            ...this.addresses,
            user: args.vaultAuthority,
            userTicketVault: ticketVault,
            userTokenVault: tokenVault,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    async cancelOrderIx(args) {
        const userVault = args.userVault ?? args.side === exports.OrderSideBorrow
            ? await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.underlyingTokenMint, new web3_js_1.PublicKey(args.user))
            : await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.bondTicketMint, new web3_js_1.PublicKey(args.user));
        const marketAccount = args.side === exports.OrderSideBorrow
            ? this.addresses.underlyingTokenVault
            : this.addresses.bondTicketMint;
        return await this.program.methods
            .cancelOrder(args.orderId)
            .accounts({
            ...this.addresses,
            user: args.user,
            userVault,
            marketAccount,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    async stakeTicketsIx(args) {
        const claimTicket = await this.deriveClaimTicketKey(args.user, args.seed);
        const bondTicketTokenAccount = args.ticketAccount ??
            (await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.bondTicketMint, new web3_js_1.PublicKey(args.user)));
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
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            systemProgram: web3_js_1.SystemProgram.programId,
        })
            .instruction();
    }
    async deriveTicketAddress(user) {
        return await (0, spl_token_1.getAssociatedTokenAddress)(this.addresses.bondTicketMint, new web3_js_1.PublicKey(user));
    }
    async deriveClaimTicketKey(ticketHolder, seed) {
        return await (0, utils_1.findDerivedAccount)(["claim_ticket", this.address, new web3_js_1.PublicKey(ticketHolder), seed], this.program.programId);
    }
    async fetchOrderbook() {
        return await orderbook_1.Orderbook.load(this);
    }
}
exports.BondMarket = BondMarket;
/**
 * Builds order parameters for a given loan amount and interest rate
 *
 * @param amount amount to be lent or borrowed
 * @param interestRate desired interest rate, in basis points
 */
const buildOrderAmount = (amount, interestRate) => {
    let orderAmount = (0, pkg_1.build_order_amount_deprecated)(BigInt(amount.toString()), BigInt(interestRate.toString()));
    return {
        maxBondTicketQty: new anchor_1.BN(orderAmount.base.toString()),
        maxUnderlyingTokenQty: new anchor_1.BN(orderAmount.quote.toString()),
        limitPrice: new anchor_1.BN(orderAmount.price.toString()),
    };
};
exports.buildOrderAmount = buildOrderAmount;
//# sourceMappingURL=bondMarket.js.map