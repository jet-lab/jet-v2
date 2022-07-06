import { Address, BN, translateAddress } from "@project-serum/anchor"
import { parsePriceData, PriceData, PriceStatus } from "@pythnetwork/client"
import { Mint, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js"
import { assert } from "chai"
import { AssociatedToken, bigIntToBn } from "../../token"
import { TokenAmount } from "../../token/tokenAmount"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { MarginPoolConfigData, MarginPoolData } from "./state"
import { MarginPoolConfig, MarginPools, MarginTokenConfig } from "../config"
import { PoolTokenChange } from "./poolTokenChange"
import { TokenMetadata } from "../metadata/state"
import { AccountPosition, PriceInfo } from "../accountPosition"
import { Number192, sleep } from "../../utils"
import { PositionTokenMetadata } from "../positionTokenMetadata"

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
  tokenPrice: number
  depositNotePrice: BN
  depositNoteConf: BN
  depositNoteTwap: BN
  loanNotePrice: BN
  loanNoteConf: BN
  loanNoteTwap: BN
}

export class Pool {
  address: PublicKey
  depositNoteMetadata: PositionTokenMetadata
  loanNoteMetadata: PositionTokenMetadata

  get name(): string | undefined {
    return this.tokenConfig?.name
  }
  get symbol(): MarginPools | undefined {
    return this.poolConfig?.symbol
  }
  get depositedTokens(): TokenAmount {
    return this.info?.vault.amount ?? TokenAmount.zero(this.decimals)
  }
  get borrowedTokensRaw(): BN {
    if (!this.info) {
      return Number192.ZERO
    }
    return new BN(this.info.marginPool.borrowedTokens, "le")
  }
  get borrowedTokens(): TokenAmount {
    if (!this.info) {
      return TokenAmount.zero(this.decimals)
    }
    const lamports = this.borrowedTokensRaw.div(Number192.ONE)
    return TokenAmount.lamports(lamports, this.decimals)
  }
  get totalValueRaw(): BN {
    return this.borrowedTokensRaw.add(Number192.from(this.depositedTokens.lamports))
  }
  get totalValue(): TokenAmount {
    return TokenAmount.lamports(this.totalValueRaw.div(Number192.ONE), this.decimals)
  }
  get uncollectedFeesRaw(): BN {
    return this.info ? new BN(this.info.marginPool.uncollectedFees, "le") : Number192.ZERO
  }
  get uncollectedFees(): TokenAmount {
    if (!this.info) {
      return TokenAmount.zero(this.decimals)
    }
    const lamports = this.uncollectedFeesRaw.div(Number192.ONE)
    return TokenAmount.lamports(lamports, this.decimals)
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
    public tokenMint: Address,
    public addresses: MarginPoolAddresses,
    public poolConfig?: MarginPoolConfig,
    public tokenConfig?: MarginTokenConfig
  ) {
    this.address = addresses.marginPool
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
      return {
        tokenPrice: 0,
        depositNotePrice: Number192.ZERO,
        depositNoteConf: Number192.ZERO,
        depositNoteTwap: Number192.ZERO,
        loanNotePrice: Number192.ZERO,
        loanNoteConf: Number192.ZERO,
        loanNoteTwap: Number192.ZERO
      }
    }

    const priceValue = Number192.fromDecimal(bigIntToBn(pythPrice.aggregate.priceComponent), pythPrice.exponent)
    const confValue = Number192.fromDecimal(bigIntToBn(pythPrice.aggregate.confidenceComponent), pythPrice.exponent)
    const twapValue = Number192.fromDecimal(bigIntToBn(pythPrice.emaPrice.valueComponent), pythPrice.exponent)

    const depositNoteExchangeRate = this.depositNoteExchangeRate()
    const loanNoteExchangeRate = this.loanNoteExchangeRate()

