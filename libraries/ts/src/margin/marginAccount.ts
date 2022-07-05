import assert from "assert"
import { Address, AnchorProvider, BN, ProgramAccount, translateAddress } from "@project-serum/anchor"
import { NATIVE_MINT, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  GetProgramAccountsFilter,
  MemcmpFilter,
  PublicKey,
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
  MarginAccountData,
  PositionKind
} from "./state"
import { MarginPrograms } from "./marginClient"
import { findDerivedAccount } from "../utils/pda"
import {
  AssociatedToken,
  bnToNumber,
  getTimestamp,
  MarginPools,
  Number128,
  Number192,
  numberToBn,
  PoolAmount,
  TokenAmount
} from ".."
import { MarginPoolConfig, MarginTokenConfig } from "./config"
import { sleep } from "../utils/util"
import { AccountPosition, PriceInfo } from "./accountPosition"

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

export type TradeAction = "deposit" | "withdraw" | "borrow" | "repay" | "swap" | "transfer"
export interface PoolPosition {
  poolConfig: MarginPoolConfig
  tokenConfig: MarginTokenConfig
  pool?: Pool
  depositNotePosition: AccountPosition | undefined
  loanNotePosition: AccountPosition | undefined
  depositBalance: TokenAmount
  depositBalanceNotes: BN
  depositValue: number
  loanBalance: TokenAmount
  loanBalanceNotes: BN
  loanValue: number
  maxTradeAmounts: Record<TradeAction, TokenAmount>
  buyingPower: TokenAmount
}

export interface AccountSummary {
  depositedValue: number
  borrowedValue: number
  accountBalance: number
  availableCollateral: number
  cRatio: number
  minCRatio: number
}

export interface Valuation {
  exposure: BN
  requiredCollateral: BN
  weightedCollateral: BN
  effectiveCollateral: BN
  availableCollateral: BN
  staleCollateralList: [PublicKey, ErrorCode][]
  pastDue: boolean
  claimErrorList: [PublicKey, ErrorCode][]
}

export interface Valuation {
  exposure: BN
  requiredCollateral: BN
  weightedCollateral: BN
  effectiveCollateral: BN
  availableCollateral: BN
  staleCollateralList: [PublicKey, ErrorCode][]
  pastDue: boolean
  claimErrorList: [PublicKey, ErrorCode][]
}

export interface MarginWalletTokens {
  all: AssociatedToken[]
  map: Record<MarginPools, AssociatedToken>
}

export class MarginAccount {
  static readonly SEED_MAX_VALUE = 65535
  info?: {
    marginAccount: MarginAccountData
    positions: AccountPositionList
  }

  addresses: MarginAccountAddresses
  positions: AccountPosition[]
  valuation: Valuation
  poolPositions: Record<MarginPools, PoolPosition>
  summary: AccountSummary

  get address() {
    return this.addresses.marginAccount
  }
  get owner() {
    return this.addresses.owner
  }
  get liquidator() {
    return this.info?.marginAccount.liquidator
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
    public pools?: Record<MarginPools, Pool>,
    public walletTokens?: MarginWalletTokens
  ) {
    this.pools = pools
    this.walletTokens = walletTokens
    this.addresses = MarginAccount.derive(programs, owner, seed)
    this.positions = this.getPositions()
    this.valuation = this.getValuation(true)
    this.poolPositions = this.getAllPoolPositions()
    this.summary = this.getSummary()
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

  static deriveTokenMetadata(programs: MarginPrograms, tokenMint: Address) {
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
    pools?: Record<MarginPools, Pool>
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
   *     pools?: Record<MarginPools, Pool>
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
    filters
  }: {
    programs: MarginPrograms
    provider: AnchorProvider
    pools?: Record<MarginPools, Pool>
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
    filters ??= []
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
      this.info = {
        marginAccount,
        positions
      }
    }
    this.positions = this.getPositions()
    this.valuation = this.getValuation(true)
    this.poolPositions = this.getAllPoolPositions()
    this.summary = this.getSummary()
  }

