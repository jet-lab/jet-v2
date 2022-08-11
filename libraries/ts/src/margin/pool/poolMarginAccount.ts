import { translateAddress } from "@project-serum/anchor"
import { NATIVE_MINT } from "@solana/spl-token"
import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import BN from "bn.js"
import assert from "assert"
import { Number192, Number128 } from "src/utils"
import { bigIntToBn, numberToBn, TokenAmount } from "../../token"
import { AccountPosition } from "../accountPosition"
import { MarginTokenConfig } from "../config"
import { IAdapter } from "../IAdapterClient"
import { MarginAccount } from "../marginAccount"
import { FEES_BUFFER, Pool, PoolAction } from "./pool"

export interface PoolPosition {
  symbol: string
  tokenConfig: MarginTokenConfig
  pool?: Pool
  depositPosition: AccountPosition | undefined
  depositBalance: TokenAmount
  depositValue: number
  loanPosition: AccountPosition | undefined
  loanBalance: TokenAmount
  loanValue: number
  maxTradeAmounts: Record<PoolAction, TokenAmount>
  liquidationEndingCollateral: TokenAmount
  buyingPower: TokenAmount
}

export class PoolMarginAccount implements IAdapter {
  public adapterProgramId: PublicKey
  private poolPositions: Record<string, PoolPosition>

  constructor(public account: MarginAccount, public pools: Record<string, Pool>) {
    this.adapterProgramId = translateAddress(account.programs.config.marginPoolProgramId)
    this.poolPositions = this.calculatePoolPositions()
  }

  /**
   * Returns the pool position for a symbol.
   * A position is returned regardless of a deposit or borrow existing.
   * Throws an error if the pool does not exist.
   * If an error is undesirable, call `getNullable` instead */
  get(symbol: string): PoolPosition {
    const position = this.getNullable(symbol)
    assert(position, "Pool symbol does not exist")
    return this.poolPositions[symbol]
  }

  /**
   * Returns the pool position for a symbol.
   * A position is returned regardless of a deposit or borrow existing.
   * Returns null if the pool does not exist. */
  getNullable(symbol: string): PoolPosition | null {
    return this.poolPositions[symbol] ?? null
  }

  all() {
    return Object.values(this.poolPositions)
  }

  getPrice(mint: PublicKey) {
    for (const pool of Object.values(this.pools)) {
      const price = pool.getPrice(mint)
      if (price) {
        return price
      }
    }
  }

  getPoolNullable(positionTokenMint: PublicKey) {
    return Object.values(this.pools).find(
      pool =>
        pool.addresses.depositNoteMint.equals(positionTokenMint) ||
        pool.addresses.loanNoteMint.equals(positionTokenMint)
    )
  }

  getPool(positionTokenMint: PublicKey) {
    const pool = this.getPoolNullable(positionTokenMint)
    assert(pool, "Pool not found")
    return pool
  }

  calculatePoolPositions(): Record<string, PoolPosition> {
    const positions: Record<string, PoolPosition> = {}
    const poolConfigs = Object.values(this.account.programs.config.tokens)

    for (let i = 0; i < poolConfigs.length; i++) {
      const poolConfig = poolConfigs[i]
      const tokenConfig = this.account.programs.config.tokens[poolConfig.symbol]
      const pool = this.pools?.[poolConfig.symbol]
      const valuation = this.account.valuation
      if (!pool?.info) {
        continue
      }

      // Deposits
      const depositNotePosition = this.account.getPosition(pool.addresses.depositNoteMint)
      const depositBalanceNotes = Number192.from(depositNotePosition?.balance ?? new BN(0))
      const depositBalance = depositBalanceNotes.mul(pool.depositNoteExchangeRate()).asTokenAmount(pool.decimals)
      const depositValue = depositNotePosition?.value ?? 0

      // Loans
      const loanNotePosition = this.account.getPosition(pool.addresses.loanNoteMint)
      const loanBalanceNotes = Number192.from(loanNotePosition?.balance ?? new BN(0))
      const loanBalance = loanBalanceNotes.mul(pool.loanNoteExchangeRate()).asTokenAmount(pool.decimals)
      const loanValue = loanNotePosition?.value ?? 0

      // Max trade amounts
      const maxTradeAmounts = this.calculateMaxTradeAmounts(pool, depositBalance, loanBalance)

      // Minimum amount to deposit for the pool to end a liquidation
      const collateralWeight = depositNotePosition?.valueModifier ?? pool.depositNoteMetadata.valueModifier
      const priceComponent = bigIntToBn(pool.info.tokenPriceOracle.aggregate.priceComponent)
      const priceExponent = pool.info.tokenPriceOracle.exponent
      const tokenPrice = Number128.fromDecimal(priceComponent, priceExponent)
      const lamportPrice = tokenPrice.div(Number128.fromDecimal(new BN(1), pool.decimals))
      const warningRiskLevel = Number128.fromDecimal(new BN(MarginAccount.RISK_WARNING_LEVEL * 100000), -5)
      const liquidationEndingCollateral = (
        collateralWeight.isZero() || lamportPrice.isZero()
          ? Number128.ZERO
          : valuation.requiredCollateral
              .sub(valuation.effectiveCollateral.mul(warningRiskLevel))
              .div(collateralWeight.mul(warningRiskLevel))
              .div(lamportPrice)
      ).asTokenAmount(pool.decimals)

      // Buying power
      // FIXME
      const buyingPower = TokenAmount.zero(pool.decimals)

      positions[poolConfig.symbol] = {
        symbol: poolConfig.symbol,
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
      }
    }

    return positions
  }

