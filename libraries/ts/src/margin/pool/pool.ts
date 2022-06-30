import { Address, BN, translateAddress } from "@project-serum/anchor"
import { parsePriceData, PriceData } from "@pythnetwork/client"
import { Mint, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, Transaction, TransactionInstruction, SYSVAR_RENT_PUBKEY } from "@solana/web3.js"
import { assert } from "chai"
import { AssociatedToken } from "../../token"
import { ONE_BN, TokenAmount, ZERO_BN } from "../../token/tokenAmount"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { MarginPoolConfigData, MarginPoolData } from "./state"
import { MarginPoolConfig, MarginPools, MarginTokenConfig } from "../config"
import { PoolAmount } from "./poolAmount"
import { AccountPosition } from "../state"
import { TokenMetadata } from "../metadata/state"
import { findDerivedAccount } from "../../utils/pda"

type TokenKindNonCollateral = { nonCollateral: Record<string, never> }
type TokenKindCollateral = { collateral: Record<string, never> }
type TokenKindClaim = { claim: Record<string, never> }

export type TokenKind = TokenKindNonCollateral | TokenKindCollateral | TokenKindClaim

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

export class Pool {
  public address: PublicKey

  get name(): string | undefined {
    return this.tokenConfig?.name
  }
  get symbol(): MarginPools | undefined {
    return this.poolConfig?.symbol
  }
  get depositedTokens(): TokenAmount {
    return this.info?.vault.amount ?? TokenAmount.zero(this.decimals)
  }
  get borrowedTokens(): TokenAmount {
    if (!this.info) {
      return TokenAmount.zero(this.decimals)
    }
    const lamports = new BN(this.info.marginPool.borrowedTokens, "le").div(ONE_BN)
    return TokenAmount.lamports(lamports, this.decimals)
  }
  get marketSize(): TokenAmount {
    return this.depositedTokens.add(this.borrowedTokens)
  }
  get uncollectedFees(): TokenAmount {
    if (!this.info) {
      return TokenAmount.zero(this.decimals)
    }
    const lamports = new BN(this.info.marginPool.uncollectedFees, "le").div(ONE_BN)
    return TokenAmount.lamports(lamports, this.decimals)
  }
  get utilizationRate(): number {
    return this.marketSize.tokens === 0 ? 0 : this.borrowedTokens.tokens / this.marketSize.tokens
  }
  get cRatio(): number {
    const utilizationRate = this.utilizationRate
    return utilizationRate === 0 ? Infinity : 1 / this.utilizationRate
  }
  get minCRatio(): number {
    return 1.0 // FIXME
  }
  get maxLeverage(): number {
    const minCRatio = this.minCRatio
    return minCRatio - 1 === 0 ? Infinity : 1 / this.minCRatio - 1
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
    this.address = this.addresses.marginPool
  }