  getAllPoolPositions(): Record<MarginPools, PoolPosition> {
    const positions: Record<string, PoolPosition> = {}
    const poolConfigs = Object.values(this.programs.config.pools)

    for (let i = 0; i < poolConfigs.length; i++) {
      const poolConfig = poolConfigs[i]
      const tokenConfig = this.programs.config.tokens[poolConfig.symbol]
      const pool = this.pools?.[poolConfig.symbol]
      if (!pool) {
        continue
      }

      const totalValueLessFees = pool.depositedTokens.add(pool.borrowedTokens).sub(pool.uncollectedFees).lamports

      // Deposits
      const poolDepositNotes = pool.info?.marginPool.depositNotes ?? Number192.ZERO
      const depositNotePosition = this.getPosition(pool.addresses.depositNoteMint)
      const depositBalanceNotes = depositNotePosition?.balance ?? Number192.ZERO
      const depositTokenBalance = poolDepositNotes.isZero()
        ? Number192.ZERO
        : totalValueLessFees.mul(depositBalanceNotes).div(poolDepositNotes)
      const depositBalance = TokenAmount.lamports(depositTokenBalance, pool.decimals)
      const depositValue = depositNotePosition?.value ?? 0

      // Loans
      const poolLoanNotes = pool.info?.marginPool.loanNotes ?? Number192.ZERO
      const poolBorrowedTokens = pool.borrowedTokens.lamports
      const loanNotePosition = this.getPosition(pool.addresses.loanNoteMint)
      const loanBalanceNotes = loanNotePosition?.balance ?? Number192.ZERO
      const loanTokenBalance = poolLoanNotes.isZero()
        ? Number192.ZERO
        : poolBorrowedTokens.mul(loanBalanceNotes).div(poolLoanNotes)
      const loanBalance = TokenAmount.lamports(loanTokenBalance, pool.decimals)
      const loanValue = loanNotePosition?.value ?? 0

      // Max trade amounts
      const maxTradeAmounts = this.getMaxTradeAmounts(pool, depositBalance, loanBalance)

      // Buying power
      // FIXME
      const buyingPower = TokenAmount.zero(pool.decimals)

      positions[poolConfig.symbol] = {
        poolConfig,
        tokenConfig,
        pool,
        depositNotePosition,
        loanNotePosition,
        depositBalance,
        depositBalanceNotes,
        depositValue,
        loanBalance,
        loanBalanceNotes,
        loanValue,
        maxTradeAmounts,
        buyingPower
      }
    }

    return positions
  }

  getMaxTradeAmounts(
    pool: Pool,
    depositBalance: TokenAmount,
    loanBalance: TokenAmount
  ): Record<TradeAction, TokenAmount> {
    if (!pool.info) {
      const zero = TokenAmount.zero(pool.decimals)
      return {
        deposit: zero,
        withdraw: zero,
        borrow: zero,
        repay: zero,
        swap: zero,
        transfer: zero
      }
    }

    const walletAmount = pool.symbol && this.walletTokens?.map[pool.symbol].amount

    // Max deposit
    const deposit = walletAmount ?? TokenAmount.zero(pool.decimals)

    // Max withdraw
    const withdrawableLamports = pool.depositNoteMetadata
      .getRequiredCollateralValue(this.valuation.availableCollateral)
      .mul(Number128.ONE)
      .div(numberToBn(pool.tokenPrice * 10 ** Number128.PRECISION))

    let withdraw = TokenAmount.min(depositBalance, pool.borrowedTokens)
    withdraw = TokenAmount.min(withdraw, TokenAmount.lamports(Number128.asBn(withdrawableLamports, 0), pool.decimals))
    withdraw = TokenAmount.max(withdraw, TokenAmount.zero(pool.decimals))

    // Max borrow
    let borrowLamports = pool.loanNoteMetadata
      .getCollateralValue(this.valuation.availableCollateral)
      .mul(Number128.ONE)
      .div(numberToBn(pool.tokenPrice * 10 ** Number128.PRECISION))
    let borrow: TokenAmount = TokenAmount.min(
      TokenAmount.lamports(Number128.asBn(borrowLamports, 0), pool.decimals),
      pool.depositedTokens
    )
    borrow = TokenAmount.max(borrow, TokenAmount.zero(pool.decimals))

    // Max repay
    let repay = walletAmount ? TokenAmount.min(loanBalance, walletAmount) : loanBalance

    // Max swap
    const swap = withdraw

    // Max transfer
    const transfer = withdraw

    return {
      deposit,
      withdraw,
      borrow,
      repay,
      swap,
      transfer
    }
  }

