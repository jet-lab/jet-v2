import { Address, BN, translateAddress } from "@project-serum/anchor"
import { parsePriceData, PriceData, PriceStatus } from "@pythnetwork/client"
import { Mint, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, TransactionInstruction, SYSVAR_RENT_PUBKEY, AccountMeta } from "@solana/web3.js"
import assert from "assert"
import { AssociatedToken, bigIntToBn, numberToBigInt, TokenAddress, TokenFormat } from "../../token"
import { TokenAmount } from "../../token/tokenAmount"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { MarginPoolConfigData, MarginPoolData } from "./state"
import { MarginTokenConfig } from "../config"
import { PoolTokenChange } from "./poolTokenChange"
import { findDerivedAccount } from "../../utils/pda"
import { PriceInfo } from "../accountPosition"
import { chunks, Number192 } from "../../utils"
import { FixedTermMarket } from "fixed-term"
import { base64 } from "@project-serum/anchor/dist/cjs/utils/bytes"
import { TokenConfig, TokenConfigInfo } from "../tokenConfig"
import { Airspace } from "../airspace"
import axios from "axios"
import { MarginPosition } from "../../../dist/wasm"
import { PositionKind } from "../../../dist"

/** A set of possible actions to perform on a margin pool. */
export type PoolAction = "deposit" | "withdraw" | "borrow" | "repay" | "repayFromDeposit" | "swap" | "transfer"

/** The PDA addresses associated with a [[Pool]] */
export interface PoolAddresses {
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

/**
 * The swap path for routing swaps
 */
export interface SwapPath {
  from_token: string
  to_token: string
  program: string
  swap_pool: string
}

/**
 * A projection or estimation of the pool after an action is taken.
 *
 * @export
 * @interface PoolProjection
 */
export interface PoolProjection {
  riskIndicator: number
  depositRate: number
  borrowRate: number
}

/**
 * An SPL swap pool
 *
 * @export
 * @interface SPLSwapPool
 */
export interface SPLSwapPool {
  swapPool: string
  authority: string
  poolMint: string
  tokenMintA: string
  tokenMintB: string
  tokenA: string
  tokenB: string
  feeAccount: string
  swapProgram: string
  swapFees: number
  swapType: "constantProduct" | "stable"
  amp?: number
}

export const feesBuffer: BN = new BN("75000000")

/**
 * A pool in which a [[MarginAccount]] can register a deposit and/or a borrow position.
 *
 * @export
 * @class Pool
 */
export class Pool {
  /**
   * The metadata of the [[Pool]] deposit note mint
   *
   * @type {TokenConfig}
   * @memberof Pool
   */
  depositNoteMetadata: TokenConfig
  /**
   * The metadata of the [[Pool]] loan note mint
   *
   * @type {TokenConfig}
   * @memberof Pool
   */
  loanNoteMetadata: TokenConfig

