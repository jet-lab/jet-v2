import { Address, BN, translateAddress } from "@project-serum/anchor"
import { parsePriceData, PriceData, PriceStatus } from "@pythnetwork/client"
import { Mint, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL
} from "@solana/web3.js"
import { assert } from "chai"
import { AssociatedToken, bigIntToBn, numberToBn, TokenAddress, TokenFormat } from "../../token"
import { TokenAmount } from "../../token/tokenAmount"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { MarginPoolConfigData, MarginPoolData } from "./state"
import { MarginPoolConfig, MarginPools, MarginTokenConfig } from "../config"
import { PoolTokenChange } from "./poolTokenChange"
import { TokenMetadata } from "../metadata/state"
import { findDerivedAccount } from "../../utils/pda"
import { PriceInfo } from "../accountPosition"
import { chunks, Number128, Number192, sendAll, sleep } from "../../utils"
import { PositionTokenMetadata } from "../positionTokenMetadata"

export type PoolAction = "deposit" | "withdraw" | "borrow" | "repay" | "swap" | "transfer"

export interface MarginPoolAddresses {
  /** The pool's token mint i.e. BTC or SOL mint address*/
  tokenMint: PublicKey
  marginPool: PublicKey
  vault: PublicKey
  depositNoteMint: PublicKey
  loanNoteMint: PublicKey
  marginPoolAdapterMetadata: PublicKey
  tokenMetadata: PublicKey
  depositNoteMetadata: PublicKey
  loanNoteMetadata: PublicKey
  controlAuthority: PublicKey
}

export interface PriceResult {
  priceValue: Number192
  depositNotePrice: BN
  depositNoteConf: BN
  depositNoteTwap: BN
  loanNotePrice: BN
  loanNoteConf: BN
  loanNoteTwap: BN
}

export interface PoolProjection {
  riskIndicator: number
  depositRate: number
  borrowRate: number
}

const feesBuffer: number = LAMPORTS_PER_SOL * 0.02

export class Pool {
  address: PublicKey
  tokenMint: PublicKey
  depositNoteMetadata: PositionTokenMetadata
  loanNoteMetadata: PositionTokenMetadata

  get name(): string | undefined {
    return this.tokenConfig?.name
  }
  get symbol(): MarginPools | undefined {
    return this.poolConfig?.symbol
  }
  get vaultRaw(): Number192 {
    return Number192.fromDecimal(this.info?.vault.amount.lamports ?? new BN(0), 0)
  }
  get vault(): TokenAmount {
    return this.vaultRaw.asTokenAmount(this.decimals)
  }
  get borrowedTokensRaw() {
    if (!this.info) {
      return Number192.ZERO
    }
    return Number192.fromBits(this.info.marginPool.borrowedTokens)
  }
  get borrowedTokens(): TokenAmount {
    return this.borrowedTokensRaw.asTokenAmount(this.decimals)
  }
  get totalValueRaw(): Number192 {
    return this.borrowedTokensRaw.add(this.vaultRaw)
  }
  get totalValue(): TokenAmount {
    return this.totalValueRaw.asTokenAmount(this.decimals)
  }
  get uncollectedFeesRaw(): Number192 {
    if (!this.info) {
      return Number192.ZERO
    }
    return Number192.fromBits(this.info.marginPool.uncollectedFees)
  }
  get uncollectedFees(): TokenAmount {
    return this.uncollectedFeesRaw.asTokenAmount(this.decimals)
  }
  get utilizationRate(): number {
    return this.totalValue.tokens === 0 ? 0 : this.borrowedTokens.tokens / this.totalValue.tokens
  }
  get depositCcRate(): number {
    return this.info ? Pool.getCcRate(this.info.marginPool.config, this.utilizationRate) : 0
  }
  get depositApy(): number {
    return Pool.getDepositApy(this.depositCcRate, this.utilizationRate)
  }
  get borrowApr(): number {
    return Pool.getBorrowApr(this.depositCcRate, this.info?.marginPool.config.managementFeeRate ?? 0)
  }
  get tokenPrice(): number {
    return this.info?.tokenPriceOracle.price ?? 0
  }
  private _prices: PriceResult
  get depositNotePrice(): PriceInfo {
    return {
      value: this._prices.depositNotePrice,
      exponent: this.info?.tokenPriceOracle.exponent ?? 0,
      timestamp: bigIntToBn(this.info?.tokenPriceOracle.timestamp),
      isValid: Number(this.info ? this.info.tokenPriceOracle.status === PriceStatus.Trading : false)
    }
  }

