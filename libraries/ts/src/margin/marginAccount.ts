import assert from "assert"
import { Address, AnchorProvider, BN, ProgramAccount, translateAddress } from "@project-serum/anchor"
import { NATIVE_MINT, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  AccountMeta,
  GetProgramAccountsFilter,
  MemcmpFilter,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  TransactionSignature
} from "@solana/web3.js"
import { Pool } from "./pool/pool"
import {
  AccountPositionList,
  AccountPositionListLayout,
  AdapterPositionFlags,
  ErrorCode,
  LiquidationData,
  MarginAccountData,
  PositionKind
} from "./state"
import { MarginPrograms } from "./marginClient"
import { findDerivedAccount } from "../utils/pda"
import { AssociatedToken, bnToNumber, getTimestamp } from ".."
import { Number128 } from "../utils/number128"
import { AccountPosition, PriceInfo } from "./accountPosition"
import { PoolMarginAccount } from "./pool/poolMarginAccount"
import { IAdapter } from "./IAdapterClient"

export interface MarginAccountAddresses {
  marginAccount: PublicKey
  owner: PublicKey
}

export interface MarginPositionAddresses {
  account: PublicKey
  tokenAccount: PublicKey
  tokenMint: PublicKey
  tokenMetadata: PublicKey
}

export interface AccountSummary {
  depositedValue: number
  borrowedValue: number
  accountBalance: number
  /** @deprecated use riskIndicator */
  cRatio: number
  /** @deprecated use riskIndicator */
  minCRatio: number
}

export interface Valuation {
  liabilities: Number128
  requiredCollateral: Number128
  requiredSetupCollateral: Number128
  weightedCollateral: Number128
  effectiveCollateral: Number128
  availableCollateral: Number128
  availableSetupCollateral: Number128
  staleCollateralList: [PublicKey, ErrorCode][]
  pastDue: boolean
  claimErrorList: [PublicKey, ErrorCode][]
}

export interface MarginWalletTokens {
  all: AssociatedToken[]
  map: Record<string, AssociatedToken>
}

export class MarginAccount {
  static readonly SEED_MAX_VALUE = 65535
  static readonly RISK_WARNING_LEVEL = 0.9
  static readonly RISK_CRITICAL_LEVEL = 0.95
  static readonly RISK_LIQUIDATION_LEVEL = 1
  static readonly SETUP_LEVERAGE_FRACTION = Number128.fromDecimal(new BN(50), -2)

  info?: {
    marginAccount: MarginAccountData
    liquidationData?: LiquidationData
    positions: AccountPositionList
  }

  addresses: MarginAccountAddresses
  positions: AccountPosition[]
  valuation: Valuation
  poolPositions: PoolMarginAccount
  summary: AccountSummary

  adapters: IAdapter[] = []

  get address() {
    return this.addresses.marginAccount
  }
  get owner() {
    return this.addresses.owner
  }
  get liquidator() {
    return this.info?.marginAccount.liquidator
  }
  get liquidaton() {
    return this.info?.marginAccount.liquidation
  }
  get isBeingLiquidated() {
    return !this.info?.marginAccount.liquidation.equals(PublicKey.default)
  }
  /** A qualitative measure of the the health of a margin account.
   * A higher value means more risk in a qualitative sense.
   * Properties:
   *  non-negative, range is [0, infinity)
   *  zero only when an account has no exposure at all
   *  account is subject to liquidation at a value of one
   */
  get riskIndicator() {
    return this.computeRiskIndicator(
      this.valuation.requiredCollateral.asNumber(),
      this.valuation.weightedCollateral.asNumber(),
      this.valuation.liabilities.asNumber()
    )
  }

  /** A just-okay risk indicator (TODO improve me) */
  computeRiskIndicator(requiredCollateral: number, weightedCollateral: number, liabilities: number): number {
    if (requiredCollateral < 0) throw Error("requiredCollateral must be non-negative")
    if (weightedCollateral < 0) throw Error("weightedCollateral must be non-negative")
    if (liabilities < 0) throw Error("liabilities must be non-negative")

    if (weightedCollateral > 0) return (requiredCollateral + liabilities) / weightedCollateral
    else if (requiredCollateral + liabilities > 0) return Infinity
    else return 0
  }

