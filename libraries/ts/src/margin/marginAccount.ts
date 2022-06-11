import assert from "assert"
import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { AccountLayout, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  Connection,
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
import { AssociatedToken, MarginPools, ZERO_BN } from ".."
import { MarginPoolConfig, MarginTokenConfig } from "./config"

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

export interface PoolPosition {
  poolConfig: MarginPoolConfig
  tokenConfig: MarginTokenConfig
  pool?: Pool
  depositNotePositionInfo: AccountPosition | undefined
  loanNotePositionInfo: AccountPosition | undefined
  depositBalance: number
  depositBalanceNotes: BN
  loanBalance: number
  loanBalanceNotes: BN
  maxDepositAmount: number
  maxWithdrawAmount: number
  maxBorrowAmount: number
  maxRepayAmount: number
  maxSwapAmount: number
  maxTransferAmount: number
  buyingPower: number
}

export interface AccountSummary {
  depositedValue: number
  borrowedValue: number
  accountBalance: number
  availableCollateral: number
  cRatio: number
  utilizationRate: number
  leverage: number
}

export interface MarginWalletTokens {
  all: AssociatedToken[]
  map: Record<MarginPools, AssociatedToken>
}

export class MarginAccount {
  static readonly SEED_MAX_VALUE = 65535
  public info?: {
    marginAccount: MarginAccountData
    positions: AccountPositionList
  }

  positions: Record<MarginPools, PoolPosition>
  summary: AccountSummary

  public addresses: MarginAccountAddresses
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

  getAllPoolPositions(): Record<MarginPools, PoolPosition> {
    const positions: Record<string, PoolPosition> = {}
    const poolConfigs = Object.values(this.programs.config.pools)

    for (let i = 0; i < poolConfigs.length; i++) {
      const poolConfig = poolConfigs[i]
      const tokenConfig = this.programs.config.tokens[poolConfig.symbol]
      const pool = this.pools?.[poolConfig.symbol]
      const depositNotePositionInfo =
        pool && this.info?.positions.positions.find(position => position.token.equals(pool.addresses.depositNoteMint))
      const loanNotePositionInfo =
        pool && this.info?.positions.positions.find(position => position.token.equals(pool.addresses.loanNoteMint))

      // FIXME: Calculate these fields. Stop using infinity
      positions[poolConfig.symbol] = {
        poolConfig,
        tokenConfig,
        pool,
        depositNotePositionInfo,
        loanNotePositionInfo,
        depositBalance: Infinity,
        depositBalanceNotes: depositNotePositionInfo?.balance ?? ZERO_BN,
        loanBalance: Infinity,
        loanBalanceNotes: loanNotePositionInfo?.balance ?? ZERO_BN,
        maxDepositAmount: Infinity,
        maxWithdrawAmount: Infinity,
        maxBorrowAmount: Infinity,
        maxRepayAmount: Infinity,
        maxSwapAmount: Infinity,
        maxTransferAmount: Infinity,
        buyingPower: Infinity
      }
    }

    return positions
  }

  getSummary(): AccountSummary {
    let depositedValue = 0
    let borrowedValue = 0

    const positions = Object.values(this.positions)
    for (let i = 0; i < positions.length; i++) {
      const position = positions[i]
      depositedValue += position.depositBalance
      borrowedValue += position.loanBalance
    }

    return {
      depositedValue,
      borrowedValue,
      accountBalance: depositedValue - borrowedValue,

      // FIXME
      availableCollateral: 0,
      cRatio: 0,
      utilizationRate: 0,
      leverage: 0
    }
  }

  static async loadTokens(programs: MarginPrograms, owner: Address): Promise<MarginWalletTokens> {
    const poolConfigs = Object.values(programs.config.pools)

    const all = await AssociatedToken.loadMultipleOrNative({ connection: programs.margin.provider.connection, owner })

    const map: Record<string, AssociatedToken> = {}
    for (let i = 0; i < all.length; i++) {
      map[poolConfigs[i].symbol] = all[i]
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
    const tokenAccount = await this.withRegisterPosition(ix, tokenMintAddress)
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

  static async getTokenAccountInfo(connection: Connection, address: PublicKey) {
    const info = await connection.getAccountInfo(address)
    return AccountLayout.decode(Buffer.from(info!.data))
  }
}
