/// <reference types="node" />
import { Keypair, PublicKey, TransactionInstruction } from "@solana/web3.js";
import { AnchorProvider, BN } from "@project-serum/anchor";
import { Market as SerumMarket, Orderbook as SerumOrderbook, OpenOrders } from "@project-serum/serum";
import { MarketOptions, Order } from "@project-serum/serum/lib/market";
import { TokenAmount } from "../../token";
import { MarginMarketConfig } from "../config";
import { MarginAccount } from "../marginAccount";
import { MarginPrograms } from "../marginClient";
export declare type SelfTradeBehavior = "decrementTake" | "cancelProvide" | "abortTransaction";
export declare type OrderSide = "sell" | "buy" | "ask" | "bid";
export declare type OrderType = "limit" | "ioc" | "postOnly";
export declare type OrderStatus = "open" | "partialFilled" | "filled" | "cancelled";
export declare class Market {
    programs: MarginPrograms;
    marketConfig: MarginMarketConfig;
    serum: SerumMarket;
    get name(): string;
    get address(): PublicKey;
    get baseMint(): PublicKey;
    get baseDecimals(): number;
    get baseSymbol(): string;
    get quoteMint(): PublicKey;
    get quoteDecimals(): number;
    get quoteSymbol(): string;
    get minOrderSize(): number;
    get tickSize(): number;
    private get baseDecimalMultiplier();
    private get quoteDecimalMultiplier();
    /**
     * Creates a Margin Market
     * @param provider
     * @param programs
     * @param marketConfig
     * @param serum
     */
    constructor(programs: MarginPrograms, marketConfig: MarginMarketConfig, serum: SerumMarket);
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
    static load({ programs, address, options }: {
        programs: MarginPrograms;
        address: PublicKey;
        options?: MarketOptions;
    }): Promise<Market>;
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
    static loadAll(programs: MarginPrograms, options?: MarketOptions): Promise<Record<string, Market>>;
    static encodeOrderSide(side: OrderSide): number;
    static encodeOrderType(type: OrderType): number;
    static encodeSelfTradeBehavior(behavior: SelfTradeBehavior): number;
    placeOrder({ marginAccount, orderSide, orderType, orderPrice, orderSize, selfTradeBehavior, clientOrderId, payer }: {
        marginAccount: MarginAccount;
        orderSide: OrderSide;
        orderType: OrderType;
        orderPrice: number;
        orderSize: TokenAmount;
        selfTradeBehavior?: SelfTradeBehavior;
        clientOrderId?: BN;
        payer?: PublicKey;
    }): Promise<string>;
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
    withPlaceOrder({ instructions, marginAccount, orderSide, orderType, orderPrice, orderSize, selfTradeBehavior, clientOrderId, openOrdersAccount, feeDiscountPubkey, payer }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        orderSide: OrderSide;
        orderType: OrderType;
        orderPrice: number;
        orderSize: TokenAmount;
        selfTradeBehavior: SelfTradeBehavior;
        clientOrderId: BN;
        openOrdersAccount: PublicKey;
        feeDiscountPubkey: PublicKey | undefined;
        payer: PublicKey;
    }): Promise<void>;
    cancelOrder({ marginAccount, order }: {
        marginAccount: MarginAccount;
        order: Order;
    }): Promise<string>;
    /**
     * Get instruction to cancel an order on Serum
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    order: Order
     *  }}
     */
    withCancelOrder({ instructions, marginAccount, order }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        order: Order;
    }): Promise<void>;
    cancelOrderByClientId(marginAccount: MarginAccount, orderId: BN): Promise<string>;
    /**
     * Get instruction to cancel an order on Serum by its clientId
     * @param {{
     *    instructions: TransactionInstruction[]
     *    marginAccount: MarginAccount
     *    orderId: BN
     *  }}
     */
    withCancelOrderByClientId({ instructions, marginAccount, orderId }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        orderId: BN;
    }): Promise<void>;
    settleFunds(marginAccount: MarginAccount, openOrders: OpenOrders, baseWallet: PublicKey, quoteWallet: PublicKey, referrerQuoteWallet?: PublicKey | null): Promise<string>;
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
    withSettleFunds({ instructions, marginAccount, openOrders, baseWallet, quoteWallet, referrerQuoteWallet }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        openOrders: OpenOrders;
        baseWallet: PublicKey;
        quoteWallet: PublicKey;
        referrerQuoteWallet: PublicKey | null;
    }): Promise<Keypair[]>;
    /**
     * Loads the Orderbook
     * @param provider
     */
    loadOrderbook(provider: AnchorProvider): Promise<Orderbook>;
    /**
     * Divide two BN's and return a number
     * @param numerator
     * @param denominator
     */
    divideBnToNumber(numerator: BN, denominator: BN): number;
    /**
     * Price helper functions
     * @param price
     */
    priceLotsToNumber(price: BN): number;
    priceNumberToLots(price: number): BN;
    /**
     * Base size helper functions
     * @param size
     */
    baseSizeToNumber(size: BN): number;
    baseSizeLotsToNumber(size: BN): number;
    baseSizeNumberToLots(size: number): BN;
    /**
     * Quote size helper functions
     * @param size
     */
    quoteSizeToNumber(size: BN): number;
    quoteSizeLotsToNumber(size: BN): number;
    quoteSizeNumberToLots(size: number): BN;
}
export declare class Orderbook {
    market: SerumMarket;
    bids: SerumOrderbook;
    asks: SerumOrderbook;
    /**
     * Creates a Margin Orderbook
     * @param market
     * @param bids
     * @param asks
     */
    constructor(market: SerumMarket, bids: SerumOrderbook, asks: SerumOrderbook);
    /**
     * Load an Orderbook for a given market
     * @param market
     * @param bidsBuffer
     * @param asksBuffer
     */
    static load({ market, bidsBuffer, asksBuffer }: {
        market: SerumMarket;
        bidsBuffer: Buffer;
        asksBuffer: Buffer;
    }): Orderbook;
    /**
     * Return bids for a given depth
     * @param depth
     */
    getBids(depth?: number): number[][];
    /**
     * Return asks for a given depth
     * @param depth
     */
    getAsks(depth?: number): number[][];
}
//# sourceMappingURL=market.d.ts.map