  /**
   * The address of the [[Pool]]
   *
   * @readonly
   * @type {PublicKey}
   * @memberof Pool
   */
  get address(): PublicKey {
    return this.addresses.marginPool
  }
  /**
   * The token mint of the [[Pool]]. It is incorrect to register a [[MarginAccount]] position using the token mint.
   * Rather `depositNoteMint` and `loanNoteMint` positions should be registered
   *
   * @readonly
   * @type {PublicKey}
   * @memberof Pool
   */
  get tokenMint(): PublicKey {
    return this.addresses.tokenMint
  }
  /**
   * The long-form token name
   *
   * @readonly
   * @type {(string)}
   * @memberof Pool
   */
  get name(): string {
    return this.tokenConfig.name
  }
  /**
   * The token symbol, such as "BTC" or "SOL"
   *
   * @readonly
   * @type {string}
   * @memberof Pool
   */
  get symbol(): string {
    return this.tokenConfig.symbol
  }
  /**
   * The raw vault balance
   *
   * @readonly
   * @type {Number192}
   * @memberof Pool
   */
  private get vaultRaw(): Number192 {
    return Number192.fromDecimal(this.info?.vault.amount.lamports ?? new BN(0), 0)
  }
  /**
   * The vault token balance
   *
   * @readonly
   * @type {TokenAmount}
   * @memberof Pool
   */
  get vault(): TokenAmount {
    return this.vaultRaw.toTokenAmount(this.decimals)
  }
  /**
   * The raw borrowed token balance
   *
   * @readonly
   * @private
   * @memberof Pool
   */
  private get borrowedTokensRaw() {
    if (!this.info) {
      return Number192.ZERO
    }
    return Number192.fromBits(this.info.marginPool.borrowedTokens)
  }
  /**
   * The borrowed tokens of the vault
   *
   * @readonly
   * @type {TokenAmount}
   * @memberof Pool
   */
  get borrowedTokens(): TokenAmount {
    return this.borrowedTokensRaw.toTokenAmount(this.decimals)
  }
  private get totalValueRaw(): Number192 {
    return this.borrowedTokensRaw.add(this.vaultRaw)
  }
  /**
   * The total tokens currently borrowed + available to borrow
   *
   * @readonly
   * @type {TokenAmount}
   * @memberof Pool
   */
  get totalValue(): TokenAmount {
    return this.totalValueRaw.toTokenAmount(this.decimals)
  }
  /**
   * The raw uncollected fees
   *
   * @readonly
   * @type {Number192}
   * @memberof Pool
   */
  private get uncollectedFeesRaw(): Number192 {
    if (!this.info) {
      return Number192.ZERO
    }
    return Number192.fromBits(this.info.marginPool.uncollectedFees)
  }
  /**
   * The uncollected fees of the pool.
   *
   * @readonly
   * @type {TokenAmount}
   * @memberof Pool
   */
  get uncollectedFees(): TokenAmount {
    return this.uncollectedFeesRaw.toTokenAmount(this.decimals)
  }
  /**
   * The borrow utilization rate, where 0 is no borrows and 1 is all tokens borrowed.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get utilizationRate(): number {
    return this.totalValue.tokens === 0 ? 0 : this.borrowedTokens.tokens / this.totalValue.tokens
  }
  /**
   * The continuous compounding deposit rate.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get depositCcRate(): number {
    return this.info ? Pool.getCcRate(this.info.marginPool.config, this.utilizationRate) : 0
  }
  /**
   * The APY depositors receive, determined by the utilization curve.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get depositApy(): number {
    const fee = this.managementFeeRate

    return Pool.getDepositRate(this.depositCcRate, this.utilizationRate, fee)
  }
  /**
   * The APR borrowers pay, determined by the utilization curve.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get borrowApr(): number {
    return Pool.getBorrowRate(this.depositCcRate)
  }
  /**
   * The management fee, a fraction of interest paid.
   */
  get managementFeeRate(): number {
    return (this.info?.marginPool.config.managementFeeRate ?? 0) * 1e-4 // bps
  }
  /**
   * The token price in USD provided by Pyth.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get tokenPrice(): number {
    return this.info?.tokenPriceOracle.price ?? 0
  }

  get tokenEmaPrice(): number {
    return this.info?.tokenPriceOracle.emaPrice.value ?? 0
  }

  private prices: PriceResult
  get depositNotePrice(): PriceInfo {
    return {
      value: this.prices.depositNotePrice,
      exponent: this.info?.tokenPriceOracle.exponent ?? 0,
      timestamp: bigIntToBn(this.info?.tokenPriceOracle.timestamp),
      isValid: Number(this.info ? this.info.tokenPriceOracle.status === PriceStatus.Trading : false)
    }
  }

  get loanNotePrice(): PriceInfo {
    return {
      value: this.prices.loanNotePrice,
      exponent: this.info?.tokenPriceOracle.exponent ?? 0,
      timestamp: bigIntToBn(this.info?.tokenPriceOracle.timestamp),
      isValid: Number(this.info ? this.info.tokenPriceOracle.status === PriceStatus.Trading : false)
    }
  }

  /**
   * The token mint decimals
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get decimals(): number {
    return this.tokenConfig?.decimals ?? this.info?.tokenMint.decimals ?? 0
  }
  /**
   * The visual token precision for UI strings.
   *
   * @readonly
   * @type {number}
   * @memberof Pool
   */
  get precision(): number {
    return this.tokenConfig?.precision ?? 0
  }

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
  public info?: {
    marginPool: MarginPoolData
    tokenMint: Mint
    vault: AssociatedToken
    depositNoteMint: Mint
    loanNoteMint: Mint
    tokenPriceOracle: PriceData
    tokenMetadata: TokenConfigInfo
  }
  /**
   * Creates a Pool
   *
   * @param programs
   * @param addresses
   * @param tokenConfig
   */
  constructor(public programs: MarginPrograms, public addresses: PoolAddresses, public tokenConfig: MarginTokenConfig) {
    this.depositNoteMetadata = new TokenConfig({ programs, airspace: undefined, tokenMint: addresses.depositNoteMint })
    this.loanNoteMetadata = new TokenConfig({ programs, airspace: undefined, tokenMint: addresses.loanNoteMint })

    const zero = new BN(0)
    this.prices = {
      priceValue: Number192.ZERO,
      depositNotePrice: zero,
      depositNoteConf: zero,
      depositNoteTwap: zero,
      loanNotePrice: zero,
      loanNoteConf: zero,
      loanNoteTwap: zero
    }
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
        throw Error("Pyth oracle does not exist but a margin pool does. The margin pool is incorrectly configured.")
      }
      this.info = {
        marginPool,
        tokenMint,
        vault: AssociatedToken.decodeAccount(vaultMintInfo, this.addresses.vault, tokenMint.decimals),
        depositNoteMint: AssociatedToken.decodeMint(depositNoteMintInfo, this.addresses.depositNoteMint),
        loanNoteMint: AssociatedToken.decodeMint(loanNoteMintInfo, this.addresses.loanNoteMint),
        tokenPriceOracle: parsePriceData(oracleInfo.data),
        tokenMetadata: (await TokenConfig.load(
          this.programs,
          Airspace.deriveAddress(this.programs.airspace.programId, this.programs.config.airspaces[0].name),
          this.addresses.tokenMint
        )).info as TokenConfigInfo
      }
    }

    this.depositNoteMetadata.decode(depositNoteMetadataInfo)
    this.loanNoteMetadata.decode(loanNoteMetadataInfo)
    this.prices = this.calculatePrices(this.info?.tokenPriceOracle)
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