  async refresh() {
    const [marginPoolInfo, poolTokenMintInfo, vaultMintInfo, depositNoteMintInfo, loanNoteMintInfo, tokenMetadataInfo] =
      await this.programs.marginPool.provider.connection.getMultipleAccountsInfo([
        this.addresses.marginPool,
        this.addresses.tokenMint,
        this.addresses.vault,
        this.addresses.depositNoteMint,
        this.addresses.loanNoteMint,
        this.addresses.tokenMetadata
      ])

    if (
      !marginPoolInfo ||
      !poolTokenMintInfo ||
      !vaultMintInfo ||
      !depositNoteMintInfo ||
      !loanNoteMintInfo ||
      !tokenMetadataInfo
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
    console.assert(x >= x0)
    console.assert(x <= x1)

    return y0 + ((x - x0) * (y1 - y0)) / (x1 - x0)
  }

  /**
   * Continous Compounding Rate
   * @param reserveConfig
   * @param utilRate
   * @returns
   */
  static getCcRate = (reserveConfig: MarginPoolConfigData, utilRate: number): number => {
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
  static getBorrowApr = (ccRate: number, fee: number): number => {
    const basisPointFactor = 10000
    fee = fee / basisPointFactor
    const secondsPerYear: number = 365 * 24 * 60 * 60
    const rt = ccRate / secondsPerYear

    return Math.log1p((1 + fee) * Math.expm1(rt)) * secondsPerYear
  }

  /** Deposit rate
   */
  static getDepositApy = (ccRate: number, utilRatio: number): number => {
    const secondsPerYear: number = 365 * 24 * 60 * 60
    const rt = ccRate / secondsPerYear

    return Math.log1p(Math.expm1(rt)) * secondsPerYear * utilRatio
  }

  /// Instruction to deposit tokens into the pool in exchange for deposit notes
  ///
  /// # Params
  ///
  /// `depositor` - The authority for the source tokens
  /// `source` - The token account that has the tokens to be deposited
  /// `destination` - The token account to send notes representing the deposit
  /// `amount` - The amount of tokens to be deposited
  async deposit({ marginAccount, source, amount }: { marginAccount: MarginAccount; source: Address; amount: number }) {
    await marginAccount.refresh()
    const position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(position)

    const instructions: TransactionInstruction[] = []

    await this.withDeposit({
      instructions,
      depositor: marginAccount.address,
      source,
      destination: position.address,
      amount: new BN(amount)
    })
    await marginAccount.withUpdatePositionBalance({ instructions, position })

    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withDeposit({
    instructions,
    depositor,
    source,
    destination,
    amount
  }: {
    instructions: TransactionInstruction[]
    depositor: Address
    source: Address
    destination: Address
    amount: BN
  }): Promise<void> {
    const ix = await this.programs.marginPool.methods
      .deposit(amount)
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

  // async refreshAllPoolPositions(
  //   connection: Connection,
  //   marginAccount: MarginAccount,
  // ) {
  //   // we need to get the positions
  //   //
  // }

  async refreshPosition(marginAccount: MarginAccount) {
    const instructions: TransactionInstruction[] = []
    await this.withMarginRefreshPosition({ instructions, marginAccount })
    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async withMarginRefreshPosition({
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

  async withMarginRefreshAllPositions({
    instructions,
    pools,
    marginAccount
  }: {
    instructions: TransactionInstruction[]
    pools: Pool[]
    marginAccount: MarginAccount
  }) {
    for (const pool of pools) {
      await pool.withMarginRefreshPosition({ instructions, marginAccount })
    }
  }

  async marginBorrow({ marginAccount, pools, amount }: { marginAccount: MarginAccount; pools: Pool[]; amount: BN }) {
    await marginAccount.refresh()
    const depositPosition = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(depositPosition)

    const loanPosition = await this.getOrCreateLoanPosition(marginAccount)
    assert(loanPosition)

    const instructions: TransactionInstruction[] = []
    await this.withMarginRefreshAllPositions({ instructions, pools, marginAccount })
    await marginAccount.withUpdateAllPositionBalances({ instructions })
    await this.withMarginBorrow({
      instructions,
      marginAccount,
      depositPosition,
      loanPosition,
      amount
    })
    try {
      return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async getOrCreateLoanPosition(marginAccount: MarginAccount) {
    let [loanNoteAccount, registerLoanIx] = await this.makeRegisterLoanInstruction(marginAccount)
    return await marginAccount.getOrCreateUnownedPosition(
      [registerLoanIx],
      this.addresses.loanNoteMint,
      loanNoteAccount
    )
  }

  /// Instruction to borrow tokens using a margin account
  ///
  /// # Params
  ///
  /// `instructions` - The array to append instuctions to.
  /// `marginAccount` - The account being borrowed against
  /// `deposit_account` - The account to receive the notes for the borrowed tokens
  /// `loan_account` - The account to receive the notes representing the debt
  /// `amount` - The amount of tokens to be borrowed
  async withMarginBorrow({
    instructions,
    marginAccount,
    depositPosition,
    loanPosition,
    amount
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    depositPosition: AccountPosition
    loanPosition: AccountPosition
    amount: BN
  }): Promise<void> {
    assert(marginAccount)
    assert(depositPosition)
    assert(loanPosition)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.programs.marginPool.methods
        .marginBorrow(amount)
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
  /// `amount` - The amount to be repaid
  async marginRepay({
    marginAccount,
    pools,
    amount
  }: {
    marginAccount: MarginAccount
    pools: Pool[]
    amount: PoolAmount
  }) {
    await marginAccount.refresh()
    const deposit_position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(deposit_position)

    const loan_position = await this.getOrCreateLoanPosition(marginAccount)
    assert(loan_position)

    const instructions: TransactionInstruction[] = []
    await marginAccount.withUpdateAllPositionBalances({ instructions })
    await this.withMarginRefreshAllPositions({ instructions, pools, marginAccount })
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.makeMarginRepayInstruction({
        marginAccount: marginAccount.address,
        deposit_account: deposit_position.address,
        loan_account: loan_position.address,
        amount
      })
    })

    // Automatically close the position once the loan is repaid.
    if (amount.value.eq(loan_position.balance)) {
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
    amount
  }: {
    marginAccount: Address
    deposit_account: Address
    loan_account: Address
    amount: PoolAmount
  }): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .marginRepay(amount.toRpcArg())
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

  /// Instruction to withdraw tokens from the pool in exchange for deposit notes
  /// (owned by a margin account)
  ///
  /// # Params
  ///
  /// `margin_account` - The margin account with the deposit to be withdrawn
  /// `source` - The token account that has the deposit notes to be exchanged
  /// `destination` - The token account to send the withdrawn deposit
  /// `amount` - The amount of the deposit
  async marginWithdraw({
    marginAccount,
    destination,
    amount
  }: {
    marginAccount: MarginAccount
    destination: Address
    amount: PoolAmount
  }) {
    const destinationAddress = translateAddress(destination)

    // FIXME: can be getPosition
    const { address: source } = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)

    const isDestinationNative = AssociatedToken.isNative(marginAccount.owner, this.tokenMint, destinationAddress)

    let marginWithdrawDestination: PublicKey
    if (isDestinationNative) {
      marginWithdrawDestination = AssociatedToken.derive(this.tokenMint, marginAccount.owner)
    } else {
      marginWithdrawDestination = destinationAddress
    }

    const instructions: TransactionInstruction[] = []
    await marginAccount.withUpdateAllPositionBalances({ instructions })

    await AssociatedToken.withCreate(instructions, marginAccount.provider, marginAccount.owner, this.tokenMint)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterProgram: this.programs.config.marginPoolProgramId,
      adapterMetadata: this.addresses.marginPoolAdapterMetadata,
      adapterInstruction: await this.makeMarginWithdrawInstruction({
        marginAccount: marginAccount.address,
        source,
        destination: marginWithdrawDestination,
        amount
      })
    })

    if (isDestinationNative) {
      AssociatedToken.withClose(instructions, marginAccount.owner, this.tokenMint, destinationAddress)
    }

    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  async makeMarginWithdrawInstruction({
    marginAccount,
    source,
    destination,
    amount
  }: {
    marginAccount: Address
    source: Address
    destination: Address
    amount: PoolAmount
  }): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .withdraw(amount.toRpcArg())
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

  async makeRegisterLoanInstruction(marginAccount: MarginAccount): Promise<[Address, TransactionInstruction]> {
    const loanNoteAccount = findDerivedAccount(
      this.programs.config.marginPoolProgramId,
      marginAccount.address,
      this.addresses.loanNoteMint
    )
    return [
      loanNoteAccount,
      await this.programs.marginPool.methods
        .registerLoan()
        .accounts({
          marginAccount: marginAccount.address,
          marginPool: this.address,
          loanNoteMint: this.addresses.loanNoteMint,
          loanNoteAccount: loanNoteAccount,
          payer: marginAccount.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .instruction()
    ]
  }

  async closePosition({ marginAccount, destination }: { marginAccount: MarginAccount; destination: Address }) {
    await marginAccount.refresh()

    const position = await marginAccount.getPosition(this.addresses.depositNoteMint)

    if (position) {
      if (position.balance.gt(ZERO_BN)) {
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
            amount: PoolAmount.notes(position.balance)
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