  getSummary(): AccountSummary {
    let collateralValue = Number128.ZERO

    for (const position of this.positions) {
      let kind = position.kind
      if (kind === PositionKind.Deposit) {
        collateralValue = collateralValue.add(position.valueRaw)
      }
    }

    const exposureNumber = bnToNumber(this.valuation.exposure)
    const cRatio = exposureNumber === 0 ? Infinity : bnToNumber(collateralValue) / exposureNumber
    const minCRatio =
      exposureNumber === 0 ? Infinity : 1 + bnToNumber(this.valuation.effectiveCollateral) / exposureNumber
    const depositedValue = bnToNumber(Number128.asBn(collateralValue, -5)) / 100000
    const borrowedValue = bnToNumber(Number128.asBn(this.valuation.exposure, -5)) / 100000

    return {
      depositedValue,
      borrowedValue,
      accountBalance: depositedValue - borrowedValue,
      availableCollateral: 0, // FIXME: total collateral * collateral weight - total claims
      cRatio,
      minCRatio
    }
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
    // FIXME: make thiis more extensible
    let price: PriceInfo | undefined
    if (this.pools) {
      price = Pool.getPrice(mint, Object.values(this.pools))
    }
    return price
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
    let exposure = Number128.ZERO
    let requiredCollateral = Number128.ZERO
    let weightedCollateral = Number128.ZERO
    let staleCollateralList: [PublicKey, ErrorCode][] = []
    let claimErrorList: [PublicKey, ErrorCode][] = []

    const constants = this.programs.margin.idl.constants
    const MAX_PRICE_QUOTE_AGE = new BN(constants.find(constant => constant.name === "MAX_PRICE_QUOTE_AGE")?.value ?? 0)
    const POS_PRICE_VALID = 1

    for (const position of this.positions) {
      let kind = position.kind
      let staleReason: ErrorCode | undefined
      {
        let balanceAge = timestamp.sub(position.balanceTimestamp)
        let priceQuoteAge = timestamp.sub(position.price.timestamp)
        if (position.price.isValid != POS_PRICE_VALID) {
          // collateral with bad prices
          staleReason = ErrorCode.InvalidPrice
        } else if (position.maxStaleness.gt(Number128.ZERO) && balanceAge.gt(position.maxStaleness)) {
          // outdated balance
          staleReason = ErrorCode.OutdatedBalance
        } else if (priceQuoteAge.gt(MAX_PRICE_QUOTE_AGE)) {
          staleReason = ErrorCode.OutdatedPrice
        } else {
          staleReason = undefined
        }
      }

      if (kind === PositionKind.NoValue) {
      } else if (kind === PositionKind.Claim) {
        if (staleReason === undefined || includeStalePositions) {
          if (
            position.balance.gt(Number128.ZERO) &&
            (position.flags & AdapterPositionFlags.PastDue) === AdapterPositionFlags.PastDue
          ) {
            pastDue = true
          }

          exposure = exposure.add(new BN(position.valueRaw))
          requiredCollateral = requiredCollateral.add(position.requiredCollateralValue())
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

    const effectiveCollateral = weightedCollateral.sub(exposure)

    return {
      exposure,
      pastDue,
      requiredCollateral,
      weightedCollateral,
      effectiveCollateral,
      get availableCollateral(): BN {
        return effectiveCollateral.sub(requiredCollateral)
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
    const poolConfigs = Object.values(programs.config.pools)

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
      const mint = translateAddress(poolConfig.tokenMint)
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
    pools?: Record<MarginPools, Pool>
    walletTokens?: MarginWalletTokens
  }) {
    if (seed === undefined) {
      seed = await this.getUnusedAccountSeed({ programs, provider, owner })
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
  static async getUnusedAccountSeed({
    programs,
    provider,
    owner
  }: {
    programs: MarginPrograms
    provider: AnchorProvider
    owner: Address
  }) {
    let accounts = await MarginAccount.loadAllByOwner({ programs, provider, owner })
    accounts = accounts.sort((a, b) => a.seed - b.seed)
    // Return any gap found in account seeds
    for (let i = 0; i < accounts.length; i++) {
      const seed = accounts[i].seed
      if (seed !== i) {
        return seed
      }
    }

    // Return +1
    return accounts.length
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

  //Deposit
  /// Transaction to deposit tokens into a margin account
  ///
  /// # Params
  ///
  /// `token_mint` - The address of the mint for the tokens being deposited
  /// `source` - The token account that the deposit will be transfered from. Can also point to the wallet to automatically wrap SOL
  /// `amount` - The amount of tokens to deposit
  async deposit(marginPool: Pool, source: Address, amount: PoolAmount) {
    assert(marginPool)
    assert(source)
    assert(amount)

    await this.createAccount()
    await sleep(2000)
    await this.refresh()
    const position = await this.getOrCreatePosition(marginPool.addresses.depositNoteMint)
    assert(position)

    const instructions: TransactionInstruction[] = []
    source = await AssociatedToken.withWrapIfNativeMint(
      instructions,
      this.provider,
      this.provider.wallet.publicKey,
      marginPool.tokenMint,
      source,
      amount
    )

    await marginPool.withDeposit({
      instructions: instructions,
      depositor: this.owner,
      source,
      destination: position.address,
      amount
    })
    await this.withUpdatePositionBalance({ instructions, position })
    return await this.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  //TODO Withdraw
  async getOrCreatePosition(tokenMint: Address) {
    assert(this.info)
    const tokenMintAddress = translateAddress(tokenMint)

    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    await this.registerPosition(tokenMintAddress)
    await this.refresh()

    for (let i = 0; i < this.positions.length; i++) {
      const position = this.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    throw new Error("Unable to register position.")
  }

  async updateAllPositionBalances() {
    const instructions: TransactionInstruction[] = []
    await this.withUpdateAllPositionBalances({ instructions })
    await this.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withUpdateAllPositionBalances({ instructions }: { instructions: TransactionInstruction[] }) {
    for (const position of this.positions) {
      await this.withUpdatePositionBalance({ instructions, position })
    }
  }

  async updatePositionBalance({ position }: { position: AccountPosition }) {
    const instructions: TransactionInstruction[] = []
    await this.withUpdatePositionBalance({ instructions, position })
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
    position: AccountPosition
  }): Promise<void> {
    const instruction = await this.programs.margin.methods
      .updatePositionBalance()
      .accounts({
        marginAccount: this.address,
        tokenAccount: position.address
      })
      .instruction()
    instructions.push(instruction)
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
  async withRegisterPosition(instructions: TransactionInstruction[], tokenMint: Address): Promise<PublicKey> {
    const tokenAccount = findDerivedAccount(this.programs.config.marginProgramId, this.address, tokenMint)
    const metadata = findDerivedAccount(this.programs.config.metadataProgramId, tokenMint)

    const ix = await this.programs.margin.methods
      .registerPosition()
      .accounts({
        authority: this.owner,
        payer: this.provider.wallet.publicKey,
        marginAccount: this.address,
        positionTokenMint: tokenMint,
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
    try {
      return await this.provider.sendAndConfirm(new Transaction().add(...ix))
    } catch (err) {
      console.log(err)
      throw err
    }
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
    try {
      return await this.provider.sendAndConfirm(new Transaction().add(...ix))
    } catch (err) {
      console.log(err)
      throw err
    }
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
      .adapterInvoke(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return { isSigner: false, isWritable: accountMeta.isWritable }
        }),
        adapterInstruction.data
      )
      .accounts({
        owner: this.owner,
        marginAccount: this.address,
        adapterProgram,
        adapterMetadata
      })
      .remainingAccounts(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return {
            pubkey: accountMeta.pubkey,
            isSigner: false,
            isWritable: accountMeta.isWritable
          }
        })
      )
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
      .accountingInvoke(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return { isSigner: false, isWritable: accountMeta.isWritable }
        }),
        adapterInstruction.data
      )
      .accounts({
        marginAccount: this.address,
        adapterProgram,
        adapterMetadata
      })
      .remainingAccounts(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return {
            pubkey: accountMeta.pubkey,
            isSigner: false,
            isWritable: accountMeta.isWritable
          }
        })
      )
      .instruction()
    instructions.push(ix)
  }
}
