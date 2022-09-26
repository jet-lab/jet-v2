import { Address, BN } from "@project-serum/anchor";
import { PriceData } from "@pythnetwork/client";
import { Mint } from "@solana/spl-token";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { AssociatedToken, TokenAddress } from "../../token";
import { TokenAmount } from "../../token/tokenAmount";
import { MarginAccount } from "../marginAccount";
import { MarginPrograms } from "../marginClient";
import { MarginPoolConfigData, MarginPoolData } from "./state";
import { MarginTokenConfig } from "../config";
import { PoolTokenChange } from "./poolTokenChange";
import { TokenMetadata } from "../metadata/state";
import { PriceInfo } from "../accountPosition";
import { Number192 } from "../../utils";
import { PositionTokenMetadata } from "../positionTokenMetadata";
/** A set of possible actions to perform on a margin pool. */
export declare type PoolAction = "deposit" | "withdraw" | "borrow" | "repay" | "repayFromDeposit" | "swap" | "transfer";
/** The PDA addresses associated with a [[Pool]] */
export interface PoolAddresses {
    /** The pool's token mint i.e. BTC or SOL mint address*/
    tokenMint: PublicKey;
    marginPool: PublicKey;
    vault: PublicKey;
    depositNoteMint: PublicKey;
    loanNoteMint: PublicKey;
    marginPoolAdapterMetadata: PublicKey;
    tokenMetadata: PublicKey;
    depositNoteMetadata: PublicKey;
    loanNoteMetadata: PublicKey;
    controlAuthority: PublicKey;
}
export interface PriceResult {
    priceValue: Number192;
    depositNotePrice: BN;
    depositNoteConf: BN;
    depositNoteTwap: BN;
    loanNotePrice: BN;
    loanNoteConf: BN;
    loanNoteTwap: BN;
}
/**
 * A projection or estimation of the pool after an action is taken.
 *
 * @export
 * @interface PoolProjection
 */
export interface PoolProjection {
    riskIndicator: number;
    depositRate: number;
    borrowRate: number;
}
/**
 * An SPL swap pool
 *
 * @export
 * @interface SPLSwapPool
 */
export interface SPLSwapPool {
    swapPool: string;
    authority: string;
    poolMint: string;
    tokenMintA: string;
    tokenMintB: string;
    tokenA: string;
    tokenB: string;
    feeAccount: string;
    swapProgram: string;
    swapFees: number;
    swapType: "constantProduct" | "stable";
    amp?: number;
}
export declare const feesBuffer: number;
/**
 * A pool in which a [[MarginAccount]] can register a deposit and/or a borrow position.
 *
 * @export
 * @class Pool
 */