    const depositNotePrice = priceValue.mul(depositNoteExchangeRate).toU64Rounded(pythPrice.exponent)
    const depositNoteConf = confValue.mul(depositNoteExchangeRate).toU64Rounded(pythPrice.exponent)
    const depositNoteTwap = twapValue.mul(depositNoteExchangeRate).toU64Rounded(pythPrice.exponent)
    const loanNotePrice = priceValue.mul(loanNoteExchangeRate).toU64Rounded(pythPrice.exponent)
    const loanNoteConf = confValue.mul(loanNoteExchangeRate).toU64Rounded(pythPrice.exponent)
    const loanNoteTwap = twapValue.mul(loanNoteExchangeRate).toU64Rounded(pythPrice.exponent)
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

  /**
   * Get the USD value of the smallest unit of deposit notes
   *
   * @return {Number192}
   * @memberof Pool
   */
  depositNoteExchangeRate(): Number192 {
    if (!this.info) {
      return Number192.ZERO
    }

    const depositNotes = BN.max(new BN(1), this.info.marginPool.depositNotes)
    const totalValue = Number192.max(Number192.ONE, this.totalValueRaw)
    return totalValue.sub(this.uncollectedFeesRaw).div(Number192.from(depositNotes))
  }

  /**
   * Get the USD value of the smallest unit of loan notes
   *
   * @return {Number192}
   * @memberof Pool
   */
  loanNoteExchangeRate(): Number192 {
    if (!this.info) {
      return Number192.ZERO
    }

    const loanNotes = BN.max(new BN(1), this.info.marginPool.loanNotes)
    const totalBorrowed = Number192.max(Number192.ONE, this.borrowedTokensRaw)
    return totalBorrowed.div(Number192.from(loanNotes))
  }

  /**
   * Linear interpolation between (x0, y0) and (x1, y1)
   * @param {number} x
   * @param {number} x0
   * @param {number} x1
   * @param {number} y0
   * @param {number} y1
   * @returns {number}
   */
  static interpolate = (x: number, x0: number, x1: number, y0: number, y1: number): number => {
    return y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
  }
  /**
   * Continous Compounding Rate
   * @param {number} reserveConfig
   * @param {number} utilRate
   * @returns {number}
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

  /**
   * Get continuous compounding borrow rate.
   *
   * @static
   * @param {number} ccRate
   * @return {number}
   * @memberof Pool
   */
  static getBorrowRate(ccRate: number): number {
    return ccRate
  }

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
  static getDepositRate(ccRate: number, utilRatio: number, feeFraction: number): number {
    return (1 - feeFraction) * ccRate * utilRatio
  }

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
  async marginRefreshAllPositionPrices({
    pools,
    marginAccount,
    markets
  }: {
    pools: Record<any, Pool> | Pool[]
    marginAccount: MarginAccount
    markets: FixedTermMarket[]
  }): Promise<string> {
    const instructions: TransactionInstruction[] = []
    await marginAccount.withPrioritisedPositionRefresh({ instructions, pools, markets })
    return await marginAccount.sendAndConfirm(instructions)
  }

