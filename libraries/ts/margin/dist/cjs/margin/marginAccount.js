"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MarginAccount = void 0;
const assert_1 = __importDefault(require("assert"));
const anchor_1 = require("@project-serum/anchor");
const spl_token_1 = require("@solana/spl-token");
const web3_js_1 = require("@solana/web3.js");
const pool_1 = require("./pool/pool");
const state_1 = require("./state");
const pda_1 = require("../utils/pda");
const __1 = require("..");
const number128_1 = require("../utils/number128");
const accountPosition_1 = require("./accountPosition");
class MarginAccount {
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
    constructor(programs, provider, owner, seed, pools, walletTokens) {
        this.programs = programs;
        this.provider = provider;
        this.seed = seed;
        this.pools = pools;
        this.walletTokens = walletTokens;
        this.owner = (0, anchor_1.translateAddress)(owner);
        this.address = MarginAccount.derive(programs, owner, seed);
        this.pools = pools;
        this.walletTokens = walletTokens;
        this.positions = this.getPositions();
        this.valuation = this.getValuation(true);
        this.poolPositions = this.getAllPoolPositions();
        this.summary = this.getSummary();
    }
    get liquidator() {
        return this.info?.marginAccount.liquidator;
    }
    /** @deprecated Please use `marginAccount.info.liquidation` instead */
    get liquidaton() {
        return this.info?.marginAccount.liquidation;
    }
    /**
     * Returns true if a [[LiquidationData]] account exists and is associated with the [[MarginAccount]].
     * Certain actions are not allowed while liquidation is in progress.
     */
    get isBeingLiquidated() {
        return this.info && !this.info.marginAccount.liquidation.equals(web3_js_1.PublicKey.default);
    }
    /** A qualitative measure of the the health of a margin account.
     * A higher value means more risk in a qualitative sense.
     * Properties:
     *  non-negative, range is [0, infinity)
     *  zero only when an account has no exposure at all
     *  account is subject to liquidation at a value of one
     */
    get riskIndicator() {
        return this.computeRiskIndicator(this.valuation.requiredCollateral.toNumber(), this.valuation.weightedCollateral.toNumber(), this.valuation.liabilities.toNumber());
    }
    /** Compute the risk indicator using components from [[Valuation]] */
    computeRiskIndicator(requiredCollateral, weightedCollateral, liabilities) {
        if (requiredCollateral < 0)
            throw Error("requiredCollateral must be non-negative");
        if (weightedCollateral < 0)
            throw Error("weightedCollateral must be non-negative");
        if (liabilities < 0)
            throw Error("liabilities must be non-negative");
        if (weightedCollateral > 0) {
            return (requiredCollateral + liabilities) / weightedCollateral;
        }
        else if (requiredCollateral + liabilities > 0) {
            return Infinity;
        }
        else {
            return 0;
        }
    }
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
    static derive(programs, owner, seed) {
        if (seed > this.SEED_MAX_VALUE || seed < 0) {
            console.log(`Seed is not within the range: 0 <= seed <= ${this.SEED_MAX_VALUE}.`);
        }
        const buffer = Buffer.alloc(2);
        buffer.writeUInt16LE(seed);
        const marginAccount = (0, pda_1.findDerivedAccount)(programs.config.marginProgramId, owner, buffer);
        return marginAccount;
    }
    /**
     * Derive the address of a [[LiquidationData]] account.
     *
     * @param {Address} liquidator
     * @return {PublicKey}
     * @memberof MarginAccount
     */
    findLiquidationAddress(liquidator) {
        return (0, pda_1.findDerivedAccount)(this.programs.config.marginProgramId, this.address, liquidator);
    }
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
    findMetadataAddress(account) {
        const accountAddress = (0, anchor_1.translateAddress)(account);
        return (0, pda_1.findDerivedAccount)(this.programs.config.metadataProgramId, accountAddress);
    }
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
    findPositionTokenAddress(positionTokenMint) {
        const positionTokenMintAddress = (0, anchor_1.translateAddress)(positionTokenMint);
        return (0, pda_1.findDerivedAccount)(this.programs.config.marginProgramId, this.address, positionTokenMintAddress);
    }
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
    static async load({ programs, provider, pools, walletTokens, owner, seed }) {
        const marginAccount = new MarginAccount(programs, provider, owner, seed, pools, walletTokens);
        await marginAccount.refresh();
        return marginAccount;
    }
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
    static async loadAllByOwner({ programs, provider, pools, walletTokens, owner, filters }) {
        const ownerFilter = {
            memcmp: {
                offset: 16,
                bytes: owner.toString()
            }
        };
        filters ?? (filters = []);
        filters.push(ownerFilter);
        const infos = await programs.margin.account.marginAccount.all(filters);
        const marginAccounts = [];
        for (let i = 0; i < infos.length; i++) {
            const { account } = infos[i];
            const seed = (0, __1.bnToNumber)(new anchor_1.BN(account.userSeed, undefined, "le"));
            const marginAccount = new MarginAccount(programs, provider, account.owner, seed, pools, walletTokens);
            await marginAccount.refresh();
            marginAccounts.push(marginAccount);
        }
        return marginAccounts;
    }
    async refresh() {
        const marginAccount = await this.programs.margin.account.marginAccount.fetchNullable(this.address);
        const positions = marginAccount ? state_1.AccountPositionListLayout.decode(new Uint8Array(marginAccount.positions)) : null;
        if (!marginAccount || !positions) {
            this.info = undefined;
        }
        else {
            // Account is being liquidated
            let liquidationData = undefined;
            if (!marginAccount.liquidation.equals(web3_js_1.PublicKey.default)) {
                liquidationData =
                    (await this.programs.margin.account.liquidation.fetchNullable(marginAccount.liquidation)) ?? undefined;
            }
            this.info = {
                marginAccount,
                liquidationData,
                positions
            };
        }
        this.positions = this.getPositions();
        this.valuation = this.getValuation(true);
        this.poolPositions = this.getAllPoolPositions();
        this.summary = this.getSummary();
    }
    getAllPoolPositions() {
        const positions = {};
        const poolConfigs = Object.values(this.programs.config.tokens);
        for (let i = 0; i < poolConfigs.length; i++) {
            const poolConfig = poolConfigs[i];
            const tokenConfig = this.programs.config.tokens[poolConfig.symbol];
            const pool = this.pools?.[poolConfig.symbol];
            if (!pool?.info) {
                continue;
            }
            // Deposits
            const depositNotePosition = this.getPositionNullable(pool.addresses.depositNoteMint);
            const depositBalanceNotes = __1.Number192.from(depositNotePosition?.balance ?? new anchor_1.BN(0));
            const depositBalance = depositBalanceNotes.mul(pool.depositNoteExchangeRate()).toTokenAmount(pool.decimals);
            const depositValue = depositNotePosition?.value ?? 0;
            // Loans
            const loanNotePosition = this.getPositionNullable(pool.addresses.loanNoteMint);
            const loanBalanceNotes = __1.Number192.from(loanNotePosition?.balance ?? new anchor_1.BN(0));
            const loanBalance = loanBalanceNotes.mul(pool.loanNoteExchangeRate()).toTokenAmount(pool.decimals);
            const loanValue = loanNotePosition?.value ?? 0;
            // Max trade amounts
            const maxTradeAmounts = this.getMaxTradeAmounts(pool, depositBalance, loanBalance);
            // Minimum amount to deposit for the pool to end a liquidation
            const collateralWeight = depositNotePosition?.valueModifier ?? pool.depositNoteMetadata.valueModifier;
            const priceComponent = (0, __1.bigIntToBn)(pool.info.tokenPriceOracle.aggregate.priceComponent);
            const priceExponent = pool.info.tokenPriceOracle.exponent;
            const tokenPrice = number128_1.Number128.fromDecimal(priceComponent, priceExponent);
            const lamportPrice = tokenPrice.div(number128_1.Number128.fromDecimal(new anchor_1.BN(1), pool.decimals));
            const warningRiskLevel = number128_1.Number128.fromDecimal(new anchor_1.BN(MarginAccount.RISK_WARNING_LEVEL * 100000), -5);
            const liquidationEndingCollateral = (collateralWeight.isZero() || lamportPrice.isZero()
                ? number128_1.Number128.ZERO
                : this.valuation.requiredCollateral
                    .sub(this.valuation.effectiveCollateral.mul(warningRiskLevel))
                    .div(collateralWeight.mul(warningRiskLevel))
                    .div(lamportPrice)).toTokenAmount(pool.decimals);
            // Buying power
            // FIXME
            const buyingPower = __1.TokenAmount.zero(pool.decimals);
            positions[poolConfig.symbol] = {
                tokenConfig,
                pool,
                depositPosition: depositNotePosition,
                loanPosition: loanNotePosition,
                depositBalance,
                depositValue,
                loanBalance,
                loanValue,
                maxTradeAmounts,
                liquidationEndingCollateral,
                buyingPower
            };
        }
        return positions;
    }
    getMaxTradeAmounts(pool, depositBalance, loanBalance) {
        const zero = __1.TokenAmount.zero(pool.decimals);
        if (!pool.info) {
            return {
                deposit: zero,
                withdraw: zero,
                borrow: zero,
                repay: zero,
                repayFromDeposit: zero,
                swap: zero,
                transfer: zero
            };
        }
        // Wallet's balance for pool
        // If depsiting or repaying SOL, maximum input should consider fees
        let walletAmount = __1.TokenAmount.zero(pool.decimals);
        if (pool.symbol && this.walletTokens) {
            walletAmount = this.walletTokens.map[pool.symbol].amount;
        }
        if (pool.tokenMint.equals(spl_token_1.NATIVE_MINT)) {
            walletAmount = __1.TokenAmount.max(walletAmount.subb((0, __1.numberToBn)(pool_1.feesBuffer)), __1.TokenAmount.zero(pool.decimals));
        }
        // Max deposit
        const deposit = walletAmount;
        const priceExponent = pool.info.tokenPriceOracle.exponent;
        const priceComponent = (0, __1.bigIntToBn)(pool.info.tokenPriceOracle.aggregate.priceComponent);
        const tokenPrice = number128_1.Number128.fromDecimal(priceComponent, priceExponent);
        const lamportPrice = tokenPrice.div(number128_1.Number128.fromDecimal(new anchor_1.BN(1), pool.decimals));
        const depositNoteValueModifier = this.getPositionNullable(pool.addresses.depositNoteMint)?.valueModifier ?? pool.depositNoteMetadata.valueModifier;
        const loanNoteValueModifier = this.getPositionNullable(pool.addresses.loanNoteMint)?.valueModifier ?? pool.loanNoteMetadata.valueModifier;
        // Max withdraw
        let withdraw = this.valuation.availableSetupCollateral
            .div(depositNoteValueModifier)
            .div(lamportPrice)
            .toTokenAmount(pool.decimals);
        withdraw = __1.TokenAmount.min(withdraw, depositBalance);
        withdraw = __1.TokenAmount.min(withdraw, pool.vault);
        withdraw = __1.TokenAmount.max(withdraw, zero);
        // Max borrow
        let borrow = this.valuation.availableSetupCollateral
            .div(number128_1.Number128.ONE.add(number128_1.Number128.ONE.div(MarginAccount.SETUP_LEVERAGE_FRACTION.mul(loanNoteValueModifier))).sub(depositNoteValueModifier))
            .div(lamportPrice)
            .toTokenAmount(pool.decimals);
        borrow = __1.TokenAmount.min(borrow, pool.vault);
        borrow = __1.TokenAmount.max(borrow, zero);
        // Max repay
        const repay = __1.TokenAmount.min(loanBalance, walletAmount);
        const repayFromDeposit = __1.TokenAmount.min(loanBalance, depositBalance);
        // Max swap
        const swap = __1.TokenAmount.min(depositBalance.add(borrow), pool.vault);
        // Max transfer
        const transfer = withdraw;
        return {
            deposit,
            withdraw,
            borrow,
            repay,
            repayFromDeposit,
            swap,
            transfer
        };
    }
    getSummary() {
        let collateralValue = number128_1.Number128.ZERO;
        for (const position of this.positions) {
            const kind = position.kind;
            if (kind === state_1.PositionKind.Deposit) {
                collateralValue = collateralValue.add(position.valueRaw);
            }
        }
        const equity = collateralValue.sub(this.valuation.liabilities);
        const exposureNumber = this.valuation.liabilities.toNumber();
        const cRatio = exposureNumber === 0 ? Infinity : collateralValue.toNumber() / exposureNumber;
        const minCRatio = exposureNumber === 0 ? 1 : 1 + this.valuation.effectiveCollateral.toNumber() / exposureNumber;
        const depositedValue = collateralValue.toNumber();
        const borrowedValue = this.valuation.liabilities.toNumber();
        const accountBalance = equity.toNumber();
        let leverage = 1.0;
        if (this.valuation.liabilities.gt(number128_1.Number128.ZERO)) {
            if (equity.lt(number128_1.Number128.ZERO) || equity.eq(number128_1.Number128.ZERO)) {
                leverage = Infinity;
            }
            else {
                collateralValue.div(equity).toNumber();
            }
        }
        const availableCollateral = this.valuation.effectiveCollateral.sub(this.valuation.requiredCollateral).toNumber();
        return {
            depositedValue,
            borrowedValue,
            accountBalance,
            availableCollateral,
            leverage,
            cRatio,
            minCRatio
        };
    }
    /**
     * Get the array of regstered [[AccountPosition]] on this account
     *
     * @return {AccountPosition[]}
     * @memberof MarginAccount
     */
    getPositions() {
        return (this.info?.positions.positions ?? [])
            .filter(position => !position.address.equals(web3_js_1.PublicKey.default))
            .map(info => {
            const price = this.getPositionPrice(info.token);
            return new accountPosition_1.AccountPosition({ info, price });
        });
    }
    /**
     * Get the registerd [[AccountPosition]] associated with the position mint.
     * Throws an error if the position does not exist.
     *
     * @param {Address} mint The position mint. For example a [[Pool]] deposit note mint.
     * @return {(AccountPosition)}
     * @memberof MarginAccount
     */
    getPosition(mint) {
        const position = this.getPositionNullable(mint);
        (0, assert_1.default)(position);
        return position;
    }
    /**
     * Get the registerd [[AccountPosition]] associated with the position mint.
     *
     * @param {Address} mint The position mint. For example a [[Pool]] deposit note mint.
     * @return {(AccountPosition | undefined)}
     * @memberof MarginAccount
     */
    getPositionNullable(mint) {
        const mintAddress = (0, anchor_1.translateAddress)(mint);
        for (let i = 0; i < this.positions.length; i++) {
            const position = this.positions[i];
            if (position.token.equals(mintAddress)) {
                return position;
            }
        }
    }
    setPositionBalance(mint, account, balance) {
        const position = this.getPositionNullable(mint);
        if (!position || !position.address.equals(account)) {
            return;
        }
        position.setBalance(balance);
        return position;
    }
    getPositionPrice(mint) {
        // FIXME: make thiis more extensible
        let price;
        if (this.pools) {
            price = pool_1.Pool.getPrice(mint, Object.values(this.pools));
        }
        return price;
    }
    setPositionPrice(mint, price) {
        this.getPositionNullable(mint)?.setPrice(price);
    }
    /**
     * Check if the given address is an authority for this margin account.
     * The owner has authority, as well as a liquidator only during liquidation.
     */
    hasAuthority(authority) {
        return authority.equals(this.owner) || this.liquidator?.equals(authority);
    }
    getValuation(includeStalePositions) {
        const timestamp = (0, __1.getTimestamp)();
        let pastDue = false;
        let liabilities = number128_1.Number128.ZERO;
        let requiredCollateral = number128_1.Number128.ZERO;
        let requiredSetupCollateral = number128_1.Number128.ZERO;
        let weightedCollateral = number128_1.Number128.ZERO;
        const staleCollateralList = [];
        const claimErrorList = [];
        const constants = this.programs.margin.idl.constants;
        const MAX_PRICE_QUOTE_AGE = new anchor_1.BN(constants.find(constant => constant.name === "MAX_PRICE_QUOTE_AGE")?.value ?? 0);
        const POS_PRICE_VALID = 1;
        for (const position of this.positions) {
            const kind = position.kind;
            let staleReason;
            {
                const balanceAge = timestamp.sub(position.balanceTimestamp);
                const priceQuoteAge = timestamp.sub(position.priceRaw.timestamp);
                if (position.priceRaw.isValid != POS_PRICE_VALID) {
                    // collateral with bad prices
                    staleReason = state_1.ErrorCode.InvalidPrice;
                }
                else if (position.maxStaleness.gt(new anchor_1.BN(0)) && balanceAge.gt(position.maxStaleness)) {
                    // outdated balance
                    staleReason = state_1.ErrorCode.OutdatedBalance;
                }
                else if (priceQuoteAge.gt(MAX_PRICE_QUOTE_AGE)) {
                    staleReason = state_1.ErrorCode.OutdatedPrice;
                }
                else {
                    staleReason = undefined;
                }
            }
            if (kind === state_1.PositionKind.NoValue) {
                // Intentional
            }
            else if (kind === state_1.PositionKind.Claim) {
                if (staleReason === undefined || includeStalePositions) {
                    if (position.balance.gt(new anchor_1.BN(0)) &&
                        (position.flags & state_1.AdapterPositionFlags.PastDue) === state_1.AdapterPositionFlags.PastDue) {
                        pastDue = true;
                    }
                    liabilities = liabilities.add(position.valueRaw);
                    requiredCollateral = requiredCollateral.add(position.requiredCollateralValue());
                    requiredSetupCollateral = requiredSetupCollateral.add(position.requiredCollateralValue(MarginAccount.SETUP_LEVERAGE_FRACTION));
                }
                if (staleReason !== undefined) {
                    claimErrorList.push([position.token, staleReason]);
                }
            }
            else if (kind === state_1.PositionKind.Deposit) {
                if (staleReason === undefined || includeStalePositions) {
                    weightedCollateral = weightedCollateral.add(position.collateralValue());
                }
                if (staleReason !== undefined) {
                    staleCollateralList.push([position.token, staleReason]);
                }
            }
        }
        const effectiveCollateral = weightedCollateral.sub(liabilities);
        return {
            liabilities,
            pastDue,
            requiredCollateral,
            requiredSetupCollateral,
            weightedCollateral,
            effectiveCollateral,
            get availableCollateral() {
                return effectiveCollateral.sub(requiredCollateral);
            },
            get availableSetupCollateral() {
                return effectiveCollateral.sub(requiredSetupCollateral);
            },
            staleCollateralList,
            claimErrorList
        };
    }
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
    static async loadTokens(programs, owner) {
        const poolConfigs = Object.values(programs.config.tokens);
        const ownerAddress = (0, anchor_1.translateAddress)(owner);
        const all = await __1.AssociatedToken.loadMultipleOrNative({
            connection: programs.margin.provider.connection,
            owner: ownerAddress
        });
        // Build out the map
        const map = {};
        for (let i = 0; i < poolConfigs.length; i++) {
            const poolConfig = poolConfigs[i];
            const tokenConfig = programs.config.tokens[poolConfig.symbol];
            // Find the associated token pubkey
            const mint = (0, anchor_1.translateAddress)(poolConfig.mint);
            const associatedTokenOrNative = mint.equals(spl_token_1.NATIVE_MINT)
                ? ownerAddress
                : __1.AssociatedToken.derive(mint, ownerAddress);
            // Find the associated token from the loadMultiple query
            let token = all.find(token => token.address.equals(associatedTokenOrNative));
            if (token === undefined) {
                token = __1.AssociatedToken.zeroAux(associatedTokenOrNative, tokenConfig.decimals);
            }
            // Add it to the map
            map[poolConfig.symbol] = token;
        }
        return { all, map };
    }
    /**
     * Fetches the account and returns if it exists.
     *
     * @return {Promise<boolean>}
     * @memberof MarginAccount
     */
    static async exists(programs, owner, seed) {
        const ownerPubkey = (0, anchor_1.translateAddress)(owner);
        const marginAccount = this.derive(programs, ownerPubkey, seed);
        const info = await programs.margin.provider.connection.getAccountInfo(marginAccount);
        return !!info;
    }
    /**
     * Fetches the account and returns if it exists
     *
     * @return {Promise<boolean>}
     * @memberof MarginAccount
     */
    async exists() {
        return await MarginAccount.exists(this.programs, this.owner, this.seed);
    }
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
    static async createAccount({ programs, provider, owner, seed, pools, walletTokens }) {
        if (seed === undefined) {
            seed = await this.getUnusedAccountSeed({ programs, provider, owner });
        }
        const marginAccount = new MarginAccount(programs, provider, owner, seed, pools, walletTokens);
        await marginAccount.createAccount();
        return marginAccount;
    }
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
    static async getUnusedAccountSeed({ programs, provider, owner }) {
        let accounts = await MarginAccount.loadAllByOwner({ programs, provider, owner });
        accounts = accounts.sort((a, b) => a.seed - b.seed);
        // Return any gap found in account seeds
        for (let i = 0; i < accounts.length; i++) {
            const seed = accounts[i].seed;
            if (seed !== i) {
                return seed;
            }
        }
        // Return +1
        return accounts.length;
    }
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
    async createAccount() {
        const instructions = [];
        await this.withCreateAccount(instructions);
        if (instructions.length > 0) {
            await this.sendAndConfirm(instructions);
        }
    }
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
    async withCreateAccount(instructions) {
        if (!(await this.exists())) {
            const ix = await this.programs.margin.methods
                .createAccount(this.seed)
                .accounts({
                owner: this.owner,
                payer: this.provider.wallet.publicKey,
                marginAccount: this.address,
                systemProgram: web3_js_1.SystemProgram.programId
            })
                .instruction();
            instructions.push(ix);
        }
    }
    /**
     * Updates all position balances. `withUpdatePositionBalance` is often included
     * in transactions after modifying balances to synchronize with the margin account.
     *
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    async updateAllPositionBalances() {
        const instructions = [];
        await this.withUpdateAllPositionBalances({ instructions });
        return await this.sendAndConfirm(instructions);
    }
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
    async withUpdateAllPositionBalances({ instructions }) {
        for (const position of this.positions) {
            await this.withUpdatePositionBalance({ instructions, position: position.address });
        }
    }
    /**
     * Updates a single position balance. `withUpdatePositionBalance` is often included
     * in transactions after modifying balances to synchronize with the margin account.
     *
     * @param {{ position: AccountPosition }} { position }
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    async updatePositionBalance({ position }) {
        const instructions = [];
        await this.withUpdatePositionBalance({ instructions, position: position.address });
        return await this.sendAndConfirm(instructions);
    }
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
    async withUpdatePositionBalance({ instructions, position }) {
        const instruction = await this.programs.margin.methods
            .updatePositionBalance()
            .accounts({
            marginAccount: this.address,
            tokenAccount: position
        })
            .instruction();
        instructions.push(instruction);
    }
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
    async refreshPositionMetadata({ positionMint }) {
        const instructions = [];
        await this.withRefreshPositionMetadata({ instructions, positionMint });
        return await this.sendAndConfirm(instructions);
    }
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
    async withRefreshPositionMetadata({ instructions, positionMint }) {
        const metadata = this.findMetadataAddress(positionMint);
        const ix = await this.programs.margin.methods
            .refreshPositionMetadata()
            .accounts({
            marginAccount: this.address,
            metadata
        })
            .instruction();
        instructions.push(ix);
    }
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
    async getOrRegisterPosition(tokenMint) {
        (0, assert_1.default)(this.info);
        const tokenMintAddress = (0, anchor_1.translateAddress)(tokenMint);
        for (let i = 0; i < this.positions.length; i++) {
            const position = this.positions[i];
            if (position.token.equals(tokenMintAddress)) {
                return position.address;
            }
        }
        await this.registerPosition(tokenMintAddress);
        await this.refresh();
        for (let i = 0; i < this.positions.length; i++) {
            const position = this.positions[i];
            if (position.token.equals(tokenMintAddress)) {
                return position.address;
            }
        }
        throw new Error("Unable to register position.");
    }
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
    async withGetOrRegisterPosition({ instructions, positionTokenMint }) {
        const tokenMintAddress = (0, anchor_1.translateAddress)(positionTokenMint);
        const position = this.getPositionNullable(tokenMintAddress);
        if (position) {
            return position.address;
        }
        return await this.withRegisterPosition({ instructions, positionTokenMint: tokenMintAddress });
    }
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
    async registerPosition(tokenMint) {
        const positionTokenMint = (0, anchor_1.translateAddress)(tokenMint);
        const instructions = [];
        await this.withRegisterPosition({ instructions, positionTokenMint });
        return await this.sendAndConfirm(instructions);
    }
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
    async withRegisterPosition({ instructions, positionTokenMint }) {
        const tokenAccount = this.findPositionTokenAddress(positionTokenMint);
        const metadata = this.findMetadataAddress(positionTokenMint);
        const ix = await this.programs.margin.methods
            .registerPosition()
            .accounts({
            authority: this.owner,
            payer: this.provider.wallet.publicKey,
            marginAccount: this.address,
            positionTokenMint: positionTokenMint,
            metadata,
            tokenAccount,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            rent: web3_js_1.SYSVAR_RENT_PUBKEY,
            systemProgram: web3_js_1.SystemProgram.programId
        })
            .instruction();
        instructions.push(ix);
        return tokenAccount;
    }
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
    async closeAccount() {
        const ix = [];
        await this.withCloseAccount(ix);
        await this.sendAndConfirm(ix);
    }
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
    async withCloseAccount(instructions) {
        for (const position of this.getPositions()) {
            await this.withClosePosition(instructions, position);
        }
        const ix = await this.programs.margin.methods
            .closeAccount()
            .accounts({
            owner: this.owner,
            receiver: this.provider.wallet.publicKey,
            marginAccount: this.address
        })
            .instruction();
        instructions.push(ix);
    }
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
    async closePosition(position) {
        const ix = [];
        await this.withClosePosition(ix, position);
        await this.sendAndConfirm(ix);
    }
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
    async withClosePosition(instructions, position) {
        const ix = await this.programs.margin.methods
            .closePosition()
            .accounts({
            authority: this.owner,
            receiver: this.provider.wallet.publicKey,
            marginAccount: this.address,
            positionTokenMint: position.token,
            tokenAccount: position.address,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID
        })
            .instruction();
        instructions.push(ix);
    }
    /** @deprecated This has been renamed to `liquidateEnd` and will be removed in a future release. */
    async stopLiquidation() {
        return await this.liquidateEnd();
    }
    /**
     * Get instruction to end a liquidation
     * @deprecated This has been renamed to `withLiquidateEnd` and will be removed in a future release. */
    async withStopLiquidation(instructions) {
        return await this.withLiquidateEnd(instructions);
    }
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
    async liquidateEnd() {
        const ix = [];
        await this.withLiquidateEnd(ix);
        return await this.sendAndConfirm(ix);
    }
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
    async withLiquidateEnd(instructions) {
        const liquidation = this.info?.marginAccount.liquidation;
        const authority = this.provider.wallet.publicKey;
        (0, assert_1.default)(liquidation);
        (0, assert_1.default)(authority);
        const ix = await this.programs.margin.methods
            .liquidateEnd()
            .accounts({
            authority,
            marginAccount: this.address,
            liquidation
        })
            .instruction();
        instructions.push(ix);
    }
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
    getRemainingLiquidationTime() {
        const startTime = this.info?.liquidationData?.startTime?.toNumber();
        if (startTime === undefined) {
            return undefined;
        }
        const timeoutConstant = this.programs.margin.idl.constants.find(constant => constant.name === "LIQUIDATION_TIMEOUT");
        (0, assert_1.default)(timeoutConstant);
        const now = Date.now() / 1000;
        const elapsed = startTime - now;
        const timeout = parseFloat(timeoutConstant.value);
        const remaining = timeout - elapsed;
        return remaining;
    }
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
    async withAdapterInvoke({ instructions, adapterProgram, adapterMetadata, adapterInstruction }) {
        const ix = await this.programs.margin.methods
            .adapterInvoke(adapterInstruction.data)
            .accounts({
            owner: this.owner,
            marginAccount: this.address,
            adapterProgram,
            adapterMetadata
        })
            .remainingAccounts(this.invokeAccounts(adapterInstruction))
            .instruction();
        instructions.push(ix);
    }
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
    async withAccountingInvoke({ instructions, adapterProgram, adapterMetadata, adapterInstruction }) {
        const ix = await this.programs.margin.methods
            .accountingInvoke(adapterInstruction.data)
            .accounts({
            marginAccount: this.address,
            adapterProgram,
            adapterMetadata
        })
            .remainingAccounts(this.invokeAccounts(adapterInstruction))
            .instruction();
        instructions.push(ix);
    }
    /**
     * prepares arguments for `adapter_invoke`, `account_invoke`, or `liquidator_invoke`
     *
     * @return {AccountMeta[]} The instruction keys but the margin account is no longer a signer.
     * @memberof MarginAccount
     */
    invokeAccounts(adapterInstruction) {
        const accounts = [];
        for (const acc of adapterInstruction.keys) {
            let isSigner = acc.isSigner;
            if (acc.pubkey.equals(this.address)) {
                isSigner = false;
            }
            accounts.push({
                pubkey: acc.pubkey,
                isSigner: isSigner,
                isWritable: acc.isWritable
            });
        }
        return accounts;
    }
    /**
     * Sends a transaction using the [[MarginAccount]] [[AnchorProvider]]
     *
     * @param {TransactionInstruction[]} instructions
     * @param {Signer[]} [signers]
     * @return {Promise<string>}
     * @memberof MarginAccount
     */
    async sendAndConfirm(instructions, signers) {
        return await (0, __1.sendAndConfirm)(this.provider, instructions, signers);
    }
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
    async sendAll(transactions) {
        return await (0, __1.sendAll)(this.provider, transactions);
    }
}
exports.MarginAccount = MarginAccount;
/**
 * The maximum [[MarginAccount]] seed value equal to `65535`.
 * Seeds are a 16 bit number and therefor only 2^16 margin accounts may exist per wallet. */
MarginAccount.SEED_MAX_VALUE = 65535;
MarginAccount.RISK_WARNING_LEVEL = 0.8;
MarginAccount.RISK_CRITICAL_LEVEL = 0.9;
MarginAccount.RISK_LIQUIDATION_LEVEL = 1;
/** The maximum risk indicator allowed by the library when setting up a  */
MarginAccount.SETUP_LEVERAGE_FRACTION = number128_1.Number128.fromDecimal(new anchor_1.BN(50), -2);
//# sourceMappingURL=marginAccount.js.map