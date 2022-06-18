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
import { AccountPosition, AccountPositionList, AccountPositionListLayout, MarginAccountData } from "./state"
import { MarginPrograms } from "./marginClient"
import { findDerivedAccount } from "../utils/pda"
import { AssociatedToken, bnToNumber, MarginPools, TokenAmount, ZERO_BN, ONE_BN } from ".."
import { MarginPoolConfig, MarginTokenConfig } from "./config"
import { sleep } from "../utils/util"

export interface MarginAccountAddresses {
  marginAccount: PublicKey
  owner: PublicKey
  positions: Record<MarginPools, MarginPositionAddresses>
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
  depositNotePositionInfo: AccountPosition | undefined
  loanNotePositionInfo: AccountPosition | undefined
  depositBalance: TokenAmount
  depositBalanceNotes: BN
  loanBalance: TokenAmount
  loanBalanceNotes: BN
  maxTradeAmounts: Record<TradeAction, TokenAmount>
  buyingPower: TokenAmount
}

export interface AccountSummary {
  depositedValue: number
  borrowedValue: number
  accountBalance: number
  availableCollateral: number
  cRatio: number
  leverage: number
  totalBuyingPower: number
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

  positions: Record<MarginPools, PoolPosition>
  summary: AccountSummary
  addresses: MarginAccountAddresses

  get address() {
    return this.addresses.marginAccount
  }
  get owner() {
    return this.addresses.owner
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
    this.addresses = MarginAccount.derive(programs, owner, seed)
    this.positions = this.getAllPoolPositions()
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

    const poolConfigs = Object.values(programs.config.pools)
    const positionAddressesList: MarginPositionAddresses[] = poolConfigs.map(poolConfig => {
      const tokenMint = translateAddress(poolConfig.tokenMint)
      const account = findDerivedAccount(programs.config.marginProgramId, marginAccount, tokenMint)
      const tokenAccount = findDerivedAccount(programs.config.marginProgramId, marginAccount, tokenMint)
      const tokenMetadata = findDerivedAccount(programs.config.metadataProgramId, tokenMint)
      return {
        tokenMint,
        account,
        tokenAccount,
        tokenMetadata
      }
    })

    const positions: Record<string, MarginPositionAddresses> = {}
    for (let i = 0; i < positionAddressesList.length; i++) {
      positions[poolConfigs[i].symbol] = positionAddressesList[i]
    }
    return { marginAccount, owner: ownerAddress, positions }
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
    this.positions = this.getAllPoolPositions()
    this.summary = this.getSummary()
  }

  static getDepositNoteExchangeRate(
    depositNotes: BN,
    depositedTokens: BN,
    borrowedTokens: BN,
    uncollectedFees: BN
  ): BN {
    const totalValue = BN.max(ONE_BN, depositedTokens).add(borrowedTokens.mul(ONE_BN))
    return totalValue.sub(uncollectedFees).div(depositNotes.mul(ONE_BN))
  }

