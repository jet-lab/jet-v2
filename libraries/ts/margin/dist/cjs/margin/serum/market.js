"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Orderbook = exports.Market = void 0;
const assert_1 = __importDefault(require("assert"));
const web3_js_1 = require("@solana/web3.js");
const anchor_1 = require("@project-serum/anchor");
const serum_1 = require("@project-serum/serum");
const token_instructions_1 = require("@project-serum/serum/lib/token-instructions");
const pool_1 = require("../pool");
class Market {
    /**
     * Creates a Margin Market
     * @param provider
     * @param programs
     * @param marketConfig
     * @param serum
     */
    constructor(programs, marketConfig, serum) {
        this.programs = programs;
        this.marketConfig = marketConfig;
        this.serum = serum;
        (0, assert_1.default)(this.programs.margin.programId);
        (0, assert_1.default)(this.programs.config.serumProgramId);
        if (!serum.decoded.accountFlags.initialized || !serum.decoded.accountFlags.market) {
            throw new Error("Invalid market state");
        }
    }
    get name() {
        return `${this.marketConfig.baseSymbol}/${this.marketConfig.quoteSymbol}`;
    }
    get address() {
        return (0, anchor_1.translateAddress)(this.marketConfig.market);
    }
    get baseMint() {
        return (0, anchor_1.translateAddress)(this.marketConfig.baseMint);
    }
    get baseDecimals() {
        return this.marketConfig.baseDecimals;
    }
    get baseSymbol() {
        return this.marketConfig.baseSymbol;
    }
    get quoteMint() {
        return (0, anchor_1.translateAddress)(this.marketConfig.quoteMint);
    }
    get quoteDecimals() {
        return this.marketConfig.quoteDecimals;
    }
    get quoteSymbol() {
        return this.marketConfig.quoteSymbol;
    }
    get minOrderSize() {
        return this.baseSizeLotsToNumber(new anchor_1.BN(1));
    }
    get tickSize() {
        return this.priceLotsToNumber(new anchor_1.BN(1));
    }
    get baseDecimalMultiplier() {
        return new anchor_1.BN(10).pow(new anchor_1.BN(this.baseDecimals));
    }
    get quoteDecimalMultiplier() {
        return new anchor_1.BN(10).pow(new anchor_1.BN(this.baseDecimals));
    }
    /**
     * Load a Margin Market
     *
     * @param {{
     *     provider: AnchorProvider
     *     programs: MarginPrograms
     *     address: PublicKey
     *     options?: MarketOptions
     *   }}
     * @return {Promise<Market>}
     */
    static async load({ programs, address, options }) {
        const marketAccount = await programs.connection.getAccountInfo(address);
        if (!marketAccount) {
            throw new Error("Market not found");
        }
        if (marketAccount.owner.equals(web3_js_1.SystemProgram.programId) && marketAccount.lamports === 0) {
            throw new Error("Market account not does not exist");
        }
        if (!marketAccount.owner.equals((0, anchor_1.translateAddress)(programs.config.serumProgramId))) {
            throw new Error("Market address not owned by Serum program: " + marketAccount.owner.toBase58());
        }
        const serum = await serum_1.Market.load(programs.connection, address, options, (0, anchor_1.translateAddress)(programs.config.serumProgramId));
        if (!serum.decoded.accountFlags.initialized ||
            !serum.decoded.accountFlags.market ||
            !serum.decoded.ownAddress.equals(address)) {
            throw new Error("Invalid market");
        }
        let marketConfig;
        for (const market of Object.values(programs.config.markets)) {
            if ((0, anchor_1.translateAddress)(market.market).equals(address)) {
                marketConfig = market;
            }
        }
        if (!marketConfig) {
            throw new Error("Unable to match market config");
        }
        return new Market(programs, marketConfig, serum);
    }
    /**
     * Load all Margin Markets
     *
     * @param {{
     *     provider: AnchorProvider
     *     programs: MarginPrograms
     *     options?: MarketOptions
     *   }}
     * @return {Promise<Record<MarginMarkets, Market>>}
     */
    static async loadAll(programs, options) {
        const markets = {};
        for (const marketConfig of Object.values(programs.config.markets)) {
            const market = await this.load({
                programs,
                address: (0, anchor_1.translateAddress)(marketConfig.market),
                options
            });
            markets[market.name] = new Market(programs, marketConfig, market.serum);
        }
        return markets;
    }
    static encodeOrderSide(side) {
        switch (side) {
            case "bid":
            case "buy":
                return 0;
            case "ask":
            case "sell":
                return 1;
        }
    }
    static encodeOrderType(type) {
        switch (type) {
            case "limit":
                return 0;
            case "ioc":
                return 1; // market order
            case "postOnly":
                return 2;
        }
    }
    static encodeSelfTradeBehavior(behavior) {
        switch (behavior) {
            case "decrementTake":
                return 0;
            case "cancelProvide":
                return 1;
            case "abortTransaction":
                return 2;
        }
    }
    async placeOrder({ marginAccount, orderSide, orderType, orderPrice, orderSize, selfTradeBehavior, clientOrderId, payer }) {
        const instructions = [];
        const orderAmount = orderSize.divn(orderPrice);
        const accountPoolPosition = marginAccount.poolPositions[this.baseSymbol];
        // If trading on margin
        if (orderAmount.gt(accountPoolPosition.depositBalance) && marginAccount.pools) {
            const difference = orderAmount.sub(accountPoolPosition.depositBalance);
            const pool = marginAccount.pools[this.baseSymbol];
            if (pool) {
                await pool.marginBorrow({
                    marginAccount,
                    pools: Object.values(marginAccount.pools),
                    change: pool_1.PoolTokenChange.setTo(accountPoolPosition.loanBalance.add(difference))
                });
            }
        }
        // Fetch or create openOrdersAccount
        const openOrdersAccount = (await this.serum.findOpenOrdersAccountsForOwner(marginAccount.provider.connection, this.address))[0];
        let openOrdersAccountPubkey = openOrdersAccount?.publicKey;
        let newOpenOrdersAccount;
        if (!openOrdersAccountPubkey) {
            newOpenOrdersAccount = new web3_js_1.Keypair();
            openOrdersAccountPubkey = newOpenOrdersAccount.publicKey;
            instructions.push(await serum_1.OpenOrders.makeCreateAccountTransaction(marginAccount.provider.connection, this.address, marginAccount.address, newOpenOrdersAccount.publicKey, this.serum.programId));
        }
        // Attempt to find MSRM fee account
        let feeDiscountPubkey;
        try {
            feeDiscountPubkey =
                (await this.serum.findBestFeeDiscountKey(marginAccount.provider.connection, marginAccount.address)).pubkey ??
                    undefined;
        }
        catch (err) {
            if (!err.message || !err.message.includes("could not find mint")) {
                console.error(err);
            }
        }
        await this.withPlaceOrder({
            instructions,
            marginAccount,
            orderSide,
            orderType,
            orderPrice,
            orderSize,
            selfTradeBehavior: selfTradeBehavior ?? "decrementTake",
            clientOrderId: clientOrderId ?? new anchor_1.BN(Date.now()),
            openOrdersAccount: openOrdersAccountPubkey,
            feeDiscountPubkey: feeDiscountPubkey,
            payer: payer ?? marginAccount.address
        });
        return await marginAccount.sendAndConfirm(instructions, newOpenOrdersAccount ? [newOpenOrdersAccount] : undefined);
    }
    /** Get instruction to submit an order to Serum
     *
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    orderSide: orderSide
     *    orderType: orderType
     *    orderPrice: number
     *    orderSize: TokenAmount
     *    selfTradeBehavior: selfTradeBehavior
     *    clientOrderId: BN
     *    payer: PublicKey
     *  }}
     */
    async withPlaceOrder({ instructions, marginAccount, orderSide, orderType, orderPrice, orderSize, selfTradeBehavior, clientOrderId, openOrdersAccount, feeDiscountPubkey, payer }) {
        const limitPrice = new anchor_1.BN(Math.round((orderPrice * Math.pow(10, this.quoteDecimals) * this.marketConfig.baseLotSize) /
            (Math.pow(10, this.baseDecimals) * this.marketConfig.quoteLotSize)));
        const maxCoinQty = orderSize.lamports;
        const baseSizeLots = maxCoinQty.toNumber() / this.marketConfig.baseLotSize;
        const maxNativePcQtyIncludingFees = new anchor_1.BN(this.marketConfig.quoteLotSize * baseSizeLots).mul(limitPrice);
        const ix = await this.programs.marginSerum.methods
            .newOrderV3(Market.encodeOrderSide(orderSide), limitPrice, maxCoinQty, maxNativePcQtyIncludingFees, Market.encodeSelfTradeBehavior(selfTradeBehavior), Market.encodeOrderType(orderType), clientOrderId, 65535)
            .accounts({
            marginAccount: marginAccount.address,
            market: this.address,
            openOrdersAccount,
            requestQueue: this.marketConfig.requestQueue,
            eventQueue: this.marketConfig.eventQueue,
            bids: this.marketConfig.bids,
            asks: this.marketConfig.asks,
            payer,
            baseVault: this.marketConfig.baseVault,
            quoteVault: this.marketConfig.quoteVault,
            splTokenProgramId: token_instructions_1.TOKEN_PROGRAM_ID,
            rentSysvarId: web3_js_1.SYSVAR_RENT_PUBKEY,
            serumProgramId: this.programs.config.serumProgramId
        })
            .remainingAccounts(feeDiscountPubkey ? [{ pubkey: feeDiscountPubkey, isSigner: false, isWritable: true }] : [])
            .instruction();
        instructions.push(ix);
    }
    async cancelOrder({ marginAccount, order }) {
        const instructions = [];
        await this.withCancelOrder({ instructions, marginAccount, order });
        return await marginAccount.sendAndConfirm(instructions);
    }
    /**
     * Get instruction to cancel an order on Serum
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    order: Order
     *  }}
     */
    async withCancelOrder({ instructions, marginAccount, order }) {
        const ix = await this.programs.marginSerum.methods
            .cancelOrderV2(Market.encodeOrderSide(order.side), order.orderId)
            .accounts({
            marginAccount: marginAccount.address,
            market: this.address,
            openOrdersAccount: order.openOrdersAddress,
            marketBids: this.marketConfig.bids,
            marketAsks: this.marketConfig.asks,
            eventQueue: this.marketConfig.eventQueue,
            serumProgramId: this.programs.config.serumProgramId
        })
            .instruction();
        instructions.push(ix);
    }
    async cancelOrderByClientId(marginAccount, orderId) {
        const instructions = [];
        await this.withCancelOrderByClientId({ instructions, marginAccount, orderId });
        return await marginAccount.sendAndConfirm(instructions);
    }
    /**
     * Get instruction to cancel an order on Serum by its clientId
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    orderId: BN
     *  }}
     */
    async withCancelOrderByClientId({ instructions, marginAccount, orderId }) {
        const ix = await this.programs.marginSerum.methods
            .cancelOrderByClientIdV2(orderId)
            .accounts({
            marginAccount: marginAccount.address,
            market: this.address,
            marketBids: this.marketConfig.bids,
            marketAsks: this.marketConfig.asks,
            eventQueue: this.marketConfig.eventQueue,
            serumProgramId: this.programs.config.serumProgramId
        })
            .instruction();
        instructions.push(ix);
    }
    async settleFunds(marginAccount, openOrders, baseWallet, quoteWallet, referrerQuoteWallet = null) {
        if (!openOrders.owner.equals(marginAccount.address)) {
            throw new Error("Invalid open orders account");
        }
        const supportsReferralFees = (0, serum_1.getLayoutVersion)((0, anchor_1.translateAddress)(this.programs.config.serumProgramId)) > 1;
        if (referrerQuoteWallet && !supportsReferralFees) {
            throw new Error("This program ID does not support referrerQuoteWallet");
        }
        const instructions = [];
        const signers = await this.withSettleFunds({
            instructions,
            marginAccount,
            openOrders,
            baseWallet,
            quoteWallet,
            referrerQuoteWallet
        });
        return await marginAccount.sendAndConfirm(instructions, signers);
    }
    /**
     * Get instruction to settle funds
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    openOrders: OpenOrders
     *    baseWallet: PublicKey
     *    quoteWallet: PublicKey
     *    referrerQuoteWallet: PublicKey | null
     *  }}
     */
    async withSettleFunds({ instructions, marginAccount, openOrders, baseWallet, quoteWallet, referrerQuoteWallet = null }) {
        const vaultSigner = await web3_js_1.PublicKey.createProgramAddress([this.address.toBuffer(), this.serum.decoded.vaultSignerNonce.toArrayLike(Buffer, "le", 8)], (0, anchor_1.translateAddress)(this.programs.config.serumProgramId));
        const signers = [];
        let wrappedSolAccount = null;
        if ((this.baseMint.equals(token_instructions_1.WRAPPED_SOL_MINT) && baseWallet.equals(openOrders.owner)) ||
            (this.quoteMint.equals(token_instructions_1.WRAPPED_SOL_MINT) && quoteWallet.equals(openOrders.owner))) {
            wrappedSolAccount = new web3_js_1.Keypair();
            instructions.push(web3_js_1.SystemProgram.createAccount({
                fromPubkey: openOrders.owner,
                newAccountPubkey: wrappedSolAccount.publicKey,
                lamports: await marginAccount.provider.connection.getMinimumBalanceForRentExemption(165),
                space: 165,
                programId: token_instructions_1.TOKEN_PROGRAM_ID
            }));
            instructions.push((0, token_instructions_1.initializeAccount)({
                account: wrappedSolAccount.publicKey,
                mint: token_instructions_1.WRAPPED_SOL_MINT,
                owner: openOrders.owner
            }));
            signers.push(wrappedSolAccount);
        }
        const ix = await this.programs.marginSerum.methods
            .settleFunds()
            .accounts({
            marginAccount: marginAccount.address,
            market: this.address,
            splTokenProgramId: token_instructions_1.TOKEN_PROGRAM_ID,
            openOrdersAccount: openOrders.address,
            coinVault: this.marketConfig.baseVault,
            pcVault: this.marketConfig.quoteVault,
            coinWallet: baseWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : baseWallet,
            pcWallet: quoteWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : quoteWallet,
            vaultSigner,
            serumProgramId: this.programs.config.serumProgramId
        })
            .remainingAccounts(referrerQuoteWallet ? [{ pubkey: referrerQuoteWallet, isSigner: false, isWritable: true }] : [])
            .instruction();
        instructions.push(ix);
        if (wrappedSolAccount) {
            instructions.push((0, token_instructions_1.closeAccount)({
                source: wrappedSolAccount.publicKey,
                destination: openOrders.owner,
                owner: openOrders.owner
            }));
        }
        return signers;
    }
    /**
     * Loads the Orderbook
     * @param provider
     */
    async loadOrderbook(provider) {
        const bidsBuffer = (await provider.connection.getAccountInfo((0, anchor_1.translateAddress)(this.marketConfig.bids)))?.data;
        const asksBuffer = (await provider.connection.getAccountInfo((0, anchor_1.translateAddress)(this.marketConfig.asks)))?.data;
        if (!bidsBuffer || !asksBuffer) {
            throw new Error("Orderbook sides not found");
        }
        const bids = serum_1.Orderbook.decode(this.serum, bidsBuffer);
        const asks = serum_1.Orderbook.decode(this.serum, asksBuffer);
        return new Orderbook(this.serum, bids, asks);
    }
    /**
     * Divide two BN's and return a number
     * @param numerator
     * @param denominator
     */
    divideBnToNumber(numerator, denominator) {
        const quotient = numerator.div(denominator).toNumber();
        const rem = numerator.umod(denominator);
        const gcd = rem.gcd(denominator);
        return quotient + rem.div(gcd).toNumber() / denominator.div(gcd).toNumber();
    }
    /**
     * Price helper functions
     * @param price
     */
    priceLotsToNumber(price) {
        return this.divideBnToNumber(price.mul(this.serum.decoded.quoteLotSize).mul(this.baseDecimalMultiplier), this.serum.decoded.baseLotSize.mul(this.quoteDecimalMultiplier));
    }
    priceNumberToLots(price) {
        return new anchor_1.BN(Math.round((price * Math.pow(10, this.quoteDecimals) * this.serum.decoded.baseLotSize.toNumber()) /
            (Math.pow(10, this.baseDecimals) * this.serum.decoded.quoteLotSize.toNumber())));
    }
    /**
     * Base size helper functions
     * @param size
     */
    baseSizeToNumber(size) {
        return this.divideBnToNumber(size, this.baseDecimalMultiplier);
    }
    baseSizeLotsToNumber(size) {
        return this.divideBnToNumber(size.mul(this.serum.decoded.baseLotSize), this.baseDecimalMultiplier);
    }
    baseSizeNumberToLots(size) {
        const native = new anchor_1.BN(Math.round(size * Math.pow(10, this.baseDecimals)));
        // rounds down to the nearest lot size
        return native.div(this.serum.decoded.baseLotSize);
    }
    /**
     * Quote size helper functions
     * @param size
     */
    quoteSizeToNumber(size) {
        return this.divideBnToNumber(size, this.quoteDecimalMultiplier);
    }
    quoteSizeLotsToNumber(size) {
        return this.divideBnToNumber(size.mul(this.serum.decoded.quoteLotSize), this.quoteDecimalMultiplier);
    }
    quoteSizeNumberToLots(size) {
        const native = new anchor_1.BN(Math.round(size * Math.pow(10, this.quoteDecimals)));
        // rounds down to the nearest lot size
        return native.div(this.serum.decoded.quoteLotSize);
    }
}
exports.Market = Market;
class Orderbook {
    /**
     * Creates a Margin Orderbook
     * @param market
     * @param bids
     * @param asks
     */
    constructor(market, bids, asks) {
        this.market = market;
        this.bids = bids;
        this.asks = asks;
    }
    /**
     * Load an Orderbook for a given market
     * @param market
     * @param bidsBuffer
     * @param asksBuffer
     */
    static load({ market, bidsBuffer, asksBuffer }) {
        const bids = serum_1.Orderbook.decode(market, bidsBuffer);
        const asks = serum_1.Orderbook.decode(market, asksBuffer);
        return new Orderbook(market, bids, asks);
    }
    /**
     * Return bids for a given depth
     * @param depth
     */
    getBids(depth = 8) {
        return this.bids.getL2(depth).map(([price, size]) => [price, size]);
    }
    /**
     * Return asks for a given depth
     * @param depth
     */
    getAsks(depth = 8) {
        return this.asks.getL2(depth).map(([price, size]) => [price, size]);
    }
}
exports.Orderbook = Orderbook;
//# sourceMappingURL=market.js.map