    const depositNotePrice = Number192.asU64Rounded(
      priceValue.mul(depositNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    const depositNoteConf = Number192.asU64Rounded(
      confValue.mul(depositNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    const depositNoteTwap = Number192.asU64Rounded(
      twapValue.mul(depositNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    const loanNotePrice = Number192.asU64Rounded(
      priceValue.mul(loanNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    const loanNoteConf = Number192.asU64Rounded(
      confValue.mul(loanNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    const loanNoteTwap = Number192.asU64Rounded(
      twapValue.mul(loanNoteExchangeRate).div(Number192.ONE),
      pythPrice.exponent
    )
    return {
      tokenPrice: pythPrice.price,
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
      return new BN(0)
    }

    const one = new BN(1)
    const depositNotes = BN.max(one, this.info.marginPool.depositNotes)
    const totalValue = BN.max(Number192.ONE, this.totalValueRaw)
    return totalValue.sub(this.uncollectedFeesRaw).mul(Number192.ONE).div(Number192.from(depositNotes))
  }

  loanNoteExchangeRate() {
    if (!this.info) {
      return new BN(0)
    }

    const one = new BN(1)
    const loanNotes = BN.max(one, this.info.marginPool.loanNotes)
    const totalBorrowed = BN.max(Number192.ONE, this.borrowedTokensRaw)
    return totalBorrowed.mul(Number192.ONE).div(Number192.from(loanNotes))
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
  async deposit({ marginAccount, change, source }: { marginAccount: MarginAccount; change: PoolTokenChange; source?: Address }) {
    assert(marginAccount)
    assert(change)

    await marginAccount.createAccount()
    await sleep(2000)
    await marginAccount.refresh()
    const position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(position)

    const instructions: TransactionInstruction[] = []
    source ??= await AssociatedToken.withCreateOrWrapIfNativeMint(
      instructions,
      marginAccount.provider,
      this.tokenMint,
      change.value
    )

    await this.withDeposit({
      instructions: instructions,
      depositor: marginAccount.owner,
      source,
      destination: position.address,
      change,
    })
    await marginAccount.withUpdatePositionBalance({ instructions, position })
    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withDeposit({
    instructions,
    depositor,
    source,
    destination,
    change
  }: {
    instructions: TransactionInstruction[]
    depositor: Address
    source: Address
    destination: Address
    change: PoolTokenChange
  }): Promise<void> {
    const ix = await this.programs.marginPool.methods
      .deposit(change.toRpcArg())
      .accounts({
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        depositor,
        source,
        destination,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)
  }

  async marginBorrow({ marginAccount, pools, change }: { marginAccount: MarginAccount; pools: Pool[]; change: PoolTokenChange }) {
    await marginAccount.refresh()
    const depositPosition = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(depositPosition)

    const loanPosition = await marginAccount.getOrCreatePosition(this.addresses.loanNoteMint)
    assert(loanPosition)

    const instructions: TransactionInstruction[] = []
    await this.withMarginRefreshAllPositionPrices({ instructions, pools, marginAccount })
    await marginAccount.withUpdateAllPositionBalances({ instructions })
    await this.withMarginBorrow({
      instructions,
      marginAccount,
      depositPosition,
      loanPosition,
      change
    })
    try {
      return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
    } catch (err) {
      console.log(err)
      throw err
    }
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
    loanPosition,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: AccountPosition
    loanPosition: AccountPosition
    change: PoolTokenChange
  }): Promise<void> {
    assert(marginAccount)
    assert(depositPosition)
    assert(loanPosition)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginBorrow(change.toRpcArg())
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          depositNoteMint: this.addresses.depositNoteMint,
          loanAccount: loanPosition.address,
          depositAccount: depositPosition.address,
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
    change
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    change: PoolTokenChange
  }) {
    await marginAccount.refresh()
    const deposit_position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(deposit_position)

    const loan_position = await marginAccount.getOrCreatePosition(this.addresses.loanNoteMint)
    assert(loan_position)

    const instructions: TransactionInstruction[] = []
    await marginAccount.withUpdateAllPositionBalances({ instructions })
    await this.withMarginRefreshAllPositionPrices({ instructions, pools, marginAccount })
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.makeMarginRepayInstruction({
        marginAccount: marginAccount.address,
        deposit_account: deposit_position.address,
        loan_account: loan_position.address,
        change
      })
    })

    // Automatically close the position once the loan is repaid.
    if (change.value.eq(loan_position.balance)) {
      //TODO
      //await marginAccount.withUpdatePositionBalance(ix, loan_position.address)
      //await marginAccount.withClosePosition(ix, loan_position)
    }

    try {
      return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async makeMarginRepayInstruction({
    marginAccount,
    deposit_account,
    loan_account,
    change
  }: {
    marginAccount: Address
    deposit_account: Address
    loan_account: Address
    change: PoolTokenChange
  }): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .marginRepay(change.toRpcArg())
      .accounts({
        marginAccount: marginAccount,
        marginPool: this.address,
        loanNoteMint: this.addresses.loanNoteMint,
        depositNoteMint: this.addresses.depositNoteMint,
        loanAccount: loan_account,
        depositAccount: deposit_account,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  /// Instruction to withdraw tokens from the pool.
  ///
  /// # Params
  ///
  /// `margin_account` - The margin account with the deposit to be withdrawn
  /// `change` - The amount to withdraw.
  /// `destination` - (Optional) The token account to send the withdrawn deposit
  async marginWithdraw({
    marginAccount,
    pools,
    change,
    destination
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    change: PoolTokenChange
    destination?: Address
  }) {
    // FIXME: can be getPosition
    const { address: source } = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)

    const preInstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []
    const postInstructions: TransactionInstruction[] = []

    let marginWithdrawDestination =
      destination ??
      (await AssociatedToken.withCreateOrUnwrapIfNativeMint(
        preInstructions,
        postInstructions,
        marginAccount.provider,
        this.tokenMint
      ))

    await this.withMarginRefreshAllPositionPrices({ instructions, pools, marginAccount })
    await marginAccount.withUpdateAllPositionBalances({ instructions })
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.makeMarginWithdrawInstruction({
        marginAccount: marginAccount.address,
        source,
        destination: marginWithdrawDestination,
        change
      })
    })

    return await marginAccount.provider.sendAndConfirm(
      new Transaction().add(...[...preInstructions, ...instructions, ...postInstructions])
    )
  }

  async makeMarginWithdrawInstruction({
    marginAccount,
    source,
    destination,
    change
  }: {
    marginAccount: Address
    source: Address
    destination: Address
    change: PoolTokenChange
  }): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .withdraw(change.toRpcArg())
      .accounts({
        depositor: marginAccount,
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        source,
        destination,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async closePosition({ marginAccount, destination }: { marginAccount: MarginAccount; destination: Address }) {
    await marginAccount.refresh()

    const position = marginAccount.getPosition(this.addresses.depositNoteMint)

    if (position) {
      if (position.balance.gt(Number192.ZERO)) {
        const destinationAddress = translateAddress(destination)

        const isDestinationNative = AssociatedToken.isNative(marginAccount.owner, this.tokenMint, destinationAddress)

        let marginWithdrawDestination: PublicKey
        if (!isDestinationNative) {
          marginWithdrawDestination = destinationAddress
        } else {
          marginWithdrawDestination = AssociatedToken.derive(this.tokenMint, marginAccount.owner)
        }

        const instructions: TransactionInstruction[] = []

        await marginAccount.withUpdatePositionBalance({ instructions, position })

        await AssociatedToken.withCreate(instructions, marginAccount.provider, marginAccount.owner, this.tokenMint)
        await marginAccount.withAdapterInvoke({
          instructions: instructions,
          adapterProgram: this.programs.config.marginPoolProgramId,
          adapterMetadata: this.addresses.marginPoolAdapterMetadata,
          adapterInstruction: await this.makeMarginWithdrawInstruction({
            marginAccount: marginAccount.address,
            source: position.address,
            destination: marginWithdrawDestination,
            change: PoolTokenChange.setTo(0),
          })
        })

        if (isDestinationNative) {
          AssociatedToken.withClose(instructions, marginAccount.owner, this.tokenMint, destinationAddress)
        }

        try {
          await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
        } catch (err) {
          console.log(err)
          throw err
        }

        await marginAccount.refresh()
      }

      await marginAccount.closePosition(position)
    }
  }
}
