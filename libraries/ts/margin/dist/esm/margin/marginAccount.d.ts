import { Address, AnchorProvider, BN } from "@project-serum/anchor";
import { GetProgramAccountsFilter, PublicKey, Signer, TransactionInstruction, TransactionSignature } from "@solana/web3.js";
import { Pool, PoolAction } from "./pool/pool";
import { AccountPositionList, ErrorCode, LiquidationData, MarginAccountData } from "./state";
import { MarginPrograms } from "./marginClient";
import { AssociatedToken, TokenAmount } from "..";
import { Number128 } from "../utils/number128";
import { MarginTokenConfig } from "./config";
import { AccountPosition, PriceInfo } from "./accountPosition";
/** A description of a position associated with a [[MarginAccount]] and [[Pool]] */
export interface PoolPosition {
    /** The [[MarginTokenConfig]] associated with the [[Pool]] token. */
    tokenConfig: MarginTokenConfig;
    /** The [[Pool]] that the position is associated with. */
    pool?: Pool;
    /** The underlying [[AccountPosition]] that stores the deposit balance. */
    depositPosition: AccountPosition | undefined;
    /** The deposit balance in the [[Pool]]. An undefined `depositPosition` leads to a balance of 0. */
    depositBalance: TokenAmount;
    /** The deposit value in the [[Pool]] denominated in USD. An undefined `depositPosition` leads to a value of 0. */
    depositValue: number;
    /** The underlying [[AccountPosition]] that stores the loan balance. */
    loanPosition: AccountPosition | undefined;
    /** The loan balance in the [[Pool]]. An undefined `loanPosition` leads to a balance of 0. */
    loanBalance: TokenAmount;
    /** The loan value in the [[Pool]] denominated in USD. An undefined `loanPosition` leads to a balance of 0. */
    loanValue: number;
    /**
     * An estimate of the maximum trade amounts possible.
     * The estimates factor in available wallet balances, [[Pool]] liquidity, margin requirements
     * and [[SETUP_LEVERAGE_FRACTION]]. */
    maxTradeAmounts: Record<PoolAction, TokenAmount>;
    /** An estimate of the amount of [[MarginTokenConfig]] collateral required to make it possible to end liquidation. */
    liquidationEndingCollateral: TokenAmount;
    buyingPower: TokenAmount;
}
export interface AccountSummary {
    depositedValue: number;
    borrowedValue: number;
    accountBalance: number;
    availableCollateral: number;
    leverage: number;
    /** @deprecated use riskIndicator */
    cRatio: number;
    /** @deprecated use riskIndicator */
    minCRatio: number;
}
/** A summation of the USD values of various positions used in margin accounting. */
export interface Valuation {
    liabilities: Number128;
    requiredCollateral: Number128;
    requiredSetupCollateral: Number128;
    weightedCollateral: Number128;
    effectiveCollateral: Number128;
    availableCollateral: Number128;
    availableSetupCollateral: Number128;
    staleCollateralList: [PublicKey, ErrorCode][];
    pastDue: boolean;
    claimErrorList: [PublicKey, ErrorCode][];
}
/**
 * A collection of [[AssociatedToken]] wallet balances. Note that only associated token accounts
 * will be present and auxiliary accounts are ignored.
 */