  /**
   * Creates an instance of margin account.
   * @param {MarginPrograms} programs
   * @param {Provider} provider The provider and wallet that can sign for this margin account
   * @param {Address} owner
   * @param {number} seed
   * @memberof MarginAccount
   */
  constructor(
    public programs: MarginPrograms,
    public provider: AnchorProvider,
    owner: Address,
    public seed: number,
    public pools: Record<string, Pool>,
    public walletTokens?: MarginWalletTokens
  ) {
    this.addresses = MarginAccount.derive(programs, owner, seed)
    this.positions = []
    this.valuation = {
      liabilities: Number128.ZERO,
      pastDue: false,
      requiredCollateral: Number128.ZERO,
      requiredSetupCollateral: Number128.ZERO,
      weightedCollateral: Number128.ZERO,
      effectiveCollateral: Number128.ZERO,
      get availableCollateral(): Number128 {
        return Number128.ZERO
      },
      get availableSetupCollateral(): Number128 {
        return Number128.ZERO
      },
      staleCollateralList: [],
      claimErrorList: []
    }
    this.summary = {
      depositedValue: 0,
      borrowedValue: 0,
      accountBalance: 0,
      cRatio: 1,
      minCRatio: 1
    }
    this.poolPositions = new PoolMarginAccount(this, pools)
    this.adapters = [this.poolPositions]
  }

  /**
   * Derive margin account PDA from owner address and seed
   *
   * @private
   * @static
   * @param {Address} marginProgramId
   * @param {Address} owner
   * @param {number} seed
   * @return {PublicKey}
   * @memberof MarginAccount
   */
  static derive(programs: MarginPrograms, owner: Address, seed: number): MarginAccountAddresses {
    if (seed > this.SEED_MAX_VALUE || seed < 0) {
      console.log(`Seed is not within the range: 0 <= seed <= ${this.SEED_MAX_VALUE}.`)
    }
    const ownerAddress = translateAddress(owner)
    const buffer = Buffer.alloc(2)
    buffer.writeUInt16LE(seed)
    const marginAccount = findDerivedAccount(programs.config.marginProgramId, owner, buffer)

    return { marginAccount, owner: ownerAddress }
  }

  static deriveLiquidation(programs: MarginPrograms, marginAccount: MarginAccount, liquidator: Address) {
    return findDerivedAccount(programs.config.marginProgramId, marginAccount.address, liquidator)
  }

  static deriveMetadata(programs: MarginPrograms, tokenMint: Address) {
    const tokenMintAddress = translateAddress(tokenMint)
    return findDerivedAccount(programs.config.metadataProgramId, tokenMintAddress)
  }