  /**
   * Send a transaction to refresh all [[MarginAccount]] deposit or borrow positions associated with this [[Pool]] so that additional
   * borrows or withdraws can occur.
   *
   * @param {MarginAccount} marginAccount
   * @return {Promise<string>}
   * @memberof Pool
   */
  async marginRefreshPositionPrice(marginAccount: MarginAccount): Promise<string> {
    const instructions: TransactionInstruction[] = []
    await this.withMarginRefreshPositionPrice({ instructions, marginAccount })
    return await marginAccount.sendAndConfirm(instructions)
  }

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
  async withMarginRefreshAllPositionPrices({
    instructions,
    pools,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    pools: Record<any, Pool> | Pool[]
    marginAccount: MarginAccount
  }): Promise<void> {
    for (const pool of Object.values(pools)) {
      const positionRegistered =
        !!marginAccount.getPositionNullable(pool.addresses.depositNoteMint) ||
        !!marginAccount.getPositionNullable(pool.addresses.loanNoteMint)
      if (positionRegistered) {
        await pool.withMarginRefreshPositionPrice({ instructions, marginAccount })
      }
    }
  }

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
  async withMarginRefreshPositionPrice({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }): Promise<void> {
    if (!marginAccount || !this.info) throw new Error("Margin or pool not fully setup")
    await marginAccount.withAccountingInvoke({
      instructions: instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginRefreshPosition()
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          tokenPriceOracle: this.info?.marginPool.tokenPriceOracle
        })
        .instruction()
    })
  }

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
  async deposit({
    marginAccount,
    change,
    source = TokenFormat.unwrappedSol
  }: {
    marginAccount: MarginAccount
    change: PoolTokenChange
    source?: TokenAddress
  }): Promise<string> {
    assert(marginAccount)
    assert(change)

    const instructions: TransactionInstruction[] = []
    await marginAccount.withCreateAccount(instructions)
    const position = await this.withGetOrRegisterDepositPosition({ instructions, marginAccount })

    await this.withDeposit({
      instructions: instructions,
      marginAccount,
      source,
      change
    })
    await marginAccount.withUpdatePositionBalance({ instructions, position })
    return await marginAccount.sendAndConfirm(instructions)
  }

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
  async withDeposit({
    instructions,
    marginAccount,
    source = TokenFormat.unwrappedSol,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    source?: TokenAddress
    change: PoolTokenChange
  }): Promise<void> {
    const provider = marginAccount.provider
    const mint = this.tokenMint
    const position = this.findDepositPositionAddress(marginAccount)
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
        destination: position,
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
    destination,
    markets
  }: {
    marginAccount: MarginAccount
    pools: Record<string, Pool> | Pool[]
    change: PoolTokenChange
    destination?: TokenAddress
    markets: FixedTermMarket[]
  }): Promise<string> {
    if (!change.changeKind.isShiftBy()) {
      throw new Error("Use ShiftBy for all borrow instructions")
    }

    // Unnecessary to supply accounts, refresh is only to get latest positions.
    // We should remove this when we are confident with websockets as this is wasteful.
    await marginAccount.refresh({
      openPositions: [],
      openOrders: []
    })
    const refreshInstructions: TransactionInstruction[] = []
    const registerinstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []

    await marginAccount.withPrioritisedPositionRefresh({
      instructions: refreshInstructions,
      pools: Object.values(pools),
      markets
    })

    await this.withGetOrRegisterDepositPosition({
      instructions: registerinstructions,
      marginAccount
    })

    await this.withGetOrRegisterLoanPosition({
      instructions: registerinstructions,
      marginAccount
    })

    await this.withMarginBorrow({
      instructions: instructions,
      marginAccount,
      change
    })

    if (destination !== undefined) {
      // The borrow will increase the deposits by some unknown amount.
      // To withdraw the full borrow, set the deposit amount to the previous known amount before borrowing
      const poolPosition = Object.values(marginAccount.poolPositions).find(
        position => position.pool && position.pool.address.equals(this.address)
      )

      if (!poolPosition)
        throw new Error(
          "Attempting to withdraw after borrowing, but can not find the pool position in the margin account to calculate the withdraw amount."
        )
      const previousDepositAmount = poolPosition.depositBalance

      const withdrawChange =
        previousDepositAmount.tokens > 0 ? PoolTokenChange.shiftBy(change.value) : PoolTokenChange.setTo(0)

      await this.withWithdraw({
        instructions: instructions,
        marginAccount,
        destination,
        change: withdrawChange
      })
    }

    return await marginAccount.sendAll([chunks(11, refreshInstructions), registerinstructions, instructions])
  }

  /// Instruction to borrow tokens using a margin account
  ///
  /// # Params
  ///
  /// `instructions` - The array to append instuctions to.
  /// `marginAccount` - The account being borrowed against
  /// `loan_account` - The account to receive the notes representing the debt
  /// `change` - The amount of tokens to be borrowed as a `PoolTokenChange`
  async withMarginBorrow({
    instructions,
    marginAccount,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    change: PoolTokenChange
  }): Promise<void> {
    assert(instructions)
    assert(marginAccount)
    assert(change)

    const depositAccount = this.findDepositPositionAddress(marginAccount)
    const loanAccount = this.findLoanPositionAddress(marginAccount)

    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: await this.programs.marginPool.methods
        .marginBorrow(change.changeKind.asParam(), change.value)
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          depositNoteMint: this.addresses.depositNoteMint,
          loanAccount,
          depositAccount,
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
    markets,
    source,
    change,
    closeLoan,
    signer
  }: {
    marginAccount: MarginAccount
    pools: Record<string, Pool> | Pool[]
    markets: FixedTermMarket[]
    source?: TokenAddress
    change: PoolTokenChange
    closeLoan?: boolean
    signer?: Address
  }): Promise<string> {
    await marginAccount.refresh({
      openPositions: [],
      openOrders: []
    })
    const refreshInstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []
    const depositPosition = await this.withGetOrRegisterDepositPosition({
      instructions: refreshInstructions,
      marginAccount
    })
    assert(depositPosition)

    await marginAccount.withPrioritisedPositionRefresh({
      instructions: refreshInstructions,
      pools: Object.values(pools),
      markets
    })

    if (source === undefined) {
      await this.withMarginRepay({
        instructions,
        marginAccount: marginAccount,
        change
      })
    } else {
      await this.withRepay({
        instructions,
        marginAccount,
        depositPosition: depositPosition,
        source,
        change,
        feesBuffer,
        sourceAuthority: signer
      })
    }

    // Automatically close the position once the loan is repaid.
    if (closeLoan) {
      await this.withCloseLoanPosition({ instructions, marginAccount })
    }

    return await marginAccount.sendAll([chunks(11, refreshInstructions), instructions])
  }

  async withMarginRepay({
    instructions,
    marginAccount,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    change: PoolTokenChange
  }): Promise<void> {
    const depositAccount = this.findDepositPositionAddress(marginAccount)
    const loanAccount = this.findLoanPositionAddress(marginAccount)

    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: await this.programs.marginPool.methods
        .marginRepay(change.changeKind.asParam(), change.value)
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          depositNoteMint: this.addresses.depositNoteMint,
          loanAccount,
          depositAccount,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })
  }

  async withRepay({
    instructions,
    marginAccount,
    source,
    change,
    feesBuffer,
    sourceAuthority
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: Address
    source: TokenAddress
    change: PoolTokenChange
    feesBuffer: BN
    sourceAuthority?: Address
  }): Promise<void> {
    const wrappedSource = await AssociatedToken.withBeginTransferFromSource({
      instructions,
      provider: marginAccount.provider,
      mint: this.tokenMint,
      source,
      feesBuffer
    })

    const loanAccount = this.findLoanPositionAddress(marginAccount)

    const ix = await this.programs.marginPool.methods
      .repay(change.changeKind.asParam(), change.value)
      .accounts({
        marginPool: this.address,
        loanNoteMint: this.addresses.loanNoteMint,
        vault: this.addresses.vault,
        loanAccount,
        repaymentTokenAccount: wrappedSource,
        repaymentAccountAuthority: sourceAuthority ?? marginAccount.provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)

    await marginAccount.withUpdatePositionBalance({
      instructions,
      position: loanAccount
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
    markets,
    change,
    destination = TokenFormat.unwrappedSol
  }: {
    marginAccount: MarginAccount
    pools: Record<string, Pool> | Pool[]
    markets: FixedTermMarket[]
    change: PoolTokenChange
    destination?: TokenAddress
  }) {
    const refreshInstructions: TransactionInstruction[] = []
    const instructions: TransactionInstruction[] = []

    await marginAccount.withPrioritisedPositionRefresh({
      instructions: refreshInstructions,
      pools: Object.values(pools),
      markets
    })
    await marginAccount.withUpdateAllPositionBalances({ instructions: refreshInstructions })
    await this.withWithdraw({
      instructions,
      marginAccount: marginAccount,
      destination,
      change
    })
    return await marginAccount.sendAll([chunks(11, refreshInstructions), instructions])
  }

  async withWithdraw({
    instructions,
    marginAccount,
    destination = TokenFormat.unwrappedSol,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    destination?: TokenAddress
    change: PoolTokenChange
  }): Promise<void> {
    const provider = marginAccount.provider
    const mint = this.tokenMint
    const source = marginAccount.getPositionNullable(this.addresses.depositNoteMint)?.address
    assert(source, "No deposit position")

    const withdrawDestination = await AssociatedToken.withBeginTransferToDestination({
      instructions,
      provider,
      mint,
      destination
    })

    await marginAccount.withAdapterInvoke({
      instructions,
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

  async withWithdrawToMargin({
    instructions,
    marginAccount,
    change
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    change: PoolTokenChange
  }): Promise<void> {
    const source = marginAccount.getPositionNullable(this.addresses.depositNoteMint)?.address
    assert(source, "No deposit position")
    const destination = AssociatedToken.derive(this.addresses.tokenMint, marginAccount.address)

    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: await this.programs.marginPool.methods
        .withdraw(change.changeKind.asParam(), change.value)
        .accounts({
          depositor: marginAccount.address,
          marginPool: this.address,
          vault: this.addresses.vault,
          depositNoteMint: this.addresses.depositNoteMint,
          source,
          destination,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .instruction()
    })
  }

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
  async routeSwap({
    endpoint,
    marginAccount,
    pools,
    outputToken,
    swapAmount,
    minAmountOut,
    repayWithOutput,
    swapPaths,
    markets,
    lookupTables,
  }: {
    endpoint: string
    marginAccount: MarginAccount
    pools: Pool[]
    outputToken: Pool
    swapAmount: TokenAmount
    minAmountOut: TokenAmount
    repayWithOutput: boolean
    swapPaths: SwapPath[]
    markets: FixedTermMarket[]
    lookupTables: { address: string; data: Uint8Array }[]
  }) {
    assert(marginAccount)
    assert(swapAmount)

    interface RouteAccounts {
      accounts: {
        is_signer: boolean
        is_writable: boolean
        pubkey: string
      }[]
      data: string
    }

    const setupInstructions: TransactionInstruction[] = []
    const swapInstructions: TransactionInstruction[] = []

    // Setup check for swap
    const projectedRiskLevel = this.projectAfterMarginSwap(
      marginAccount,
      swapAmount.tokens,
      minAmountOut.tokens,
      outputToken,
      true
    ).riskIndicator
    if (projectedRiskLevel >= 1) {
      console.error(`Swap would exceed maximum risk (projected: ${projectedRiskLevel})`)
      return "Setup check failed"
    }

    // Refresh prices
    await marginAccount.withPrioritisedPositionRefresh({
      instructions: setupInstructions,
      pools,
      markets
    })

    // // Source deposit position fetch / creation
    // const sourceAccount = await this.withGetOrRegisterDepositPosition({
    //   instructions: registerInstructions,
    //   marginAccount
    // })

    // // Destination deposit position fetch / creation
    // const destinationAccount = await outputToken.withGetOrRegisterDepositPosition({
    //   instructions: registerInstructions,
    //   marginAccount
    // })

    // Transit source account fetch / creation
    await AssociatedToken.withCreate(
      setupInstructions,
      marginAccount.provider,
      marginAccount.address,
      this.addresses.tokenMint
    )

    // Transit destination account fetch / creation
    await AssociatedToken.withCreate(
      setupInstructions,
      marginAccount.provider,
      marginAccount.address,
      outputToken.tokenMint
    )

    // Destination pool account if it doesn't exist
    await marginAccount.withGetOrRegisterPosition({
      instructions: setupInstructions,
      positionTokenMint: outputToken.addresses.depositNoteMint
    });

    // Default change kind to a shiftBy
    const accountPoolPosition = marginAccount.poolPositions[this.symbol]
    let changeKind = PoolTokenChange.shiftBy(swapAmount)
    // If swapping total balance, use setTo(0)
    if (swapAmount.gte(accountPoolPosition.depositBalance)) {
      changeKind = PoolTokenChange.setTo(0)
    }

    // If swapping on margin
    if (swapAmount.gt(accountPoolPosition.depositBalance) && marginAccount.pools) {
      const difference = swapAmount.sub(accountPoolPosition.depositBalance)
      await this.withGetOrRegisterLoanPosition({
        instructions: setupInstructions,
        marginAccount
      })
      await this.withMarginBorrow({
        instructions: setupInstructions,
        marginAccount,
        change: PoolTokenChange.setTo(accountPoolPosition.loanBalance.add(difference))
      })
    }

    const swapInstruction = await axios.post<RouteAccounts>(`${endpoint}/swap/instruction`, {
      margin_account: marginAccount.address.toBase58(),
      from: this.tokenMint.toBase58(),
      to: outputToken.tokenMint.toBase58(),
      path: swapPaths,
      shift: changeKind.changeKind.isShiftBy(),
      input: changeKind.value.toNumber(),
      minimum_out: minAmountOut.lamports.toNumber()
    })

    const swapAccounts = swapInstruction.data.accounts.map(a => {
      return {
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.is_signer,
        isWritable: a.is_writable
      }
    })
    const swapData = base64.decode(swapInstruction.data.data)

    await this.withRouteSwap({
      instructions: swapInstructions,
      marginAccount,
      swapAccounts,
      swapData
    })

    // If they have a debt of the output token, automatically repay as much as possible
    const outputDebtPosition = marginAccount.poolPositions[outputToken.symbol].loanBalance
    if (!outputDebtPosition.isZero() && repayWithOutput) {
      const change = minAmountOut.gt(outputDebtPosition)
        ? PoolTokenChange.setTo(0)
        : PoolTokenChange.shiftBy(minAmountOut)
      await outputToken.withMarginRepay({
        instructions: swapInstructions,
        marginAccount,
        change
      })
    }

    return await marginAccount.sendAndConfirmV0(
      [setupInstructions, swapInstructions],
      lookupTables
    )
  }

  async withRouteSwap({
    instructions,
    marginAccount,
    // sourceAccount,
    // destinationAccount,
    swapAccounts,
    swapData
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    // sourceAccount: Address
    // destinationAccount: Address
    swapAccounts: AccountMeta[]
    swapData: Buffer
  }): Promise<void> {
    // Create instructions for the swap route based on chosen path

    // Swap
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: new TransactionInstruction({
        programId: this.programs.marginSwap.programId,
        data: swapData,
        keys: swapAccounts
      })
    })

    // Update account positions
    // await marginAccount.withUpdatePositionBalance({ instructions, position: sourceAccount })
    // await marginAccount.withUpdatePositionBalance({ instructions, position: destinationAccount })

    // TODO: there might be dust in intermediate accounts, refresh those too
  }

  /**
   * Derive the address of a deposit position token account associated with a [[MarginAccount]].
   *
   * @param {MarginAccount} marginAccount
   * @return {PublicKey}
   * @memberof MarginAccount
   */
  findDepositPositionAddress(marginAccount: MarginAccount): PublicKey {
    return marginAccount.findPositionTokenAddress(this.addresses.depositNoteMint)
  }

  async withGetOrRegisterDepositPosition({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }): Promise<Address> {
    const positionTokenMint = this.addresses.depositNoteMint
    const position = marginAccount.getPositionNullable(positionTokenMint)
    if (position) {
      return position.address
    }
    return await this.withRegisterDepositPosition({
      instructions,
      marginAccount
    })
  }

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
  async withRegisterDepositPosition({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }): Promise<Address> {
    const positionTokenMint = this.addresses.depositNoteMint
    const account = marginAccount.getPositionNullable(positionTokenMint)
    if (account) {
      return account.address
    }
    return await marginAccount.withRegisterPosition({ instructions, positionTokenMint })
  }

  async withCloseDepositPosition({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }) {
    const position = marginAccount.getPosition(this.addresses.depositNoteMint)
    await marginAccount.withClosePosition(instructions, position)
  }

  async withdrawAndCloseDepositPosition({
    marginAccount,
    destination
  }: {
    marginAccount: MarginAccount
    destination: Address
  }) {
    await marginAccount.refresh({
      openPositions: [],
      openOrders: []
    })

    const position = marginAccount.getPositionNullable(this.addresses.depositNoteMint)

    if (position) {
      const instructions: TransactionInstruction[] = []

      if (position.balance.gt(new BN(0))) {
        const destinationAddress = translateAddress(destination)

        const isDestinationNative = AssociatedToken.isNative(marginAccount.owner, this.tokenMint, destinationAddress)

        let marginWithdrawDestination: PublicKey
        if (!isDestinationNative) {
          marginWithdrawDestination = destinationAddress
        } else {
          marginWithdrawDestination = AssociatedToken.derive(this.tokenMint, marginAccount.owner)
        }

        await marginAccount.withUpdatePositionBalance({ instructions, position: position.address })

        await AssociatedToken.withCreate(instructions, marginAccount.provider, marginAccount.owner, this.tokenMint)
        await this.withWithdraw({
          instructions,
          marginAccount: marginAccount,
          destination: marginWithdrawDestination,
          change: PoolTokenChange.setTo(0)
        })

        if (isDestinationNative) {
          AssociatedToken.withClose(instructions, marginAccount.owner, this.tokenMint, destinationAddress)
        }
      }

      await this.withCloseDepositPosition({ instructions, marginAccount })

      await marginAccount.sendAll([instructions])
      await marginAccount.refresh({
        openPositions: [],
        openOrders: []
      })
    }
  }

  findLoanPositionAddress(marginAccount: MarginAccount) {
    return findDerivedAccount(
      this.programs.config.marginPoolProgramId,
      marginAccount.address,
      this.addresses.loanNoteMint
    )
  }

  async withGetOrRegisterLoanPosition({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }): Promise<Address> {
    const positionTokenMint = this.addresses.loanNoteMint
    const account = marginAccount.getPositionNullable(positionTokenMint)
    if (account) {
      return account.address
    }
    return await this.withRegisterLoanPosition(instructions, marginAccount)
  }

  async withRegisterLoanPosition(
    instructions: TransactionInstruction[],
    marginAccount: MarginAccount
  ): Promise<Address> {
    const loanNoteAccount = this.findLoanPositionAddress(marginAccount)
    await marginAccount.withAdapterInvoke({
      instructions: instructions,
      adapterInstruction: await this.programs.marginPool.methods
        .registerLoan()
        .accounts({
          marginAccount: marginAccount.address,
          positionTokenMetadata: this.addresses.loanNoteMetadata,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          loanNoteAccount,
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

  async withCloseLoanPosition({
    instructions,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
  }) {
    const loanNoteAccount = findDerivedAccount(
      this.programs.config.marginPoolProgramId,
      marginAccount.address,
      this.addresses.loanNoteMint
    )
    await marginAccount.withAdapterInvoke({
      instructions,
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

  projectAfterAction(
    marginAccount: MarginAccount,
    amount: number,
    action: PoolAction,
    // For swap projections
    minAmountOut?: number,
    outputToken?: Pool,
    repayWithProceeds?: boolean
  ): PoolProjection {
    switch (action) {
      case "deposit":
        return this.projectAfterDeposit(marginAccount, amount)
      case "withdraw":
        return this.projectAfterWithdraw(marginAccount, amount)
      case "borrow":
        return this.projectAfterBorrowAndNotWithdraw(marginAccount, amount)
      case "repay":
        return this.projectAfterRepay(marginAccount, amount)
      case "repayFromDeposit":
        return this.projectAfterRepayFromDeposit(marginAccount, amount)
      case "swap":
        return this.projectAfterMarginSwap(marginAccount, amount, minAmountOut, outputToken, repayWithProceeds === true)
      default:
        throw new Error("Unknown pool action")
    }
  }

  /// Projects the deposit / borrow rates and user's risk level after a deposit into the pool.
  projectAfterDeposit(marginAccount: MarginAccount, amount: number): PoolProjection {
    if ((this.info === undefined) || (marginAccount.prices == undefined)) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens
    const totalTokens = this.totalValue.tokens + amount

    const utilRatio = borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Deposit increases
      this.depositPositionChange(marginAccount, amount),
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(
      marginValuation.requiredCollateral,
      marginValuation.weightedCollateral,
      marginValuation.liabilities
    )

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit / borrow rates and user's risk level after a withdrawal from the pool.
  projectAfterWithdraw(marginAccount: MarginAccount, amount: number): PoolProjection {
    const position = marginAccount.poolPositions[this.symbol]
    if (this.info === undefined || amount > position.depositBalance.tokens) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens
    const totalTokens = this.totalValue.tokens - amount

    const utilRatio = totalTokens === 0 ? 0 : borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Deposit decreases
      this.depositPositionChange(marginAccount, -amount),
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(
      marginValuation.requiredCollateral,
      marginValuation.weightedCollateral > 0 ? marginValuation.weightedCollateral : 0,
      marginValuation.liabilities
    )

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit / borrow rates and user's risk level after a borrow from the pool.
  projectAfterBorrow(marginAccount: MarginAccount, amount: number): PoolProjection {
    if (this.info === undefined) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens + amount
    const totalTokens = this.totalValue.tokens + amount

    const utilRatio = borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Loan increases, the amount is withdrawn to the wallet
      // TODO: Is this correct even for fixed term?
      this.loanPositionChange(marginAccount, amount)
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(marginValuation.requiredCollateral, marginValuation.weightedCollateral, marginValuation.liabilities)

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit / borrow rates and user's risk level after repaying a loan from the pool.
  projectAfterRepay(marginAccount: MarginAccount, amount: number): PoolProjection {
    const position = marginAccount.poolPositions[this.symbol]
    if (position === undefined || this.info === undefined || amount > position.loanBalance.tokens) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens - amount
    const totalTokens = this.totalValue.tokens - amount

    const utilRatio = totalTokens === 0 ? 0 : borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Loan decreases, the repayment is from the wallet
      this.loanPositionChange(marginAccount, -amount)
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(
      marginValuation.requiredCollateral >= 0 ? marginValuation.requiredCollateral : 0,
      marginValuation.weightedCollateral,
      marginValuation.liabilities >= 0 ? marginValuation.liabilities : 0
    )

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit / borrow rates and user's risk level after repaying a loan from the pool.
  projectAfterRepayFromDeposit(marginAccount: MarginAccount, amount: number): PoolProjection {
    const position = marginAccount.poolPositions[this.symbol]
    if (
      position === undefined ||
      this.info === undefined ||
      amount > position.loanBalance.tokens ||
      amount > position.depositBalance.tokens
    ) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens - amount
    const totalTokens = this.totalValue.tokens - 2 * amount

    const utilRatio = totalTokens === 0 ? 0 : borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Deposit decreases
      this.depositPositionChange(marginAccount, -amount),
      // Loan decreases
      this.loanPositionChange(marginAccount, -amount)
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(
      marginValuation.requiredCollateral > 0 ? marginValuation.requiredCollateral : 0,
      marginValuation.weightedCollateral > 0 ? marginValuation.weightedCollateral : 0,
      marginValuation.liabilities > 0 ? marginValuation.liabilities : 0
    )

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the deposit / borrow rates and user's risk level after a borrow from the pool.
  projectAfterBorrowAndNotWithdraw(marginAccount: MarginAccount, amount: number): PoolProjection {
    if (this.info === undefined) {
      return this.getDefaultPoolProjection(marginAccount)
    }

    const borrowedTokens = this.borrowedTokens.tokens + amount
    const totalTokens = this.totalValue.tokens + 2 * amount

    const utilRatio = borrowedTokens / totalTokens
    const depositCcRate = Pool.getCcRate(this.info.marginPool.config, utilRatio)
    const fee = this.managementFeeRate

    const depositRate = Pool.getDepositRate(depositCcRate, utilRatio, fee)
    const borrowRate = Pool.getBorrowRate(depositCcRate)

    const positionChanges: MarginPosition[] = [
      // Deposit increases
      this.depositPositionChange(marginAccount, amount),
      // Loan increases
      this.loanPositionChange(marginAccount, amount)
    ];

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    const riskIndicator = marginAccount.computeRiskIndicator(marginValuation.requiredCollateral, marginValuation.weightedCollateral, marginValuation.liabilities)

    return { riskIndicator, depositRate, borrowRate }
  }

  /// Projects the user's risk level after a swap.
  projectAfterMarginSwap(
    marginAccount: MarginAccount,
    amount: number,
    minAmountOut: number | undefined,
    outputToken: Pool | undefined,
    repayWithProceeds: boolean,
    _setupCheck?: boolean
  ): PoolProjection {
    const defaults = this.getDefaultPoolProjection(marginAccount)
    if (!minAmountOut || !outputToken) {
      return defaults
    }

    // TODO(Neville): This is legacy, for route swaps we'll consider whether 
    // the swap is from a margin pool or directly from a margin account.

    const positionChanges: MarginPosition[] = [
      // Deposit from this pool decreases
      this.depositPositionChange(marginAccount, -amount),
      // Deposit in the output pool increases by minAmountOut
      outputToken.depositPositionChange(marginAccount, minAmountOut)
    ];

    if (repayWithProceeds) {
      // Determine how much is repayable in tokens
      const repayableAmount = Math.min(
        marginAccount.poolPositions[outputToken.symbol].depositBalance.tokens / outputToken.depositNoteExchangeRate().toNumber() + minAmountOut,
        marginAccount.poolPositions[outputToken.symbol].loanBalance.tokens / outputToken.loanNoteExchangeRate().toNumber()
      );
      // Deposit in the output pool decreases by the repayable amount
      positionChanges.push(outputToken.depositPositionChange(marginAccount, -repayableAmount))
      // Loan in the output pool decreases by the repayable amount
      positionChanges.push(outputToken.loanPositionChange(marginAccount, -repayableAmount))
    }

    const marginValuation = marginAccount.projectValuation(positionChanges, marginAccount.prices);

    // TODO: this doesn't consider setup collateral requirements.
    // They are unnecessary as this is technically dead code, keeping it for reference
    // until we complete route swaps.
    const riskIndicator = marginAccount.computeRiskIndicator(
      marginValuation.requiredCollateral,
      marginValuation.weightedCollateral,
      marginValuation.liabilities
    )

    // TODO: add pool projections for rates
    return {
      riskIndicator,
      depositRate: defaults.depositRate,
      borrowRate: defaults.borrowRate
    }
  }

  // TODO: this should return values that indicate the return value hasn't changed
  private getDefaultPoolProjection(marginAccount: MarginAccount) {
    return {
      riskIndicator: marginAccount.riskIndicator,
      depositRate: this.depositApy,
      borrowRate: this.borrowApr
    }
  }

  private depositPositionChange(marginAccount: MarginAccount, amount: number): MarginPosition {
    const position = marginAccount.poolPositions[this.symbol]
    return new MarginPosition(
      marginAccount.findPositionTokenAddress(this.addresses.depositNoteMint).toBase58(),
      this.addresses.depositNoteMint.toString(),
      numberToBigInt(amount * Math.pow(10, this.decimals) / this.depositNoteExchangeRate().toNumber()),
      -this.decimals,
      position.depositPosition?.kind ?? this.depositNoteMetadata.tokenKind,
      // Use an existing position's modifier or the default
      position.depositPosition?.valueModifier ?? this.depositNoteMetadata.valueModifier
    )
  }

  private loanPositionChange(marginAccount: MarginAccount, amount: number): MarginPosition {
    const position = marginAccount.poolPositions[this.symbol]
    return new MarginPosition(
      this.findLoanPositionAddress(marginAccount).toBase58(),
      this.addresses.loanNoteMint.toString(),
      numberToBigInt(amount * Math.pow(10, this.decimals) / this.loanNoteExchangeRate().toNumber()),
      -this.decimals,
      PositionKind.Claim,
      // Use an existing position's modifier or the default
      position.loanPosition?.valueModifier ?? this.loanNoteMetadata.valueModifier
    )
  }
}