export interface MarginWalletTokens {
    /** An array of every associated token account owned by the wallet. */
    all: AssociatedToken[];
    /** A map of token symbols to associated token accounts.
     *
     * ## Usage
     *
     * ```ts
     * map["USDC"].amount.tokens.toFixed(2)
     * ```
     *
     * ## Remarks
     *
     * Only tokens within the [[MarginConfig]] will be present. */
    map: Record<string, AssociatedToken>;
}
export declare class MarginAccount {
    programs: MarginPrograms;
    provider: AnchorProvider;
    seed: number;
    pools?: Record<string, Pool> | undefined;
    walletTokens?: MarginWalletTokens | undefined;
    /**
     * The maximum [[MarginAccount]] seed value equal to `65535`.
     * Seeds are a 16 bit number and therefor only 2^16 margin accounts may exist per wallet. */
    static readonly SEED_MAX_VALUE = 65535;
    static readonly RISK_WARNING_LEVEL = 0.8;
    static readonly RISK_CRITICAL_LEVEL = 0.9;
    static readonly RISK_LIQUIDATION_LEVEL = 1;
    /** The maximum risk indicator allowed by the library when setting up a  */
    static readonly SETUP_LEVERAGE_FRACTION: Number128;
    /** The raw accounts associated with the margin account. */
    info?: {
        /** The decoded [[MarginAccountData]]. */
        marginAccount: MarginAccountData;
        /** The decoded [[LiquidationData]]. This may only be present during liquidation. */
        liquidationData?: LiquidationData;
        /** The decoded position data in the margin account. */
        positions: AccountPositionList;
    };
    /** The address of the [[MarginAccount]] */
    address: PublicKey;
    /** The owner of the [[MarginAccount]] */
    owner: PublicKey;
    /** The parsed [[AccountPosition]] array of the margin account. */
    positions: AccountPosition[];
    /** The summarized [[PoolPosition]] array of pool deposits and borrows. */
    poolPositions: Record<string, PoolPosition>;
    /** The [[Valuation]] of the margin account. */
    valuation: Valuation;
    summary: AccountSummary;
    get liquidator(): PublicKey | undefined;
    /** @deprecated Please use `marginAccount.info.liquidation` instead */
    get liquidaton(): PublicKey | undefined;
    /**
     * Returns true if a [[LiquidationData]] account exists and is associated with the [[MarginAccount]].
     * Certain actions are not allowed while liquidation is in progress.
     */
    get isBeingLiquidated(): boolean | undefined;
    /** A qualitative measure of the the health of a margin account.
     * A higher value means more risk in a qualitative sense.
     * Properties:
     *  non-negative, range is [0, infinity)
     *  zero only when an account has no exposure at all
     *  account is subject to liquidation at a value of one
     */
    get riskIndicator(): number;
    /** Compute the risk indicator using components from [[Valuation]] */
    computeRiskIndicator(requiredCollateral: number, weightedCollateral: number, liabilities: number): number;
    /**
     * Creates an instance of margin account.
     * @param {MarginPrograms} programs
     * @param {Provider} provider The provider and wallet that can sign for this margin account
     * @param {Address} owner
     * @param {number} seed
     * @param {Record<string, Pool>} pools
     * @param {MarginWalletTokens} walletTokens
     * @memberof MarginAccount
     */
    constructor(programs: MarginPrograms, provider: AnchorProvider, owner: Address, seed: number, pools?: Record<string, Pool> | undefined, walletTokens?: MarginWalletTokens | undefined);
    /**
     * Derive margin account PDA from owner address and seed
     *
     * @private
     * @static
     * @param {MarginPrograms} programs
     * @param {Address} owner
     * @param {number} seed
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    static derive(programs: MarginPrograms, owner: Address, seed: number): PublicKey;
    /**
     * Derive the address of a [[LiquidationData]] account.
     *
     * @param {Address} liquidator
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    findLiquidationAddress(liquidator: Address): PublicKey;
    /**
     * Derive the address of a metadata account.
     *
     * ## Remarks
     *
     * Some account types such as pools, adapters and position mints have
     * metadata associated with them. The metadata type is determined by the account type.
     *
     * @param {Address} account
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    findMetadataAddress(account: Address): PublicKey;
    /**
     * Derive the address of a position token account associated with a [[MarginAccount]]
     * and position token mint.
     *
     * ## Remarks
     *
     * It is recommended to use other functions to find specfic position types. e.g. using [[Pool]].findDepositPositionAddress
     *
     * @param {Address} positionTokenMint
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    findPositionTokenAddress(positionTokenMint: Address): PublicKey;
    /**
     *
     * @param args
     * @param {MarginPrograms} args.programs
     * @param {AnchorProvider} args.provider The provider and wallet that can sign for this margin account
     * @param {Record<string, Pool>} args.pools Collection of [[Pool]] to calculate pool positions and prices.
     * @param {MarginWalletTokens} args.walletTokens Tokens owned by the wallet to calculate max deposit amounts.
     * @param {Address} args.owner
     * @param {number} args.seed
     * @returns {Promise<MarginAccount>}
     */
    static load({ programs, provider, pools, walletTokens, owner, seed }: {
        programs: MarginPrograms;
        provider: AnchorProvider;
        pools?: Record<string, Pool>;
        walletTokens?: MarginWalletTokens;
        owner: Address;
        seed: number;
    }): Promise<MarginAccount>;
    /**
     * Load all margin accounts for a wallet with an optional filter.
     *
     * @static
     * @param {({
     *     programs: MarginPrograms
     *     provider: AnchorProvider
     *     pools?: Record<string, Pool>
     *     walletTokens?: MarginWalletTokens
     *     filters?: GetProgramAccountsFilter[] | Buffer
     *   })} {
     *     programs,
     *     provider,
     *     pools,
     *     walletTokens,
     *     filters
     *   }
     * @return {Promise<MarginAccount[]>}
     * @memberof MarginAccount
     */
    static loadAllByOwner({ programs, provider, pools, walletTokens, owner, filters }: {
        programs: MarginPrograms;
        provider: AnchorProvider;
        pools?: Record<string, Pool>;
        walletTokens?: MarginWalletTokens;
        owner: Address;
        filters?: GetProgramAccountsFilter[];
    }): Promise<MarginAccount[]>;
    refresh(): Promise<void>;
    private getAllPoolPositions;
    private getMaxTradeAmounts;
    private getSummary;
    /**
     * Get the array of regstered [[AccountPosition]] on this account
     *
     * @return {AccountPosition[]}
     * @memberof MarginAccount
     */
    getPositions(): AccountPosition[];
    /**
     * Get the registerd [[AccountPosition]] associated with the position mint.
     * Throws an error if the position does not exist.
     *
     * @param {Address} mint The position mint. For example a [[Pool]] deposit note mint.
     * @return {(AccountPosition)}
     * @memberof MarginAccount
     */
    getPosition(mint: Address): AccountPosition;
    /**
     * Get the registerd [[AccountPosition]] associated with the position mint.
     *
     * @param {Address} mint The position mint. For example a [[Pool]] deposit note mint.
     * @return {(AccountPosition | undefined)}
     * @memberof MarginAccount
     */
    getPositionNullable(mint: Address): AccountPosition | undefined;
    setPositionBalance(mint: PublicKey, account: PublicKey, balance: BN): AccountPosition | undefined;
    getPositionPrice(mint: PublicKey): PriceInfo | undefined;
    setPositionPrice(mint: PublicKey, price: PriceInfo): void;
    /**
     * Check if the given address is an authority for this margin account.
     * The owner has authority, as well as a liquidator only during liquidation.
     */
    hasAuthority(authority: PublicKey): boolean | undefined;
    private getValuation;
    /**
     * Loads all tokens in the users wallet.
     * Provides an array and a map of tokens mapped by pool.
     *
     * @static
     * @param {MarginPrograms} programs
     * @param {Address} owner
     * @return {Promise<MarginWalletTokens>}
     * @memberof MarginAccount
     */
    static loadTokens(programs: MarginPrograms, owner: Address): Promise<MarginWalletTokens>;
    /**
     * Fetches the account and returns if it exists.
     *
     * @return {Promise<boolean>}
     * @memberof MarginAccount
     */
    static exists(programs: MarginPrograms, owner: Address, seed: number): Promise<boolean>;
    /**
     * Fetches the account and returns if it exists
     *
     * @return {Promise<boolean>}
     * @memberof MarginAccount
     */
    exists(): Promise<boolean>;
    /**
     * Create the margin account if it does not exist.
     * If no seed is provided, one will be located.
     *
     * ## Example
     *
     * ```javascript
     * // Load programs
     * const config = await MarginClient.getConfig("devnet")
     * const programs = MarginClient.getPrograms(provider, config)
     *
     * // Load tokens and wallet
     * const pools = await poolManager.loadAll()
     * const walletTokens = await MarginAccount.loadTokens(programs, walletPubkey)
     *
     * // Create margin account
     * const marginAccount = await MarginAccount.createAccount({
     *   programs,
     *   provider,
     *   owner: wallet.publicKey,
     *   seed: 0,
     *   pools,
     *   walletTokens
     * })
     * ```
     *
     * @static
     * @param args
     * @param {MarginPrograms} args.programs
     * @param {AnchorProvider} args.provider A provider that may be used to sign transactions modifying the account
     * @param {Address} args.owner The address of the [[MarginAccount]] owner
     * @param {number} args.seed The seed or ID of the [[MarginAccount]] in the range of (0, 65535]
     * @param {Record<string, Pool>} args.pools A [[Pool]] collection to calculate pool positions.
     * @param {MarginWalletTokens} args.walletTokens The tokens in the owners wallet to determine max trade amounts.
     * @return {Promise<MarginAccount>}
     * @memberof MarginAccount
     */
    static createAccount({ programs, provider, owner, seed, pools, walletTokens }: {
        programs: MarginPrograms;
        provider: AnchorProvider;
        owner: Address;
        seed?: number;
        pools?: Record<string, Pool>;
        walletTokens?: MarginWalletTokens;
    }): Promise<MarginAccount>;
    /**
     * Searches for a margin account that does not exist yet and returns its seed.
     *
     * @static
     * @param {{
     *     programs: MarginPrograms
     *     provider: AnchorProvider
     *     owner: Address
     *   }}
     * @memberof MarginAccount
     */
    static getUnusedAccountSeed({ programs, provider, owner }: {
        programs: MarginPrograms;
        provider: AnchorProvider;
        owner: Address;
    }): Promise<number>;
    /**
     * Create the margin account if it does not exist.
     * If no seed is provided, one will be located.
     *
     * ## Example
     *
     * ```javascript
     * // Load programs
     * const config = await MarginClient.getConfig("devnet")
     * const programs = MarginClient.getPrograms(provider, config)
     *
     * // Load tokens and wallet
     * const pools = await poolManager.loadAll()
     * const walletTokens = await MarginAccount.loadTokens(programs, walletPubkey)
     *
     * // Create margin account
     * const marginAccount = new MarginAccount({
     *    programs,
     *    provider,
     *    walletPubkey,
     *    0,
     *    pools,
     *    walletTokens
     * })
     *
     * await marginAccount.createAccount()
     * ```
     *
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    createAccount(): Promise<void>;
    /**
     * Get instruction to create the account if it does not exist.
     *
     * ## Example
     *
     * ```ts
     * const instructions: TransactionInstruction[] = []
     * await marginAccount.withCreateAccount(instructions)
     * if (instructions.length > 0) {
     *   await marginAccount.sendAndConfirm(instructions)
     * }
     * ```
     *
     * @param {TransactionInstruction[]} instructions
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withCreateAccount(instructions: TransactionInstruction[]): Promise<void>;
    /**
     * Updates all position balances. `withUpdatePositionBalance` is often included
     * in transactions after modifying balances to synchronize with the margin account.
     *
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    updateAllPositionBalances(): Promise<string>;
    /**
     * Create instructions to update all position balances. `withUpdatePositionBalance` often included in
     * transactions after modifying balances ot synchronize with the margin account.
     *
     * ## Example
     *
     * ```ts
     * const instructions: TransactionInstruction[] = []
     * await marginAccount.withUpdateAllPositionBalances({ instructions })
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param {{ instructions: TransactionInstruction[] }} { instructions }
     * @memberof MarginAccount
     */
    withUpdateAllPositionBalances({ instructions }: {
        instructions: TransactionInstruction[];
    }): Promise<void>;
    /**
     * Updates a single position balance. `withUpdatePositionBalance` is often included
     * in transactions after modifying balances to synchronize with the margin account.
     *
     * @param {{ position: AccountPosition }} { position }
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    updatePositionBalance({ position }: {
        position: AccountPosition;
    }): Promise<string>;
    /**
     * Get instruction to update the accounting for assets in
     * the custody of the margin account.
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Find the position
     * const depositNote = pools["SOL"].addresses.depositNoteMint
     * const position = marginAccount.getPosition(depositNote).address
     *
     * // Update the position balance
     * const instructions: TransactionInstruction[] = []
     * await marginAccount.withUpdatePositionBalance({ instructions, position })
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param {{
     *     instructions: TransactionInstruction[]
     *     position: Address
     *   }} {
     *     instructions,
     *     position
     *   }
     * @return {*}  {Promise<void>}
     * @memberof MarginAccount
     */
    withUpdatePositionBalance({ instructions, position }: {
        instructions: TransactionInstruction[];
        position: Address;
    }): Promise<void>;
    /**
     * Sends a transaction to refresh the metadata for a position.
     *
     * ## Remarks
     *
     * When a position is registered some position mint metadata is copied to the position.
     * This data can become out of sync if the mint metadata is changed. Refreshing the position
     * metadata may at the benefit or detriment to the owner.
     *
     * @param {{ positionMint: Address }} { positionMint }
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    refreshPositionMetadata({ positionMint }: {
        positionMint: Address;
    }): Promise<string>;
    /**
     * Creates an instruction to refresh the metadata for a position.
     *
     * ## Remarks
     *
     * When a position is registered some position mint metadata is copied to the position.
     * This data can become out of sync if the mint metadata is changed. Refreshing the position
     * metadata may at the benefit or detriment to the owner.
     *
     * @param {{
     *     instructions: TransactionInstruction[]
     *     positionMint: Address
     *   }} {
     *     instructions,
     *     positionMint
     *   }
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withRefreshPositionMetadata({ instructions, positionMint }: {
        instructions: TransactionInstruction[];
        positionMint: Address;
    }): Promise<void>;
    /**
     * Get the [[AccountPosition]] [[PublicKey]] and sends a transaction to
     * create it if it doesn't exist.
     *
     * ## Remarks
     *
     * It is recommended to use other functions to register specfic position types. e.g. using [[Pool]].withRegisterDepositPosition
     *
     * In web apps it's recommended to call `withGetOrCreatePosition` as part of a larger
     * transaction to prompt for a wallet signature less often.
     *
     * ## Example
     *
     * ```ts
     * // Load margin pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * const depositNote = pools["SOL"].addresses.depositNoteMint
     * await marginAccount.getOrRegisterPosition(depositNote)
     * ```
     *
     * @param {Address} tokenMint
     * @return {Promise<PublicKey>}
     * @memberof MarginAccount
     */
    getOrRegisterPosition(tokenMint: Address): Promise<PublicKey>;
    /**
     * Get the [[AccountPosition]] [[PublicKey]] and appends an instructon to
     * create it if it doesn't exist.
     *
     * ## Remarks
     *
     * It is recommended to use other functions to register specfic position types. e.g. using [[Pool]].withRegisterDepositPosition
     *
     * ## Example
     *
     * ```ts
     * // Load margin pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Register position
     * const positionTokenMint = pools["SOL"].addresses.depositNoteMint
     * const instructions: TransactionInstruction[] = []
     * await marginAccount.withGetOrRegisterPosition({ instructions, positionTokenMint })
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param args
     * @param {TransactionInstruction[]} args.instructions The instructions to append to
     * @param {Address} args.positionTokenMint The position mint to register a position for
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    withGetOrRegisterPosition({ instructions, positionTokenMint }: {
        instructions: TransactionInstruction[];
        positionTokenMint: Address;
    }): Promise<PublicKey>;
    /**
     * Sends a transaction to register an [[AccountPosition]] for the mint. When registering a [[Pool]] position,
     * the mint would not be Bitcoin or SOL, but rather the `depositNoteMint` or `loanNoteMint` found in `pool.addresses`.
     * A margin account has a limited capacity of positions.
     *
     * ## Remarks
     *
     * It is recommended to use other functions to register specfic position types. e.g. using [[Pool]].withRegisterDepositPosition
     *
     * In web apps it's is recommended to use `withRegisterPosition` as part of a larget transaction
     * to prompt for a wallet signature less often.
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Register the SOL deposit position
     * const depositNoteMint = pools["SOL"].addresses.depositNoteMint
     * await marginAccount.registerPosition(depositNoteMint)
     * ```
     *
     * @param {Address} tokenMint
     * @return {Promise<TransactionSignature>}
     * @memberof MarginAccount
     */
    registerPosition(tokenMint: Address): Promise<TransactionSignature>;
    /**
     * Get instruction to register new position
     *
     * ## Remarks
     *
     * It is recommended to use other functions to register specfic position types. e.g. using [[Pool]].withRegisterDepositPosition
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Register the SOL deposit position
     * const positionTokenMint = pools["SOL"].addresses.depositNoteMint
     * const instructions: TransactionInstruction[] = []
     * const position = await marginAccount.withRegisterPosition({ instructions, positionTokenMint })
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param args
     * @param {TransactionInstruction[]} args.instructions Instructions array to append to.
     * @param {Address} args.positionTokenMint The mint for the relevant token for the position
     * @return {Promise<PublicKey>} Returns the instruction, and the address of the token account to be created for the position.
     * @memberof MarginAccount
     */
    withRegisterPosition({ instructions, positionTokenMint }: {
        instructions: TransactionInstruction[];
        positionTokenMint: Address;
    }): Promise<PublicKey>;
    /**
     * Send a transaction to close the [[MarginAccount]] and return rent to the owner.
     * All positions must have a zero balance and be closed first.
     *
     * ## Example
     *
     * ```ts
     * // Close all positions. A non zero balance results in an error
     * for (const position of marginAccount.getPositions()) {
     *   await marginAccount.closePosition(position)
     * }
     *
     * // Close the account and send the transaction
     * await marginAccount.closeAccount()
     * ```
     *
     * @memberof MarginAccount
     */
    closeAccount(): Promise<void>;
    /**
     * Create an instruction to close the [[MarginAccount]] and return rent to the owner.
     * All positions must have a zero balance and be closed first.
     *
     * ## Example
     *
     * ```ts
     * const instructions: TransactionInstruction[] = []
     *
     * // Close all positions. A non zero balance results in an error
     * for (const position of marginAccount.getPositions()) {
     *   await marginAccount.withClosePosition(instructions, position)
     * }
     *
     * // Close the account and send the transaction
     * await marginAccount.withCloseAccount(instructions)
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param {TransactionInstruction[]} instructions
     * @returns {Promise<void>}
     * @memberof MarginAccount
     */
    withCloseAccount(instructions: TransactionInstruction[]): Promise<void>;
    /**
     * Send a transaction to close a position. A non-zero balance will result in a transaction error.
     * There is a limited capacity for positions so it is recommended to close positions that
     * are no longer needed.
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Get the SOL position
     * const depositNoteMint = pools["SOL"].addresses.depositNoteMint
     * const position = marginAccount.getPosition(depositNoteMint)
     *
     * await marginAccount.closePosition(position)
     * ```
     *
     * @param {AccountPosition} position
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    closePosition(position: AccountPosition): Promise<void>;
    /**
     * Create an instruction to close a position. A non-zero balance will result in a transaction error.
     * There is a limited capacity for positions so it is recommended to close positions that
     * are no longer needed.
     *
     * ## Example
     *
     * ```ts
     * // Load the pools
     * const poolManager = new PoolManager(programs, provider)
     * const pools = await poolManager.loadAll()
     *
     * // Get the SOL position
     * const depositNoteMint = pools["SOL"].addresses.depositNoteMint
     * const position = marginAccount.getPosition(depositNoteMint)
     *
     * const instructions: TransactionInstruction[] = []
     * await marginAccount.closePosition(instructions, position)
     * await marginAccount.sendAndConfirm(instructions)
     * ```
     *
     * @param {TransactionInstruction[]} instructions
     * @param {AccountPosition} position
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withClosePosition(instructions: TransactionInstruction[], position: AccountPosition): Promise<void>;
    /** @deprecated This has been renamed to `liquidateEnd` and will be removed in a future release. */
    stopLiquidation(): Promise<string>;
    /**
     * Get instruction to end a liquidation
     * @deprecated This has been renamed to `withLiquidateEnd` and will be removed in a future release. */
    withStopLiquidation(instructions: TransactionInstruction[]): Promise<void>;
    /**
     * Send a transaction to end a liquidation.
     *
     * ## Remarks
     *
     * The [[MarginAccount]] can enter liquidation while it's `riskIndicator` is at or above 1.0.
     * Liquidation is in progress when `isBeingLiquidated` returns true.
     * Liquidation can only end when enough collateral is deposited or enough collateral is liquidated to lower `riskIndicator` sufficiently.
     *
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    liquidateEnd(): Promise<string>;
    /**
     * Get instruction to end a liquidation
     *
     * ## Remarks
     *
     * The [[MarginAccount]] can enter liquidation while it's `riskIndicator` is at or above 1.0.
     * Liquidation is in progress when `isBeingLiquidated` returns true.
     *
     * ## Authority
     *
     * The [[MarginAccount]].`provider`.`wallet` will be used as the authority for the transaction.
     * The liquidator may end the liquidation at any time.
     * The margin account owner may end the liquidation only when at least one condition is true:
     * 1) When enough collateral is deposited or enough collateral is liquidated to lower `riskIndicator` sufficiently.
     * 2) When the liquidation has timed out when [[MarginAccount]]`.getRemainingLiquidationTime()` is negative
     *
     * @param {TransactionInstruction[]} instructions The instructions to append to
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withLiquidateEnd(instructions: TransactionInstruction[]): Promise<void>;
    /**
     * Get the time remaining on a liquidation until timeout in seconds.
     *
     * ## Remarks
     *
     * If `getRemainingLiquidationTime` is a negative number then `liquidationEnd` can be called
     * by the margin account owner regardless of the current margin account health.
     *
     * @return {number | undefined}
     * @memberof MarginAccount
     */
    getRemainingLiquidationTime(): number | undefined;
    /**
     * Create an instruction that performs an action by invoking other adapter programs, allowing them to alter
     * the balances of the token accounts belonging to this margin account. The transaction fails if the [[MarginAccount]]
     * does not have sufficent collateral.
     *
     * ## Remarks
     *
     * This instruction is not invoked directly, but rather internally for example by [[Pool]] when depositing.
     *
     * @param {{
     *     instructions: TransactionInstruction[]
     *     adapterProgram: Address
     *     adapterMetadata: Address
     *     adapterInstruction: TransactionInstruction
     *   }} {
     *     instructions,
     *     adapterProgram,
     *     adapterMetadata,
     *     adapterInstruction
     *   }
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withAdapterInvoke({ instructions, adapterProgram, adapterMetadata, adapterInstruction }: {
        instructions: TransactionInstruction[];
        adapterProgram: Address;
        adapterMetadata: Address;
        adapterInstruction: TransactionInstruction;
    }): Promise<void>;
    /**
     * Create an instruction to perform an action by invoking other adapter programs, allowing them only to
     * refresh the state of the margin account to be consistent with the actual
     * underlying prices or positions, but not permitting new position changes.
     *
     * ## Remarks
     *
     * This instruction is not invoked directly, but rather internally for example by [[Pool]] when depositing.
     * Accounting invoke is necessary when several position values have to be refreshed but in the interim there
     * aren't enough fresh positions to satisfy margin requirements.
     *
     * @param {{
     *     instructions: TransactionInstruction[]
     *     adapterProgram: Address
     *     adapterMetadata: Address
     *     adapterInstruction: TransactionInstruction
     *   }} {
     *     instructions,
     *     adapterProgram,
     *     adapterMetadata,
     *     adapterInstruction
     *   }
     * @return {Promise<void>}
     * @memberof MarginAccount
     */
    withAccountingInvoke({ instructions, adapterProgram, adapterMetadata, adapterInstruction }: {
        instructions: TransactionInstruction[];
        adapterProgram: Address;
        adapterMetadata: Address;
        adapterInstruction: TransactionInstruction;
    }): Promise<void>;
    /**
     * prepares arguments for `adapter_invoke`, `account_invoke`, or `liquidator_invoke`
     *
     * @return {AccountMeta[]} The instruction keys but the margin account is no longer a signer.
     * @memberof MarginAccount
     */
    private invokeAccounts;
    /**
     * Sends a transaction using the [[MarginAccount]] [[AnchorProvider]]
     *
     * @param {TransactionInstruction[]} instructions
     * @param {Signer[]} [signers]
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    sendAndConfirm(instructions: TransactionInstruction[], signers?: Signer[]): Promise<string>;
    /**
     * Sends a collection of transactions using the [[MarginAccount]] [[AnchorProvider]].
     *
     * ## Remarks
     *
     * This function has 2 additional features compared to `sendAll` from web3.js or anchor.
     * - Logging a [[Transaction]] error will include [[Transaction]] logs.
     * - If an [[Transaction]] array element is itself a `TransactionInstruction[][]` this function will send those transactions in parallel.
     *
     * @param {((TransactionInstruction[] | TransactionInstruction[][])[])} transactions
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    sendAll(transactions: (TransactionInstruction[] | TransactionInstruction[][])[]): Promise<string>;
}
//# sourceMappingURL=marginAccount.d.ts.map