  /**
   *
   * @param {MarginPrograms} programs
   * @param {AnchorProvider} provider The provider and wallet that can sign for this margin account
   * @param {Address} owner
   * @param {number} seed
   * @returns {Promise<MarginAccount>}
   */
  static async load({
    programs,
    provider,
    pools,
    walletTokens,
    owner,
    seed
  }: {
    programs: MarginPrograms
    provider: AnchorProvider
    pools: Record<string, Pool>
    walletTokens?: MarginWalletTokens
    owner: Address
    seed: number
  }): Promise<MarginAccount> {
    const marginAccount = new MarginAccount(programs, provider, owner, seed, pools, walletTokens)
    await marginAccount.refresh()
    return marginAccount
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
  static async loadAllByOwner({
    programs,
    provider,
    pools,
    walletTokens,
    owner,
    filters = []
  }: {
    programs: MarginPrograms
    provider: AnchorProvider
    pools: Record<string, Pool>
    walletTokens?: MarginWalletTokens
    owner: Address
    filters?: GetProgramAccountsFilter[]
  }): Promise<MarginAccount[]> {
    const ownerFilter: MemcmpFilter = {
      memcmp: {
        offset: 16,
        bytes: owner.toString()
      }
    }
    filters.push(ownerFilter)
    const infos: ProgramAccount<MarginAccountData>[] = await programs.margin.account.marginAccount.all(filters)
    const marginAccounts: MarginAccount[] = []
    for (let i = 0; i < infos.length; i++) {
      const { account } = infos[i]
      const seed = bnToNumber(new BN(account.userSeed, undefined, "le"))
      const marginAccount = new MarginAccount(programs, provider, account.owner, seed, pools, walletTokens)
      await marginAccount.refresh()
      marginAccounts.push(marginAccount)
    }
    return marginAccounts
  }

  async refresh() {
    const marginAccount = await this.programs.margin.account.marginAccount.fetchNullable(this.address)
    const positions = marginAccount ? AccountPositionListLayout.decode(new Uint8Array(marginAccount.positions)) : null
    if (!marginAccount || !positions) {
      this.info = undefined
    } else {
      // Account is being liquidated
      let liquidationData: LiquidationData | undefined = undefined
      if (!marginAccount.liquidation.equals(PublicKey.default)) {
        liquidationData =
          (await this.programs.margin.account.liquidation.fetchNullable(marginAccount.liquidation)) ?? undefined
      }
      this.info = {
        marginAccount,
        liquidationData,
        positions
      }
    }
    this.positions = this.getPositions()
    this.valuation = this.getValuation(true)
    this.summary = this.getSummary()

    this.poolPositions = new PoolMarginAccount(this, this.pools)
    this.adapters = [this.poolPositions]
  }

  getSummary(): AccountSummary {
    let collateralValue = Number128.ZERO

    for (const position of this.positions) {
      const kind = position.kind
      if (kind === PositionKind.Deposit) {
        collateralValue = collateralValue.add(position.valueRaw)
      }
    }

    const exposureNumber = this.valuation.liabilities.asNumber()
    const cRatio = exposureNumber === 0 ? Infinity : collateralValue.asNumber() / exposureNumber
    const minCRatio = exposureNumber === 0 ? 1 : 1 + this.valuation.effectiveCollateral.asNumber() / exposureNumber
    const depositedValue = collateralValue.asNumber()
    const borrowedValue = this.valuation.liabilities.asNumber()
    const accountBalance = collateralValue.sub(this.valuation.liabilities).asNumber()

    return {
      depositedValue,
      borrowedValue,
      accountBalance,
      cRatio,
      minCRatio
    }
  }

  getAdapter(adapterProgramId: Address): IAdapter | undefined {
    const adapterAddress = translateAddress(adapterProgramId)
    return this.adapters.find(adapter => adapter.adapterProgramId.equals(adapterAddress))
  }

  /** Get the list of positions on this account */
  getPositions() {
    return (this.info?.positions.positions ?? [])
      .filter(position => !position.address.equals(PublicKey.default))
      .map(info => {
        const price = this.getPositionPrice(info.token)
        return new AccountPosition({ info, price })
      })
  }

  getPosition(mint: Address) {
    const mintAddress = translateAddress(mint)

    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(mintAddress)) {
        return position
      }
    }
  }

  setPositionBalance(mint: PublicKey, account: PublicKey, balance: BN) {
    const position = this.getPosition(mint)

    if (!position || !position.address.equals(account)) {
      return
    }

    position.setBalance(balance)

    return position
  }

  getPositionPrice(mint: PublicKey) {
    for (const adapter of this.adapters) {
      const price = adapter.getPrice(mint)
      if (price) {
        return price
      }
    }
  }

  setPositionPrice(mint: PublicKey, price: PriceInfo) {
    this.getPosition(mint)?.setPrice(price)
  }

  /** Check if the given address is an authority for this margin account */
  hasAuthority(authority: PublicKey) {
    return authority.equals(this.owner) || this.liquidator?.equals(authority)
  }

  getValuation(includeStalePositions: boolean): Valuation {
    const timestamp = getTimestamp()

    let pastDue = false
    let liabilities = Number128.ZERO
    let requiredCollateral = Number128.ZERO
    let requiredSetupCollateral = Number128.ZERO
    let weightedCollateral = Number128.ZERO
    const staleCollateralList: [PublicKey, ErrorCode][] = []
    const claimErrorList: [PublicKey, ErrorCode][] = []

    const constants = this.programs.margin.idl.constants
    const MAX_PRICE_QUOTE_AGE = new BN(constants.find(constant => constant.name === "MAX_PRICE_QUOTE_AGE")?.value ?? 0)
    const POS_PRICE_VALID = 1

    for (const position of this.positions) {
      const kind = position.kind
      let staleReason: ErrorCode | undefined
      {
        const balanceAge = timestamp.sub(position.balanceTimestamp)
        const priceQuoteAge = timestamp.sub(position.price.timestamp)
        if (position.price.isValid != POS_PRICE_VALID) {
          // collateral with bad prices
          staleReason = ErrorCode.InvalidPrice
        } else if (position.maxStaleness.gt(new BN(0)) && balanceAge.gt(position.maxStaleness)) {
          // outdated balance
          staleReason = ErrorCode.OutdatedBalance
        } else if (priceQuoteAge.gt(MAX_PRICE_QUOTE_AGE)) {
          staleReason = ErrorCode.OutdatedPrice
        } else {
          staleReason = undefined
        }
      }

      if (kind === PositionKind.NoValue) {
        // FIXME
      } else if (kind === PositionKind.Claim) {
        if (staleReason === undefined || includeStalePositions) {
          if (
            position.balance.gt(new BN(0)) &&
            (position.flags & AdapterPositionFlags.PastDue) === AdapterPositionFlags.PastDue
          ) {
            pastDue = true
          }

          liabilities = liabilities.add(position.valueRaw)
          requiredCollateral = requiredCollateral.add(position.requiredCollateralValue())
          requiredSetupCollateral = requiredSetupCollateral.add(
            position.requiredCollateralValue(MarginAccount.SETUP_LEVERAGE_FRACTION)
          )
        }
        if (staleReason !== undefined) {
          claimErrorList.push([position.token, staleReason])
        }
      } else if (kind === PositionKind.Deposit) {
        if (staleReason === undefined || includeStalePositions) {
          weightedCollateral = weightedCollateral.add(position.collateralValue())
        }
        if (staleReason !== undefined) {
          staleCollateralList.push([position.token, staleReason])
        }
      }
    }

    const effectiveCollateral = weightedCollateral.sub(liabilities)

    return {
      liabilities,
      pastDue,
      requiredCollateral,
      requiredSetupCollateral,
      weightedCollateral,
      effectiveCollateral,
      get availableCollateral(): Number128 {
        return effectiveCollateral.sub(requiredCollateral)
      },
      get availableSetupCollateral(): Number128 {
        return effectiveCollateral.sub(requiredSetupCollateral)
      },
      staleCollateralList,
      claimErrorList
    }
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
  static async loadTokens(programs: MarginPrograms, owner: Address): Promise<MarginWalletTokens> {
    const poolConfigs = Object.values(programs.config.tokens)

    const ownerAddress = translateAddress(owner)

    const all = await AssociatedToken.loadMultipleOrNative({
      connection: programs.margin.provider.connection,
      owner: ownerAddress
    })

    // Build out the map
    const map: Record<string, AssociatedToken> = {}
    for (let i = 0; i < poolConfigs.length; i++) {
      const poolConfig = poolConfigs[i]
      const tokenConfig = programs.config.tokens[poolConfig.symbol]

      // Find the associated token pubkey
      const mint = translateAddress(poolConfig.mint)
      const associatedTokenOrNative = mint.equals(NATIVE_MINT)
        ? ownerAddress
        : AssociatedToken.derive(mint, ownerAddress)

      // Find the associated token from the loadMultiple query
      let token = all.find(token => token.address.equals(associatedTokenOrNative))
      if (token === undefined) {
        token = AssociatedToken.zeroAux(associatedTokenOrNative, tokenConfig.decimals)
      }

      // Add it to the map
      map[poolConfig.symbol] = token
    }
    return { all, map }
  }

  static async exists(programs: MarginPrograms, owner: Address, seed: number): Promise<boolean> {
    const ownerPubkey = translateAddress(owner)
    const { marginAccount } = this.derive(programs, ownerPubkey, seed)
    const info = await programs.margin.provider.connection.getAccountInfo(marginAccount)
    return !!info
  }

  async exists(): Promise<boolean> {
    return await MarginAccount.exists(this.programs, this.owner, this.seed)
  }

  /** Create the margin account. If no seed is provided, one will be located. */
  static async createAccount({
    programs,
    provider,
    owner,
    seed,
    pools,
    walletTokens
  }: {
    programs: MarginPrograms
    provider: AnchorProvider
    owner: Address
    seed?: number
    pools: Record<string, Pool>
    walletTokens?: MarginWalletTokens
  }) {
    if (seed === undefined) {
      seed = await this.getUnusedAccountSeed({ programs, owner })
    }
    const marginAccount = new MarginAccount(programs, provider, owner, seed, pools, walletTokens)
    await marginAccount.createAccount()
    return marginAccount
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
  static async getUnusedAccountSeed({ programs, owner }: { programs: MarginPrograms; owner: Address }) {
    const ownerFilter: MemcmpFilter = {
      memcmp: {
        offset: 16,
        bytes: owner.toString()
      }
    }
    const infos: ProgramAccount<MarginAccountData>[] = await programs.margin.account.marginAccount.all([ownerFilter])
    let seeds: number[] = []
    for (let i = 0; i < infos.length; i++) {
      const seed = bnToNumber(new BN(infos[i].account.userSeed, undefined, "le"))
      seeds.push(seed)
    }

    seeds = seeds.sort((a, b) => a - b)
    // Return any gap found in account seeds
    for (let i = 0; i < seeds.length; i++) {
      const seed = seeds[i]
      if (seed !== i) {
        return seed
      }
    }

    // Return +1
    return seeds.length
  }

  /** Create the margin account using it's owner and seed. */
  async createAccount() {
    const ix: TransactionInstruction[] = []
    await this.withCreateAccount(ix)
    if (ix.length > 0) {
      return await this.provider.sendAndConfirm(new Transaction().add(...ix))
    }
  }

  /** Get instruction to create the account */
  async withCreateAccount(instructions: TransactionInstruction[]): Promise<void> {
    if (!(await this.exists())) {
      const ix = await this.programs.margin.methods
        .createAccount(this.seed)
        .accounts({
          owner: this.owner,
          payer: this.provider.wallet.publicKey,
          marginAccount: this.address,
          systemProgram: SystemProgram.programId
        })
        .instruction()
      instructions.push(ix)
    }
  }

  async getOrCreatePosition(tokenMint: Address) {
    assert(this.info)
    const tokenMintAddress = translateAddress(tokenMint)
    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position.address
      }
    }
    await this.registerPosition(tokenMintAddress)
    await this.refresh()
    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position.address
      }
    }
    throw new Error("Unable to register position.")
  }

  async withGetOrCreatePosition({
    positionTokenMint,
    instructions
  }: {
    positionTokenMint: Address
    instructions: TransactionInstruction[]
  }) {
    const tokenMintAddress = translateAddress(positionTokenMint)
    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position.address
      }
    }

    return await this.withRegisterPosition(instructions, tokenMintAddress)
  }

  async updateAllPositionBalances() {
    const instructions: TransactionInstruction[] = []
    await this.withUpdateAllPositionBalances({ instructions })
    await this.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withUpdateAllPositionBalances({ instructions }: { instructions: TransactionInstruction[] }) {
    for (const position of this.positions) {
      await this.withUpdatePositionBalance({ instructions, position: position.address })
    }
  }

  async updatePositionBalance({ position }: { position: AccountPosition }) {
    const instructions: TransactionInstruction[] = []
    await this.withUpdatePositionBalance({ instructions, position: position.address })
    return await this.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  /// Get instruction to update the accounting for assets in
  /// the custody of the margin account.
  ///
  /// # Params
  ///
  /// `account` - The account address that has had a balance change
  async withUpdatePositionBalance({
    instructions,
    position
  }: {
    instructions: TransactionInstruction[]
    position: Address
  }): Promise<void> {
    const instruction = await this.programs.margin.methods
      .updatePositionBalance()
      .accounts({
        marginAccount: this.address,
        tokenAccount: position
      })
      .instruction()
    instructions.push(instruction)
  }

  async withRefreshAllPositions({ instructions }: { instructions: TransactionInstruction[] }) {
    for (const position of this.positions) {
      const adapter = this.getAdapter(position.adapter)
      if (adapter) {
        await adapter.withRefreshPosition(instructions, position.token)
      }
    }
  }

  async registerPosition(tokenMint: Address): Promise<TransactionSignature> {
    const tokenMintAddress = translateAddress(tokenMint)
    const ix: TransactionInstruction[] = []
    await this.withRegisterPosition(ix, tokenMintAddress)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to register new position
  ///
  /// # Params
  ///
  /// `token_mint` - The mint for the relevant token for the position
  /// `token_oracle` - The oracle account with price information on the token
  ///
  /// # Returns
  ///
  /// Returns the instruction, and the address of the token account to be
  /// created for the position.
  async withRegisterPosition(instructions: TransactionInstruction[], positionTokenMint: Address): Promise<PublicKey> {
    const tokenAccount = findDerivedAccount(this.programs.config.marginProgramId, this.address, positionTokenMint)
    const metadata = findDerivedAccount(this.programs.config.metadataProgramId, positionTokenMint)

    const ix = await this.programs.margin.methods
      .registerPosition()
      .accounts({
        authority: this.owner,
        payer: this.provider.wallet.publicKey,
        marginAccount: this.address,
        positionTokenMint: positionTokenMint,
        metadata,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId
      })
      .instruction()
    instructions.push(ix)
    return tokenAccount
  }

  async closeAccount() {
    const ix: TransactionInstruction[] = []
    await this.withCloseAccount(ix)
    await this.sendAndConfirm(ix)
  }

  /// Get instruction to close an account
  ///
  /// # Params
  ///
  async withCloseAccount(instructions: TransactionInstruction[]): Promise<void> {
    const ix = await this.programs.margin.methods
      .closeAccount()
      .accounts({
        owner: this.owner,
        receiver: this.provider.wallet.publicKey,
        marginAccount: this.address
      })
      .instruction()
    instructions.push(ix)
  }

  async closePosition(position: AccountPosition) {
    const ix: TransactionInstruction[] = []
    await this.withClosePosition(ix, position)
    await this.sendAndConfirm(ix)
  }

  /// Get instruction to close a position
  ///
  /// # Params
  ///
  /// `token_account` - The address of the token account for the position being closed
  async withClosePosition(instructions: TransactionInstruction[], position: AccountPosition): Promise<void> {
    //const authority = findDerivedAccount(this.programs.config.controlProgramId)

    const ix = await this.programs.margin.methods
      .closePosition()
      .accounts({
        authority: this.owner,
        receiver: this.provider.wallet.publicKey,
        marginAccount: this.address,
        positionTokenMint: position.token,
        tokenAccount: position.address,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)
  }

  async stopLiquidation() {
    const ix: TransactionInstruction[] = []
    await this.withStopLiquidation(ix)
    return await this.sendAndConfirm(ix)
  }

  /// Get instruction to close stop a liquidation
  ///
  /// # Params
  ///
  async withStopLiquidation(instructions: TransactionInstruction[]): Promise<void> {
    const ix = await this.programs.margin.methods
      .liquidateEnd()
      .accounts({
        authority: this.owner,
        marginAccount: this.address,
        liquidation: this.liquidaton
      })
      .instruction()
    instructions.push(ix)
  }

  // Get the remaining time on a liquidation
  getRemainingLiquidationTime() {
    if (this.info?.liquidationData) {
      return Date.now() / 1000 - bnToNumber(this.info.liquidationData.startTime)
    }
  }

  async withAdapterInvoke({
    instructions,
    adapterProgram,
    adapterMetadata,
    adapterInstruction
  }: {
    instructions: TransactionInstruction[]
    adapterProgram: Address
    adapterMetadata: Address
    adapterInstruction: TransactionInstruction
  }): Promise<void> {
    const ix = await this.programs.margin.methods
      .adapterInvoke(adapterInstruction.data)
      .accounts({
        owner: this.owner,
        marginAccount: this.address,
        adapterProgram,
        adapterMetadata
      })
      .remainingAccounts(this.invokeAccounts(adapterInstruction))
      .instruction()
    instructions.push(ix)
  }

  async withAccountingInvoke({
    instructions,
    adapterProgram,
    adapterMetadata,
    adapterInstruction
  }: {
    instructions: TransactionInstruction[]
    adapterProgram: Address
    adapterMetadata: Address
    adapterInstruction: TransactionInstruction
  }): Promise<void> {
    const ix = await this.programs.margin.methods
      .accountingInvoke(adapterInstruction.data)
      .accounts({
        marginAccount: this.address,
        adapterProgram,
        adapterMetadata
      })
      .remainingAccounts(this.invokeAccounts(adapterInstruction))
      .instruction()
    instructions.push(ix)
  }

  // prepares arguments for adapterInvoke, accountInvoke, or liquidatorInvoke
  invokeAccounts(adapterInstruction: TransactionInstruction): AccountMeta[] {
    const accounts: AccountMeta[] = []
    for (const acc of adapterInstruction.keys) {
      let isSigner = false
      if (acc.pubkey != this.address) {
        isSigner = acc.isSigner
      }
      accounts.push({
        pubkey: acc.pubkey,
        isSigner: isSigner,
        isWritable: acc.isWritable
      })
    }

    return accounts
  }

  async sendAndConfirm(instructions: TransactionInstruction[], signers?: Signer[]) {
    try {
      return await this.provider.sendAndConfirm(new Transaction().add(...instructions), signers)
    } catch (err) {
      console.log(err)
      throw err
    }
  }
}