  calculateMaxTradeAmounts(
    pool: Pool,
    depositBalance: TokenAmount,
    loanBalance: TokenAmount
  ): Record<PoolAction, TokenAmount> {
    const zero = TokenAmount.zero(pool.decimals)
    if (!pool.info) {
      return {
        deposit: zero,
        withdraw: zero,
        borrow: zero,
        repay: zero,
        swap: zero,
        transfer: zero
      }
    }

    // Wallet's balance for pool
    // If depsiting or repaying SOL, maximum input should consider fees
    let walletAmount = TokenAmount.zero(pool.decimals)
    if (pool.symbol && this.account.walletTokens) {
      walletAmount = this.account.walletTokens.map[pool.symbol].amount
    }
    if (pool.tokenMint.equals(NATIVE_MINT)) {
      walletAmount = TokenAmount.max(walletAmount.subb(numberToBn(FEES_BUFFER)), TokenAmount.zero(pool.decimals))
    }

    // Max deposit
    const deposit = walletAmount

    const priceExponent = pool.info.tokenPriceOracle.exponent
    const priceComponent = bigIntToBn(pool.info.tokenPriceOracle.aggregate.priceComponent)
    const tokenPrice = Number128.fromDecimal(priceComponent, priceExponent)
    const lamportPrice = tokenPrice.div(Number128.fromDecimal(new BN(1), pool.decimals))

    const depositNoteValueModifier =
      this.account.getPosition(pool.addresses.depositNoteMint)?.valueModifier ?? pool.depositNoteMetadata.valueModifier
    const loanNoteValueModifier =
      this.account.getPosition(pool.addresses.loanNoteMint)?.valueModifier ?? pool.loanNoteMetadata.valueModifier

    // Max withdraw
    let withdraw = this.account.valuation.availableSetupCollateral
      .div(depositNoteValueModifier)
      .div(lamportPrice)
      .asTokenAmount(pool.decimals)
    withdraw = TokenAmount.min(withdraw, depositBalance)
    withdraw = TokenAmount.min(withdraw, pool.vault)
    withdraw = TokenAmount.max(withdraw, zero)

    // Max borrow
    let borrow = this.account.valuation.availableSetupCollateral
      .div(Number128.ONE.add(Number128.ONE.div(MarginAccount.SETUP_LEVERAGE_FRACTION.mul(loanNoteValueModifier))))
      .div(lamportPrice)
      .asTokenAmount(pool.decimals)
    borrow = TokenAmount.min(borrow, pool.vault)
    borrow = TokenAmount.max(borrow, zero)

    // Max repay
    const repay = TokenAmount.min(loanBalance, walletAmount)

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

  async withRefreshPosition(instructions: TransactionInstruction[], positionTokenMint: PublicKey): Promise<void> {
    const pool = this.getPool(positionTokenMint)
    await pool.withMarginRefreshPositionPrice({ instructions, marginAccount: this.account })
  }
}