  get loanNotePrice(): PriceInfo {
    return {
      value: this._prices.loanNotePrice,
      exponent: this.info?.tokenPriceOracle.exponent ?? 0,
      timestamp: bigIntToBn(this.info?.tokenPriceOracle.timestamp),
      isValid: Number(this.info ? this.info.tokenPriceOracle.status === PriceStatus.Trading : false)
    }
  }

  get decimals(): number {
    return this.tokenConfig?.decimals ?? this.info?.tokenMint.decimals ?? 0
  }
  get precision(): number {
    return this.tokenConfig?.precision ?? 0
  }

  public info?: {
    marginPool: MarginPoolData
    tokenMint: Mint
    vault: AssociatedToken
    depositNoteMint: Mint
    loanNoteMint: Mint
    tokenPriceOracle: PriceData
    tokenMetadata: TokenMetadata
  }
  /**
   * Creates a Margin Pool
   * @param programs
   * @param tokenMint
   * @param addresses
   * @param poolConfig
   * @param tokenConfig
   */
  constructor(
    public programs: MarginPrograms,
    public addresses: MarginPoolAddresses,
    public poolConfig?: MarginPoolConfig,
    public tokenConfig?: MarginTokenConfig
  ) {
    this.address = addresses.marginPool
    this.tokenMint = addresses.tokenMint
    this.depositNoteMetadata = new PositionTokenMetadata({ programs, tokenMint: addresses.depositNoteMint })
    this.loanNoteMetadata = new PositionTokenMetadata({ programs, tokenMint: addresses.loanNoteMint })
    this._prices = this.calculatePrices(this.info?.tokenPriceOracle)
  }

  async refresh() {
    const [
      marginPoolInfo,
      poolTokenMintInfo,
      vaultMintInfo,
      depositNoteMintInfo,
      loanNoteMintInfo,
      tokenMetadataInfo,
      depositNoteMetadataInfo,
      loanNoteMetadataInfo
    ] = await this.programs.marginPool.provider.connection.getMultipleAccountsInfo([
      this.addresses.marginPool,
      this.addresses.tokenMint,
      this.addresses.vault,
      this.addresses.depositNoteMint,
      this.addresses.loanNoteMint,
      this.addresses.tokenMetadata,
      this.addresses.depositNoteMetadata,
      this.addresses.loanNoteMetadata
    ])

    if (
      !marginPoolInfo ||
      !poolTokenMintInfo ||
      !vaultMintInfo ||
      !depositNoteMintInfo ||
      !loanNoteMintInfo ||
      !tokenMetadataInfo ||
      !depositNoteMetadataInfo ||
      !loanNoteMetadataInfo
    ) {
      this.info = undefined
    } else {
      const marginPool = this.programs.marginPool.coder.accounts.decode<MarginPoolData>(
        "marginPool",
        marginPoolInfo.data
      )
      const tokenMint = AssociatedToken.decodeMint(poolTokenMintInfo, this.addresses.tokenMint)
      const oracleInfo = await this.programs.marginPool.provider.connection.getAccountInfo(marginPool.tokenPriceOracle)
      if (!oracleInfo) {
        throw "Pyth oracle does not exist but a margin pool does. The margin pool is incorrectly configured."
      }
      this.info = {
        marginPool,
        tokenMint,
        vault: AssociatedToken.decodeAccount(vaultMintInfo, this.addresses.vault, tokenMint.decimals),
        depositNoteMint: AssociatedToken.decodeMint(depositNoteMintInfo, this.addresses.depositNoteMint),
        loanNoteMint: AssociatedToken.decodeMint(loanNoteMintInfo, this.addresses.loanNoteMint),
        tokenPriceOracle: parsePriceData(oracleInfo.data),
        tokenMetadata: this.programs.metadata.coder.accounts.decode<TokenMetadata>(
          "tokenMetadata",
          tokenMetadataInfo.data
        )
      }
    }

    this.depositNoteMetadata.decode(depositNoteMetadataInfo)
    this.loanNoteMetadata.decode(loanNoteMetadataInfo)
    this._prices = this.calculatePrices(this.info?.tokenPriceOracle)
  }