export declare class Pool {
    programs: MarginPrograms;
    addresses: PoolAddresses;
    tokenConfig: MarginTokenConfig;
    /**
     * The metadata of the [[Pool]] deposit note mint
     *
     * @type {PositionTokenMetadata}
     * @memberof Pool
     */
    depositNoteMetadata: PositionTokenMetadata;
    /**
     * The metadata of the [[Pool]] loan note mint
     *
     * @type {PositionTokenMetadata}
     * @memberof Pool
     */
    loanNoteMetadata: PositionTokenMetadata;
    /**
     * The address of the [[Pool]]
     *
     * @readonly
     * @type {PublicKey}
     * @memberof Pool
     */
    get address(): PublicKey;
    /**
     * The token mint of the [[Pool]]. It is incorrect to register a [[MarginAccount]] position using the token mint.
     * Rather `depositNoteMint` and `loanNoteMint` positions should be registered
     *
     * @readonly
     * @type {PublicKey}
     * @memberof Pool
     */
    get tokenMint(): PublicKey;
    /**
     * The long-form token name
     *
     * @readonly
     * @type {(string | undefined)}
     * @memberof Pool
     */
    get name(): string | undefined;
    /**
     * The token symbol, such as "BTC" or "SOL"
     *
     * @readonly
     * @type {string}
     * @memberof Pool
     */
    get symbol(): string;
    /**
     * The raw vault balance
     *
     * @readonly
     * @type {Number192}
     * @memberof Pool
     */
    private get vaultRaw();
    /**
     * The vault token balance
     *
     * @readonly
     * @type {TokenAmount}
     * @memberof Pool
     */
    get vault(): TokenAmount;
    /**
     * The raw borrowed token balance
     *
     * @readonly
     * @private
     * @memberof Pool
     */
    private get borrowedTokensRaw();
    /**
     * The borrowed tokens of the vault
     *
     * @readonly
     * @type {TokenAmount}
     * @memberof Pool
     */
    get borrowedTokens(): TokenAmount;
    private get totalValueRaw();
    /**
     * The total tokens currently borrowed + available to borrow
     *
     * @readonly
     * @type {TokenAmount}
     * @memberof Pool
     */
    get totalValue(): TokenAmount;
    /**
     * The raw uncollected fees
     *
     * @readonly
     * @type {Number192}
     * @memberof Pool
     */
    private get uncollectedFeesRaw();
    /**
     * The uncollected fees of the pool.
     *
     * @readonly
     * @type {TokenAmount}
     * @memberof Pool
     */
    get uncollectedFees(): TokenAmount;
    /**
     * The borrow utilization rate, where 0 is no borrows and 1 is all tokens borrowed.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get utilizationRate(): number;
    /**
     * The continuous compounding deposit rate.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get depositCcRate(): number;
    /**
     * The APY depositors receive, determined by the utilization curve.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get depositApy(): number;
    /**
     * The APR borrowers pay, determined by the utilization curve.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get borrowApr(): number;
    /**
     * The management fee, a fraction of interest paid.
     */
    get managementFeeRate(): number;
    /**
     * The token price in USD provided by Pyth.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get tokenPrice(): number;
    private prices;
    get depositNotePrice(): PriceInfo;
    get loanNotePrice(): PriceInfo;
    /**
     * The token mint decimals
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get decimals(): number;
    /**
     * The visual token precision for UI strings.
     *
     * @readonly
     * @type {number}
     * @memberof Pool
     */
    get precision(): number;
    /**
     * Underlying accounts associated with the [[Pool]]
     *
     * @type {{
     *     marginPool: MarginPoolData
     *     tokenMint: Mint
     *     vault: AssociatedToken
     *     depositNoteMint: Mint
     *     loanNoteMint: Mint
     *     tokenPriceOracle: PriceData
     *     tokenMetadata: TokenMetadata
     *   }}
     * @memberof Pool
     */
    info?: {
        marginPool: MarginPoolData;
        tokenMint: Mint;
        vault: AssociatedToken;
        depositNoteMint: Mint;
        loanNoteMint: Mint;
        tokenPriceOracle: PriceData;
        tokenMetadata: TokenMetadata;
    };
    /**
     * Creates a Pool
     *
     * @param programs
     * @param addresses
     * @param tokenConfig
     */
    constructor(programs: MarginPrograms, addresses: PoolAddresses, tokenConfig: MarginTokenConfig);
    refresh(): Promise<void>;
    /****************************
     * Program Implementation
     ****************************/
    calculatePrices(pythPrice: PriceData | undefined): PriceResult;
    /**
     * Get the USD value of the smallest unit of deposit notes
     *
     * @return {Number192}
     * @memberof Pool
     */
    depositNoteExchangeRate(): Number192;
    /**
     * Get the USD value of the smallest unit of loan notes
     *
     * @return {Number192}
     * @memberof Pool
     */
    loanNoteExchangeRate(): Number192;
    /**
     * Linear interpolation between (x0, y0) and (x1, y1)
     * @param {number} x
     * @param {number} x0
     * @param {number} x1
     * @param {number} y0
     * @param {number} y1
     * @returns {number}
     */
    static interpolate: (x: number, x0: number, x1: number, y0: number, y1: number) => number;
    /**
     * Continous Compounding Rate
     * @param {number} reserveConfig
     * @param {number} utilRate
     * @returns {number}
     */
    static getCcRate(reserveConfig: MarginPoolConfigData, utilRate: number): number;
    /**
     * Get continuous compounding borrow rate.
     *
     * @static
     * @param {number} ccRate
     * @return {number}
     * @memberof Pool
     */
    static getBorrowRate(ccRate: number): number;
    /**
     * Get continuous compounding deposit rate.
     *
     * @static
     * @param {number} ccRate
     * @param {number} utilRatio
     * @param {number} feeFraction
     * @return {*}  {number}
     * @memberof Pool
     */
    static getDepositRate(ccRate: number, utilRatio: number, feeFraction: number): number;
    getPrice(mint: PublicKey): PriceInfo | undefined;
    static getPrice(mint: PublicKey, pools: Record<any, Pool> | Pool[]): PriceInfo | undefined;
    /****************************
     * Transactionss
     ****************************/
    /**
     * Send a transaction to refresh all [[MarginAccount]] pool positions so that additional
     * borrows or withdraws can occur.
     *
     * @param {({
     *     pools: Record<any, Pool> | Pool[]
     *     marginAccount: MarginAccount
     *   })} {
     *     pools,
     *     marginAccount
     *   }
     * @return {Promise<string>}
     * @memberof Pool
     */
    marginRefreshAllPositionPrices({ pools, marginAccount }: {
        pools: Record<any, Pool> | Pool[];
        marginAccount: MarginAccount;
    }): Promise<string>;
    /**
     * Send a transaction to refresh all [[MarginAccount]] deposit or borrow positions associated with this [[Pool]] so that additional
     * borrows or withdraws can occur.
     *
     * @param {MarginAccount} marginAccount
     * @return {Promise<string>}
     * @memberof Pool
     */
    marginRefreshPositionPrice(marginAccount: MarginAccount): Promise<string>;
    /**
     * Create instructions to refresh all [[MarginAccount]] pool positions so that additional
     * borrows or withdraws can occur.
     *
     * @param {({
     *     instructions: TransactionInstruction[]
     *     pools: Record<any, Pool> | Pool[]
     *     marginAccount: MarginAccount
     *   })} {
     *     instructions,
     *     pools,
     *     marginAccount
     *   }
     * @return {Promise<void>}
     * @memberof Pool
     */
    withMarginRefreshAllPositionPrices({ instructions, pools, marginAccount }: {
        instructions: TransactionInstruction[];
        pools: Record<any, Pool> | Pool[];
        marginAccount: MarginAccount;
    }): Promise<void>;
    /**
     * Create instructions to refresh all [[MarginAccount]] deposit or borrow positions associated with this [[Pool]] so that additional
     * borrows or withdraws can occur.
     *
     * @param {{
     *     instructions: TransactionInstruction[]
     *     marginAccount: MarginAccount
     *   }} {
     *     instructions,
     *     marginAccount
     *   }
     * @return {Promise<void>}
     * @memberof Pool
     */
    withMarginRefreshPositionPrice({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<void>;
    /**
     * Send a transaction to deposit tokens into the pool.
     *
     * This function will
     * - create the margin account (if required),
     * - register the position (if required),
     * - Wrap SOL according to the `source` param,
     * - and update the position balance after.
     *
     * @param args
     * @param args.marginAccount - The margin account that will receive the deposit.
     * @param args.change - The amount of tokens to be deposited in lamports.
     * @param args.source - (Optional) The token account that the deposit will be transfered from. The wallet balance or associated token account will be used if unspecified.
     */
    deposit({ marginAccount, change, source }: {
        marginAccount: MarginAccount;
        change: PoolTokenChange;
        source?: TokenAddress;
    }): Promise<string>;
    /**
     * Create an instruction to deposit into the pool.
     *
     * This function will wrap SOL according to the `source` param.
     * It is required that
     * - The margin account is created,
     * - a deposit position is registered
     * - and the position balance is updated after.
     *
     * @param args
     * @param args.instructions - The array to append instructions to
     * @param args.marginAccount - The margin account that will receive the deposit.
     * @param args.source - (Optional) The token account that the deposit will be transfered from. The wallet balance or associated token account will be used if unspecified.
     * @param args.change - The amount of tokens to be deposited in lamports.
     * @return {Promise<void>}
     * @memberof Pool
     */
    withDeposit({ instructions, marginAccount, source, change }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        source?: TokenAddress;
        change: PoolTokenChange;
    }): Promise<void>;
    marginBorrow({ marginAccount, pools, change, destination }: {
        marginAccount: MarginAccount;
        pools: Record<string, Pool> | Pool[];
        change: PoolTokenChange;
        destination?: TokenAddress;
    }): Promise<string>;
    withMarginBorrow({ instructions, marginAccount, change }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        change: PoolTokenChange;
    }): Promise<void>;
    marginRepay({ marginAccount, pools, source, change, closeLoan, signer }: {
        marginAccount: MarginAccount;
        pools: Record<string, Pool> | Pool[];
        source?: TokenAddress;
        change: PoolTokenChange;
        closeLoan?: boolean;
        signer?: Address;
    }): Promise<string>;
    withMarginRepay({ instructions, marginAccount, change }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        change: PoolTokenChange;
    }): Promise<void>;
    withRepay({ instructions, marginAccount, source, change, feesBuffer, sourceAuthority }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        depositPosition: Address;
        source: TokenAddress;
        change: PoolTokenChange;
        feesBuffer: number;
        sourceAuthority?: Address;
    }): Promise<void>;
    withdraw({ marginAccount, pools, change, destination }: {
        marginAccount: MarginAccount;
        pools: Record<string, Pool> | Pool[];
        change: PoolTokenChange;
        destination?: TokenAddress;
    }): Promise<string>;
    withWithdraw({ instructions, marginAccount, destination, change }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        destination?: TokenAddress;
        change: PoolTokenChange;
    }): Promise<void>;
    /**
     * Transaction to swap tokens
     *
     * @param `marginAccount` - The margin account that will receive the deposit.
     * @param `pools` - Array of margin pools
     * @param `outputToken` - The corresponding pool for the token being swapped to.
     * @param `swapPool` - The SPL swap pool the exchange is taking place in.
     * @param `swapAmount` - The amount being swapped.
     * @param `minAmountOut` - The minimum output amount based on swapAmount and slippage.
     */
    splTokenSwap({ marginAccount, pools, outputToken, swapPool, swapAmount, minAmountOut, repayWithOutput }: {
        marginAccount: MarginAccount;
        pools: Pool[];
        outputToken: Pool;
        swapPool: SPLSwapPool;
        swapAmount: TokenAmount;
        minAmountOut: TokenAmount;
        repayWithOutput: boolean;
    }): Promise<string>;
    withSPLTokenSwap({ instructions, marginAccount, outputToken, swapPool, changeKind, minAmountOut, sourceAccount, destinationAccount, transitSourceAccount, transitDestinationAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
        outputToken: Pool;
        swapPool: SPLSwapPool;
        changeKind: PoolTokenChange;
        minAmountOut: TokenAmount;
        sourceAccount: Address;
        destinationAccount: Address;
        transitSourceAccount: Address;
        transitDestinationAccount: Address;
    }): Promise<void>;
    /**
     * Derive the address of a deposit position token account associated with a [[MarginAccount]].
     *
     * @param {MarginAccount} marginAccount
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    findDepositPositionAddress(marginAccount: MarginAccount): PublicKey;
    withGetOrRegisterDepositPosition({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<Address>;
    /**
     * Get instruction to register new pool deposit position that is custodied by margin
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Register the SOL deposit position
     * const pool = pools["SOL"]
     * await pool.withRegisterDepositPosition({ instructions, marginAccount })
     * ```
     *
     * @param args
     * @param {TransactionInstruction[]} args.instructions Instructions array to append to.
     * @param {Address} args.marginAccount The margin account that will custody the position.
     * @return {Promise<PublicKey>} Returns the instruction, and the address of the deposit note token account to be created for the position.
     * @memberof MarginAccount
     */
    withRegisterDepositPosition({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<Address>;
    withCloseDepositPosition({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<void>;
    withdrawAndCloseDepositPosition({ marginAccount, destination }: {
        marginAccount: MarginAccount;
        destination: Address;
    }): Promise<void>;
    findLoanPositionAddress(marginAccount: MarginAccount): PublicKey;
    withGetOrRegisterLoanPosition({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<Address>;
    withRegisterLoanPosition(instructions: TransactionInstruction[], marginAccount: MarginAccount): Promise<Address>;
    withCloseLoanPosition({ instructions, marginAccount }: {
        instructions: TransactionInstruction[];
        marginAccount: MarginAccount;
    }): Promise<void>;
    projectAfterAction(marginAccount: MarginAccount, amount: number, action: PoolAction, minAmountOut?: number, outputToken?: Pool): PoolProjection;
    projectAfterDeposit(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterWithdraw(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterBorrow(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterRepay(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterRepayFromDeposit(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterBorrowAndNotWithdraw(marginAccount: MarginAccount, amount: number): PoolProjection;
    projectAfterMarginSwap(marginAccount: MarginAccount, amount: number, minAmountOut: number | undefined, outputToken: Pool | undefined, setupCheck?: boolean): PoolProjection;
    private getDefaultPoolProjection;
}
//# sourceMappingURL=pool.d.ts.map