  static getLoanNoteExchangeRate(borrowNotes: BN, borrowedTokens: BN): BN {
    return BN.max(ONE_BN, borrowedTokens).div(BN.max(ONE_BN, borrowNotes.mul(ONE_BN)))
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

      // Exchange rates
      const depositNoteExchangeRate = MarginAccount.getDepositNoteExchangeRate(
        pool.info?.marginPool.depositNotes ?? ZERO_BN,
        pool.availableLiquidity.lamports,
        pool.outstandingDebt.lamports,
        ZERO_BN // TODO: add pool.uncollectedFees when merged to master
      )
      const loanNoteExchangeRate = MarginAccount.getLoanNoteExchangeRate(
        pool.info?.marginPool.loanNotes ?? ZERO_BN,
        pool.outstandingDebt.lamports
      )

      // Deposits
      const depositNotePositionInfo = this.info?.positions.positions.find(position =>
        position.token.equals(pool.addresses.depositNoteMint)
      )
      const depositBalanceNotes = depositNoteExchangeRate.mul(depositNotePositionInfo?.balance ?? ZERO_BN)
      const depositBalance = new TokenAmount(depositBalanceNotes.div(ONE_BN), pool?.decimals ?? 0)

      // Loans
      const loanNotePositionInfo = this.info?.positions.positions.find(position =>
        position.token.equals(pool.addresses.loanNoteMint)
      )
      const loanBalanceNotes = loanNoteExchangeRate.mul(loanNotePositionInfo?.balance ?? ZERO_BN)
      const loanBalance = new TokenAmount(loanBalanceNotes.div(ONE_BN), pool?.decimals ?? 0)

      // Max trade amounts
      const maxTradeAmounts = this.getMaxTradeAmounts(pool, depositBalance, loanBalance)

      // Buying power
      const buyingPower = depositBalance
        .muln(pool.tokenPrice)
        .muln(pool.maxLeverage)
        .sub(loanBalance.muln(pool.tokenPrice))

      positions[poolConfig.symbol] = {
        poolConfig,
        tokenConfig,
        pool,
        depositNotePositionInfo,
        loanNotePositionInfo,
        depositBalance,
        depositBalanceNotes,
        loanBalance,
        loanBalanceNotes,
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
    const depositedValue = depositBalance.muln(pool.tokenPrice)
    const loanValue = loanBalance.muln(pool.tokenPrice)

    // Max deposit
    const deposit =
      pool.symbol && this.walletTokens ? this.walletTokens.map[pool.symbol].amount : TokenAmount.zero(pool.decimals)

    // Max withdraw
    let withdraw = !loanValue.isZero()
      ? depositedValue.subn(pool.minCRatio * loanValue.tokens).divn(pool.tokenPrice)
      : depositBalance
    if (withdraw.gt(depositBalance)) {
      withdraw = depositBalance
    }
    if (withdraw.gt(pool.availableLiquidity)) {
      withdraw = pool.availableLiquidity
    }

    // Max borrow
    let borrow = depositedValue.divn(pool.minCRatio - loanValue.tokens).divn(pool.tokenPrice)
    if (borrow.gt(pool.availableLiquidity)) {
      borrow = pool.availableLiquidity
    }

    // Max repay
    let repay = loanBalance
    if (pool.symbol && this.walletTokens && this.walletTokens.map[pool.symbol].amount.lt(loanBalance)) {
      repay = this.walletTokens[pool.symbol].amount
    }

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
    let depositedValue = 0
    let borrowedValue = 0
    let totalBuyingPower = 0

    const positions = Object.values(this.positions)
    for (let i = 0; i < positions.length; i++) {
      const position = positions[i]
      depositedValue += position.depositBalance.tokens * (position.pool?.tokenPrice ?? 0)
      borrowedValue += position.loanBalance.tokens * (position.pool?.tokenPrice ?? 0)
      totalBuyingPower += position.buyingPower.tokens
    }

    return {
      depositedValue,
      borrowedValue,
      accountBalance: depositedValue - borrowedValue,
      availableCollateral: 0, // FIXME: total collateral * collateral weight - total claims
      cRatio: depositedValue / borrowedValue,
      leverage: borrowedValue / depositedValue,
      totalBuyingPower
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
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
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
  /// `source` - The token account that the deposit will be transfered from
  /// `amount` - The amount of tokens to deposit
  async deposit(marginPool: Pool, source: Address, amount: BN) {
    await this.createAccount()
    await sleep(2000)
    await this.refresh()
    const position = await this.getOrCreatePosition(marginPool.addresses.depositNoteMint)
    assert(position)

    const ix: TransactionInstruction[] = []
    await marginPool.withDeposit({
      instructions: ix,
      depositor: this.owner,
      source,
      destination: position.address,
      amount
    })
    await this.withUpdatePositionBalance(ix, position.address)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  //TODO Withdraw
  async getOrCreatePosition(tokenMint: Address) {
    assert(this.info)
    const tokenMintAddress = translateAddress(tokenMint)

    for (let i = 0; i < this.info.positions.length.toNumber(); i++) {
      const position = this.info.positions.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    await this.registerPosition(tokenMintAddress)
    await this.refresh()

    for (let i = 0; i < this.info.positions.length.toNumber(); i++) {
      const position = this.info.positions.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    throw new Error("Unable to register position.")
  }

  async getTokenMetadata(tokenMint: Address) {
    const metadataAddress = MarginAccount.deriveTokenMetadata(this.programs, tokenMint)
    return await this.programs.metadata.account.tokenMetadata.fetch(metadataAddress)
  }

  async updatePositionBalance(account: PublicKey) {
    const ix: TransactionInstruction[] = []
    await this.withUpdatePositionBalance(ix, account)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to update the accounting for assets in
  /// the custody of the margin account.
  ///
  /// # Params
  ///
  /// `account` - The account address that has had a balance change
  async withUpdatePositionBalance(instructions: TransactionInstruction[], account: PublicKey): Promise<void> {
    const ix = await this.programs.margin.methods
      .updatePositionBalance()
      .accounts({
        marginAccount: this.address,
        tokenAccount: account
      })
      .instruction()
    instructions.push(ix)
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

  async closePosition(tokenAccount: PublicKey) {
    const ix: TransactionInstruction[] = []
    await this.withClosePosition(ix, tokenAccount)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to close a position
  ///
  /// # Params
  ///
  /// `token_account` - The address of the token account for the position being closed
  async withClosePosition(instructions: TransactionInstruction[], tokenAccount: PublicKey): Promise<void> {
    const authority = findDerivedAccount(this.programs.config.controlProgramId)

    const ix = await this.programs.margin.methods
      .closePosition()
      .accounts({
        authority: authority,
        receiver: this.provider.wallet.publicKey,
        marginAccount: this.address,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)
  }
}