  /****************************
   * Program Implementation
   ****************************/

  calculatePrices(pythPrice: PriceData | undefined): PriceResult {
    if (
      !pythPrice ||
      pythPrice.status !== PriceStatus.Trading ||
      pythPrice.price === undefined ||
      pythPrice.confidence === undefined
    ) {
      const zero = new BN(0)
      return {
        priceValue: Number192.ZERO,
        depositNotePrice: zero,
        depositNoteConf: zero,
        depositNoteTwap: zero,
        loanNotePrice: zero,
        loanNoteConf: zero,
        loanNoteTwap: zero
      }
    }

    const priceValue = Number192.fromDecimal(bigIntToBn(pythPrice.aggregate.priceComponent), pythPrice.exponent)
    const confValue = Number192.fromDecimal(bigIntToBn(pythPrice.aggregate.confidenceComponent), pythPrice.exponent)
    const twapValue = Number192.fromDecimal(bigIntToBn(pythPrice.emaPrice.valueComponent), pythPrice.exponent)

    const depositNoteExchangeRate = this.depositNoteExchangeRate()
    const loanNoteExchangeRate = this.loanNoteExchangeRate()

    const depositNotePrice = priceValue.mul(depositNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    const depositNoteConf = confValue.mul(depositNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    const depositNoteTwap = twapValue.mul(depositNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    const loanNotePrice = priceValue.mul(loanNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    const loanNoteConf = confValue.mul(loanNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    const loanNoteTwap = twapValue.mul(loanNoteExchangeRate).asU64Rounded(pythPrice.exponent)
    return {
      priceValue,
      depositNotePrice,
      depositNoteConf,
      depositNoteTwap,
      loanNotePrice,
      loanNoteConf,
      loanNoteTwap
    }
  }

  depositNoteExchangeRate() {
    if (!this.info) {
      return Number192.ZERO
    }

    const depositNotes = BN.max(new BN(1), this.info.marginPool.depositNotes)
    const totalValue = Number192.max(Number192.ONE, this.totalValueRaw)
    return totalValue.sub(this.uncollectedFeesRaw).div(Number192.from(depositNotes))
  }

  loanNoteExchangeRate() {
    if (!this.info) {
      return Number192.ZERO
    }

    const loanNotes = BN.max(new BN(1), this.info.marginPool.loanNotes)
    const totalBorrowed = Number192.max(Number192.ONE, this.borrowedTokensRaw)
    return totalBorrowed.div(Number192.from(loanNotes))
  }

  /**
   * Linear interpolation between (x0, y0) and (x1, y1)
   * @param x
   * @param x0
   * @param x1
   * @param y0
   * @param y1
   * @returns
   */
  static interpolate = (x: number, x0: number, x1: number, y0: number, y1: number): number => {
    assert(x >= x0)
    assert(x <= x1)

    return y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
  }
  /**
   * Continous Compounding Rate
   * @param reserveConfig
   * @param utilRate
   * @returns
   */
  static getCcRate(reserveConfig: MarginPoolConfigData, utilRate: number): number {
    const basisPointFactor = 10000
    const util1 = reserveConfig.utilizationRate1 / basisPointFactor
    const util2 = reserveConfig.utilizationRate2 / basisPointFactor
    const borrow0 = reserveConfig.borrowRate0 / basisPointFactor
    const borrow1 = reserveConfig.borrowRate1 / basisPointFactor
    const borrow2 = reserveConfig.borrowRate2 / basisPointFactor
    const borrow3 = reserveConfig.borrowRate3 / basisPointFactor

    if (utilRate <= util1) {
      return this.interpolate(utilRate, 0, util1, borrow0, borrow1)
    } else if (utilRate <= util2) {
      return this.interpolate(utilRate, util1, util2, borrow1, borrow2)
    } else {
      return this.interpolate(utilRate, util2, 1, borrow2, borrow3)
    }
  }

  /** Borrow rate
   */
  static getBorrowApr(ccRate: number, fee: number): number {
    const basisPointFactor = 10000
    fee = fee / basisPointFactor
    const secondsPerYear: number = 365 * 24 * 60 * 60
    const rt = ccRate / secondsPerYear

    return Math.log1p((1 + fee) * Math.expm1(rt)) * secondsPerYear
  }

  /** Deposit rate
   */
  static getDepositApy(ccRate: number, utilRatio: number): number {
    const secondsPerYear: number = 365 * 24 * 60 * 60
    const rt = ccRate / secondsPerYear

    return Math.log1p(Math.expm1(rt)) * secondsPerYear * utilRatio
  }

  getPrice(mint: PublicKey) {
    if (mint.equals(this.addresses.depositNoteMint)) {
      return this.depositNotePrice
    } else if (mint.equals(this.addresses.loanNoteMint)) {
      return this.loanNotePrice
    }
  }

  static getPrice(mint: PublicKey, pools: Pool[]): PriceInfo | undefined {
    for (const pool of pools) {
      const price = pool.getPrice(mint)
      if (price) {
        return price
      }
    }
  }

  /****************************
   * Transactionss
   ****************************/

  async marginRefreshAllPositionPrices({ pools, marginAccount }: { pools: Pool[]; marginAccount: MarginAccount }) {
    const instructions: TransactionInstruction[] = []
    for (const pool of pools) {
      await pool.withMarginRefreshPositionPrice({ instructions, marginAccount })
    }
    await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async marginRefreshPositionPrice(marginAccount: MarginAccount) {
    const instructions: TransactionInstruction[] = []
    await this.withMarginRefreshPositionPrice({ instructions, marginAccount })
    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withMarginRefreshAllPositionPrices({
    instructions,
    pools,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    pools: Pool[]
    marginAccount: MarginAccount
  }) {
    for (const pool of pools) {
      await pool.withMarginRefreshPositionPrice({ instructions, marginAccount })
    }
  }

  async withMarginRefreshPositionPrice({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }): Promise<void> {
    assert(marginAccount)
    assert(this.info, "Must refresh the pool once.")
    await marginAccount.withAccountingInvoke({
      instructions: instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginRefreshPosition()
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          tokenPriceOracle: this.info.tokenMetadata.pythPrice
        })
        .instruction()
    })
  }

  /**
   * Transaction to deposit tokens into the pool
   *
   * @param `marginAccount` - The margin account that will receive the deposit.
   * @param `change` - The amount of tokens to be deposited in lamports.
   * @param `source` - (Optional) The token account that the deposit will be transfered from. The wallet balance or associated token account will be used if unspecified.
   */
  async deposit({
    marginAccount,
    change,
    source = TokenFormat.unwrappedSol
  }: {
    marginAccount: MarginAccount
    change: PoolTokenChange
    source?: TokenAddress
  }) {
    assert(marginAccount)
    assert(change)

    await marginAccount.createAccount()
    await sleep(2000)
    await marginAccount.refresh()

    const instructions: TransactionInstruction[] = []
    const position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(position)

    await this.withDeposit({
      instructions: instructions,
      marginAccount,
      source,
      destination: position,
      change
    })
    await marginAccount.withUpdatePositionBalance({ instructions, position })
    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withDeposit({
    instructions,
    marginAccount,
    source = TokenFormat.unwrappedSol,
    destination,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    source?: TokenAddress
    destination: Address
    change: PoolTokenChange
  }): Promise<void> {
    const provider = marginAccount.provider
    const mint = this.tokenMint

    const wrappedSource = await AssociatedToken.withBeginTransferFromSource({
      instructions,
      provider,
      mint,
      feesBuffer,
      source
    })

    const ix = await this.programs.marginPool.methods
      .deposit(change.changeKind.asParam(), change.value)
      .accounts({
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        depositor: marginAccount.owner,
        source: wrappedSource,
        destination,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)

    AssociatedToken.withEndTransfer({
      instructions,
      provider,
      mint,
      destination: source
    })
  }

  async marginBorrow({
    marginAccount,
    pools,
    change,
    destination
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    change: PoolTokenChange
    destination?: TokenAddress
  }) {
    await marginAccount.refresh()
    const refreshInstructions: TransactionInstruction[] = []
    const instructionsInstructions: TransactionInstruction[] = []

    const depositPosition = await marginAccount.withGetOrCreatePosition({
      positionTokenMint: this.addresses.depositNoteMint,
      instructions: refreshInstructions
    })
    assert(depositPosition)

    await this.withMarginRefreshAllPositionPrices({ instructions: refreshInstructions, pools, marginAccount })

    const loanNoteAccount = await this.withGetOrCreateLoanPosition(instructionsInstructions, marginAccount)

    await this.withMarginBorrow({
      instructions: instructionsInstructions,
      marginAccount,
      depositPosition,
      loanNoteAccount,
      change
    })

    if (destination !== undefined) {
      // The borrow will increase the deposits by some unknown amount.
      // To withdraw the full borrow, set the deposit amount to the previous known amount before borrowing
      const poolPosition = Object.values(marginAccount.poolPositions).find(
        position => position.pool && position.pool.address.equals(this.address)
      )
      assert(
        poolPosition,
        "Attempting to withdraw after borrowing, but can not find the pool position in the margin account to calculate the withdraw amount."
      )
      const previousDepositAmount = poolPosition.depositBalance
      const withdrawChange = PoolTokenChange.setTo(previousDepositAmount)
      await this.withWithdraw({
        instructions: instructionsInstructions,
        marginAccount,
        source: depositPosition,
        destination,
        change: withdrawChange
      })
    }

    await sendAll(marginAccount.provider, [chunks(11, refreshInstructions), instructionsInstructions])
  }

  async withGetOrCreateLoanPosition(
    instructions: TransactionInstruction[],
    marginAccount: MarginAccount
  ): Promise<Address> {
    const account = marginAccount.getPosition(this.addresses.loanNoteMint)
    if (account) {
      return account.address
    }
    return await this.withRegisterLoan(instructions, marginAccount)
  }

  async withGetOrCreateDepositNotePosition(
    instructions: TransactionInstruction[],
    marginAccount: MarginAccount
  ): Promise<Address> {
    const account = marginAccount.getPosition(this.addresses.depositNoteMint)
    if (account) {
      return account.address
    }
    return await marginAccount.withRegisterPosition(instructions, this.addresses.depositNoteMint)
  }

  /// Instruction to borrow tokens using a margin account
  ///
  /// # Params
  ///
  /// `instructions` - The array to append instuctions to.
  /// `marginAccount` - The account being borrowed against
  /// `deposit_account` - The account to receive the notes for the borrowed tokens
  /// `loan_account` - The account to receive the notes representing the debt
  /// `change` - The amount of tokens to be borrowed as a `PoolTokenChange`
  async withMarginBorrow({
    instructions,
    marginAccount,
    depositPosition,
    loanNoteAccount,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: Address
    loanNoteAccount: Address
    change: PoolTokenChange
  }): Promise<void> {
    assert(marginAccount)
    assert(depositPosition)
    assert(loanNoteAccount)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginBorrow(change.changeKind.asParam(), change.value)
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          depositNoteMint: this.addresses.depositNoteMint,
          loanAccount: loanNoteAccount,
          depositAccount: depositPosition,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })
  }

  /// Instruction to repay tokens owed by a margin account
  ///
  /// # Params
  ///
  /// `margin_scratch` - The scratch account for the margin system
  /// `margin_account` - The account with the loan to be repaid
  /// `deposit_account` - The account with notes to repay the loan
  /// `loan_account` - The account with the loan debt to be reduced
  /// `change` - The amount to be repaid
  async marginRepay({
    marginAccount,
    pools,
    source,
    change
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    source?: TokenAddress
    change: PoolTokenChange
  }) {
    await marginAccount.refresh()
    const refreshInstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []
    const depositPosition = await marginAccount.withGetOrCreatePosition({
      positionTokenMint: this.addresses.depositNoteMint,
      instructions: refreshInstructions
    })
    assert(depositPosition)

    await this.withMarginRefreshAllPositionPrices({ instructions: refreshInstructions, pools, marginAccount })

    const loanNoteAccount = await this.withGetOrCreateLoanPosition(instructions, marginAccount)

    if (source === undefined) {
      await this.withMarginRepay({
        instructions,
        marginAccount: marginAccount,
        depositPosition: depositPosition,
        loanPosition: loanNoteAccount,
        change
      })
    } else {
      await this.withRepay({
        instructions,
        marginAccount,
        depositPosition: depositPosition,
        loanPosition: loanNoteAccount,
        source,
        change,
        feesBuffer
      })
    }

    // Automatically close the position once the loan is repaid.
    // this doesn't work because it compares notes to tokens
    // let loanPosition = marginAccount.getPosition(this.addresses.loanNoteMint)
    // if (loanPosition && amount.value.eq(loanPosition.balance)) {
    //   await this.withCloseLoan(instructions, marginAccount)
    // }

    await sendAll(marginAccount.provider, [chunks(11, refreshInstructions), instructions])
  }

  async withMarginRepay({
    instructions,
    marginAccount,
    depositPosition,
    loanPosition,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: Address
    loanPosition: Address
    change: PoolTokenChange
  }): Promise<void> {
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginRepay(change.changeKind.asParam(), change.value)
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          depositNoteMint: this.addresses.depositNoteMint,
          loanAccount: loanPosition,
          depositAccount: depositPosition,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })
  }

  async withRepay({
    instructions,
    marginAccount,
    loanPosition,
    source,
    change,
    feesBuffer,
    sourceAuthority
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: Address
    loanPosition: Address
    source: TokenAddress
    change: PoolTokenChange
    feesBuffer: number
    sourceAuthority?: Address
  }): Promise<void> {
    const wrappedSource = await AssociatedToken.withBeginTransferFromSource({
      instructions,
      provider: marginAccount.provider,
      mint: this.tokenMint,
      source,
      feesBuffer
    })

    const ix = await this.programs.marginPool.methods
      .repay(change.changeKind.asParam(), change.value)
      .accounts({
        marginPool: this.address,
        loanNoteMint: this.addresses.loanNoteMint,
        vault: this.addresses.vault,
        loanAccount: loanPosition,
        repaymentTokenAccount: wrappedSource,
        repaymentAccountAuthority: sourceAuthority,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)

    await marginAccount.withUpdatePositionBalance({
      instructions,
      position: loanPosition
    })

    AssociatedToken.withEndTransfer({
      instructions,
      provider: marginAccount.provider,
      mint: this.tokenMint,
      destination: source
    })
  }

  /// Instruction to withdraw tokens from the pool.
  ///
  /// # Params
  ///
  /// `margin_account` - The margin account with the deposit to be withdrawn
  /// `change` - The amount to withdraw.
  /// `destination` - (Optional) The token account to send the withdrawn deposit
  async withdraw({
    marginAccount,
    pools,
    change,
    destination = TokenFormat.unwrappedSol
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    change: PoolTokenChange
    destination?: TokenAddress
  }) {
    // FIXME: can source be calculated in withdraw?
    const source = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)

    const preInstructions: TransactionInstruction[] = []
    const refreshInstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []

    await this.withMarginRefreshAllPositionPrices({ instructions: refreshInstructions, pools, marginAccount })
    await marginAccount.withUpdateAllPositionBalances({ instructions: refreshInstructions })
    await this.withWithdraw({
      instructions,
      marginAccount: marginAccount,
      source,
      destination,
      change
    })
    return await sendAll(marginAccount.provider, [preInstructions, chunks(11, refreshInstructions), instructions])
  }

  async withWithdraw({
    instructions,
    marginAccount,
    source,
    destination = TokenFormat.unwrappedSol,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    source: Address
    destination?: TokenAddress
    change: PoolTokenChange
  }): Promise<void> {
    const provider = marginAccount.provider
    const mint = this.tokenMint

    const withdrawDestination = await AssociatedToken.withBeginTransferToDestination({
      instructions,
      provider,
      mint,
      destination
    })

    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .withdraw(change.changeKind.asParam(), change.value)
        .accounts({
          depositor: marginAccount.address,
          marginPool: this.address,
          vault: this.addresses.vault,
          depositNoteMint: this.addresses.depositNoteMint,
          source,
          destination: withdrawDestination,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })

    AssociatedToken.withEndTransfer({
      instructions,
      provider,
      mint,
      destination
    })
  }

  async withRegisterLoan(instructions: TransactionInstruction[], marginAccount: MarginAccount): Promise<Address> {
    const loanNoteAccount = findDerivedAccount(
      this.programs.config.marginPoolProgramId,
      marginAccount.address,
      this.addresses.loanNoteMint
    )
    await marginAccount.withAdapterInvoke({
      instructions: instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .registerLoan()
        .accounts({
          marginAccount: marginAccount.address,
          positionTokenMetadata: this.addresses.loanNoteMetadata,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          loanNoteAccount: loanNoteAccount,
          payer: marginAccount.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .instruction()
    })
    await this.withMarginRefreshPositionPrice({ instructions, marginAccount })
    return loanNoteAccount
  }

  async withCloseLoan(instructions: TransactionInstruction[], marginAccount: MarginAccount) {
    const loanNoteAccount = findDerivedAccount(
      this.programs.config.marginPoolProgramId,
      marginAccount.address,
      this.addresses.loanNoteMint
    )
    await marginAccount.withAdapterInvoke({
      instructions: instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .closeLoan()
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          loanNoteAccount: loanNoteAccount,
          beneficiary: marginAccount.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })
  }

  // only closes deposit
  async closePosition({ marginAccount, destination }: { marginAccount: MarginAccount; destination: Address }) {
    await marginAccount.refresh()

    const position = marginAccount.getPosition(this.addresses.depositNoteMint)

    if (position) {
      if (position.balance.gt(new BN(0))) {
        const destinationAddress = translateAddress(destination)

        const isDestinationNative = AssociatedToken.isNative(marginAccount.owner, this.tokenMint, destinationAddress)

        let marginWithdrawDestination: PublicKey
        if (!isDestinationNative) {
          marginWithdrawDestination = destinationAddress
        } else {
          marginWithdrawDestination = AssociatedToken.derive(this.tokenMint, marginAccount.owner)
        }

        const instructions: TransactionInstruction[] = []

        await marginAccount.withUpdatePositionBalance({ instructions, position: position.address })

        await AssociatedToken.withCreate(instructions, marginAccount.provider, marginAccount.owner, this.tokenMint)
        await this.withWithdraw({
          instructions,
          marginAccount: marginAccount,
          source: position.address,
          destination: marginWithdrawDestination,
          change: PoolTokenChange.setTo(TokenAmount.zero(this.decimals))
        })

        if (isDestinationNative) {
          AssociatedToken.withClose(instructions, marginAccount.owner, this.tokenMint, destinationAddress)
        }

        await sendAll(marginAccount.provider, [instructions])
        await marginAccount.refresh()
      }

      await marginAccount.closePosition(position)
    }
  }

  projectAfterAction(marginAccount: MarginAccount, amount: TokenAmount, action: PoolAction): PoolProjection {
    switch (action) {
      case "deposit":
        return this.projectAfterDeposit(marginAccount, amount)
      case "withdraw":
        return this.projectAfterWithdraw(marginAccount, amount)
      case "borrow":
        return this.projectAfterBorrow(marginAccount, amount)
      case "repay":
        return this.projectAfterRepay(marginAccount, amount)
      default:
        throw new Error("Unknown pool action")
    }
  }

  /// Projects the deposit and borrow rates after a deposit into the pool.
  projectAfterDeposit(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    const borrowedTokens = this.borrowedTokens.tokens
    const totalValue = this.totalValue.add(amount).tokens

    const utilRatio = borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const depositNoteValueModifer = this.depositNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.add(
      amountValue.mul(depositNoteValueModifer)
    )
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral.div(effectiveCollateral).asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit and borrow rates after a withdrawal from the pool.
  projectAfterWithdraw(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    if (amount.tokens > this.vault.tokens) {
      throw "not enough tokens in the vault"
    }

    const borrowedTokens = this.borrowedTokens.tokens
    const totalValue = this.totalValue.sub(amount).tokens

    const utilRatio = totalValue === 0 ? 0 : borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const depositNoteValueModifer = this.depositNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.sub(
      amountValue.mul(depositNoteValueModifer)
    )
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral.div(effectiveCollateral).asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit and borrow rates after a borrow from the pool.
  projectAfterBorrow(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    const borrowedTokens = this.borrowedTokens.add(amount).tokens
    const totalValue = this.totalValue.add(amount).tokens

    const utilRatio = borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const loanNoteValueModifer = this.loanNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.sub(amountValue)
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral
          .add(amountValue.div(loanNoteValueModifer))
          .div(effectiveCollateral)
          .asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit and borrow rates after repaying a loan from the pool.
  projectAfterRepay(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    if (amount.tokens > this.borrowedTokens.tokens) {
      throw "not enough borrowed tokens"
    }

    const borrowedTokens = this.borrowedTokens.sub(amount).tokens
    const totalValue = this.totalValue.sub(amount).tokens

    const utilRatio = totalValue === 0 ? 0 : borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const loanNoteValueModifer = this.loanNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.add(amountValue)
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral
          .sub(amountValue.div(loanNoteValueModifer))
          .div(marginAccount.valuation.effectiveCollateral.add(amountValue))
          .asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit and borrow rates after repaying a loan from the pool.
  projectAfterRepayFromDeposit(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    if (amount.tokens > this.borrowedTokens.tokens) {
      throw "not enough borrowed tokens"
    }

    const borrowedTokens = this.borrowedTokens.sub(amount).tokens
    const totalValue = this.totalValue.sub(amount).sub(amount).tokens

    const utilRatio = totalValue === 0 ? 0 : borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const depositNoteValueModifer = this.depositNoteMetadata.valueModifier
    const loanNoteValueModifer = this.loanNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.add(
      amountValue.mul(Number128.ONE.sub(depositNoteValueModifer))
    )
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral
          .sub(amountValue.div(loanNoteValueModifer))
          .div(effectiveCollateral)
          .asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit and borrow rates after a borrow from the pool.
  projectAfterBorrowAndNotWithdraw(marginAccount: MarginAccount, amount: TokenAmount): PoolProjection {
    if (this.info == undefined) {
      throw "must have pool info initialised"
    }

    const borrowedTokens = this.borrowedTokens.add(amount).tokens
    const totalValue = this.totalValue.add(amount).add(amount).tokens

    const utilRatio = borrowedTokens / totalValue
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)

    const depositRate = Pool.getDepositApy(depositCcRate, utilRatio)
    const borrowRate = Pool.getBorrowApr(depositCcRate, utilRatio)

    const depositNoteValueModifer = this.depositNoteMetadata.valueModifier
    const loanNoteValueModifer = this.loanNoteMetadata.valueModifier
    const amountValue = Number128.from(numberToBn(amount.tokens * this._prices.priceValue.asNumber()))

    const effectiveCollateral = marginAccount.valuation.effectiveCollateral.sub(
      amountValue.mul(Number128.ONE.sub(depositNoteValueModifer))
    )
    const riskIndicator: number = !effectiveCollateral.isZero()
      ? marginAccount.valuation.requiredCollateral
          .add(amountValue.div(loanNoteValueModifer))
          .div(effectiveCollateral)
          .asNumber()
      : 0

    return { riskIndicator, depositRate, borrowRate }
  }
}
