import { Program, BN, Address } from "@project-serum/anchor"
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js"
import { FixedTermMarketConfig, MarginAccount, MarginTokenConfig, Pool } from "../margin"
import { JetFixedTerm } from "./types"
import { fetchData, findFixedTermDerivedAccount, translateWasmInstruction } from "./utils"
import {
  MakerSimulation,
  OrderbookModel,
  OrderbookSnapshot,
  TakerSimulation,
  rate_to_price,
  MarketInfo,
  MarginUserInfo,
  deserializeMarketFromBuffer,
  deserializeMarginUserFromBuffer,
  initializeMarginUserIx,
  WasmTransactionInstruction,
  configureAutoRollLendIx,
  configureAutoRollBorrowIx
} from "../wasm"
import { AssociatedToken, bigIntToBn, bnToBigInt } from "../token"

export const U64_MAX = 18_446_744_073_709_551_615n
export interface OrderParams {
  maxTicketQty: BN
  maxUnderlyingTokenQty: BN
  limitPrice: BN
  matchLimit: BN
  postOnly: boolean
  postAllowed: boolean
  autoStake: boolean
  autoRoll: boolean
}

export interface DebtInfo {
  nextNewTermLoanSeqno: BN
  nextUnpaidTermLoanSeqno: BN
  nextTermLoanMaturity: BN
  pending: BN
  committed: BN
}

export interface AssetInfo {
  entitledTokens: BN
  entitledTickets: BN
  nextDepositSeqno: BN
  nextUnredeemedDepositSeqno: BN
  ticketsStaked: BN
  postedQuote: BN
  _reserved0: number[]
}

export interface LendAutoRollConfig {
  limitPrice: BN
}

export interface BorrowAutoRollConfig {
  limitPrice: BN
  rollTenor: BN
}

export interface ClaimTicket {
  owner: PublicKey
  market: PublicKey
  maturationTimestamp: BN
  redeemable: BN
}

/**
 * Class for loading and interacting with a FixedTermMarket
 */
export class FixedTermMarket {
  readonly addresses: {
    market: PublicKey
    orderbookMarketState: PublicKey
    eventQueue: PublicKey
    asks: PublicKey
    bids: PublicKey
    underlyingTokenMint: PublicKey
    underlyingTokenVault: PublicKey
    feeVault: PublicKey
    ticketMint: PublicKey
    claimsMint: PublicKey
    claimsMetadata: PublicKey
    ticketCollateralMint: PublicKey
    ticketCollateralMetadata: PublicKey
    underlyingCollateralMint: PublicKey
    underlyingCollateralMetadata: PublicKey
    underlyingOracle: PublicKey
    ticketOracle: PublicKey
    marginAdapterMetadata: PublicKey
  }
  readonly info: MarketInfo
  readonly program: Program<JetFixedTerm>
  public orderbookModel: OrderbookModel | undefined = undefined
  private constructor(
    market: PublicKey,
    claimsMetadata: PublicKey,
    ticketCollateralMetadata: PublicKey,
    underlyingCollateralMetadata: PublicKey,
    marginAdapterMetadata: PublicKey,
    program: Program<JetFixedTerm>,
    info: MarketInfo
  ) {
    this.addresses = {
      orderbookMarketState: new PublicKey(info.orderbookMarketState),
      eventQueue: new PublicKey(info.eventQueue),
      asks: new PublicKey(info.asks),
      bids: new PublicKey(info.bids),
      underlyingTokenMint: new PublicKey(info.underlyingTokenMint),
      underlyingTokenVault: new PublicKey(info.underlyingTokenVault),
      feeVault: new PublicKey(info.feeVault),
      ticketMint: new PublicKey(info.ticketMint),
      claimsMint: new PublicKey(info.claimsMint),
      ticketCollateralMint: new PublicKey(info.ticketCollateralMint),
      underlyingCollateralMint: new PublicKey(info.underlyingCollateralMint),
      underlyingOracle: new PublicKey(info.underlyingOracle),
      ticketOracle: new PublicKey(info.ticketOracle),
      claimsMetadata,
      ticketCollateralMetadata,
      underlyingCollateralMetadata,
      marginAdapterMetadata,
      market
    }
    this.program = program
    this.info = info
  }

  get address() {
    return this.addresses.market
  }

  get provider() {
    return this.program.provider
  }

  /**
   * Loads the program state from on chain and returns a `FixedTermMarket` client
   * class for interaction with the market
   *
   * @param program The anchor `JetFixedTerm` program
   * @param market The address of the `market` account
   * @returns
   */
  static async load(
    program: Program<JetFixedTerm>,
    market: Address,
    jetMarginProgramId: Address
  ): Promise<FixedTermMarket> {
    const data = await fetchData(program.provider.connection, market)
    const info: MarketInfo = deserializeMarketFromBuffer(data)
    const claimsMetadata = await findFixedTermDerivedAccount(
      ["token-config", new PublicKey(info.airspace), new PublicKey(info.claimsMint)],
      new PublicKey(jetMarginProgramId)
    )
    const ticketCollateralMetadata = await findFixedTermDerivedAccount(
      ["token-config", new PublicKey(info.airspace), new PublicKey(info.ticketCollateralMint)],
      new PublicKey(jetMarginProgramId)
    )
    const underlyingCollateralMetadata = await findFixedTermDerivedAccount(
      ["token-config", new PublicKey(info.airspace), new PublicKey(info.underlyingCollateralMint)],
      new PublicKey(jetMarginProgramId)
    )
    const marginAdapterMetadata = await findFixedTermDerivedAccount(
      [program.programId],
      new PublicKey(jetMarginProgramId)
    )

    return new FixedTermMarket(
      new PublicKey(market),
      new PublicKey(claimsMetadata),
      new PublicKey(ticketCollateralMetadata),
      new PublicKey(underlyingCollateralMetadata),
      new PublicKey(marginAdapterMetadata),
      program,
      info
    )
  }

  async requestBorrowIx(
    user: MarginAccount,
    payer: Address,
    amount: BN,
    rate: BN,
    tenor: number,
    autoRoll: boolean = false
  ): Promise<TransactionInstruction> {
    const seed = await this.fetchDebtSeed(user)
    const limitPrice = new BN(rate_to_price(BigInt(rate.toString()), BigInt(tenor)).toString())
    const params: OrderParams = {
      maxTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: true,
      autoStake: true,
      autoRoll
    }
    return await this.borrowIx(user, payer, params, seed)
  }

  async borrowNowIx(
    user: MarginAccount,
    payer: Address,
    amount: BN,
    autoRoll: boolean = false
  ): Promise<TransactionInstruction> {
    const seed = await this.fetchDebtSeed(user)
    // TODO: rethink amounts here, current is placeholder
    const params: OrderParams = {
      maxTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice: new BN(0.00001),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: false,
      autoStake: true,
      autoRoll
    }
    return await this.borrowIx(user, payer, params, seed)
  }

  async borrowIx(
    user: MarginAccount,
    payer: Address,
    params: OrderParams,
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    const marginUser = await this.deriveMarginUserAddress(user)
    const termLoan = await this.deriveTermLoanAddress(marginUser, seed)
    const claims = await this.deriveMarginUserClaims(marginUser)
    const tokenCollateral = await this.deriveTokenCollateral(marginUser)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)

    return this.program.methods
      .marginBorrowOrder(params)
      .accounts({
        ...this.addresses,
        orderbookMut: this.orderbookMut(),
        marginUser,
        marginAccount: user.address,
        termLoan,
        claims,
        tokenCollateral,
        payer,
        underlyingSettlement: underlyingSettlement,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenCollateralMint: this.addresses.underlyingCollateralMint
      })
      .instruction()
  }

  async offerLoanIx(
    user: MarginAccount,
    amount: BN,
    rate: BN,
    payer: Address,
    tenor: number,
    autoRoll: boolean = false
  ): Promise<TransactionInstruction> {
    const seed = await this.fetchDepositSeed(user)
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.ticketMint, user.address, true)
    const limitPrice = bigIntToBn(rate_to_price(bnToBigInt(rate), BigInt(tenor)))
    const params: OrderParams = {
      maxTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: true,
      autoStake: true,
      autoRoll
    }
    return await this.lendIx(user, userTicketVault, userTokenVault, payer, params, seed)
  }

  async lendNowIx(
    user: MarginAccount,
    amount: BN,
    payer: Address,
    autoRoll: boolean = false
  ): Promise<TransactionInstruction> {
    const seed = await this.fetchDepositSeed(user)
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.ticketMint, user.address, true)
    const params: OrderParams = {
      maxTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice: new BN(2 ** 32),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: false,
      autoStake: true,
      autoRoll
    }
    return await this.lendIx(user, userTicketVault, userTokenVault, payer, params, seed)
  }

  async lendIx(
    user: MarginAccount,
    userTicketVault: Address,
    userTokenVault: Address,
    payer: Address,
    params: OrderParams,
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    let ticketSettlement = userTicketVault
    const marketUser = await this.deriveMarginUserAddress(user)
    if (params.autoStake) {
      ticketSettlement = await this.deriveTermDepositAddress(user.address, seed)
    }
    const ticketCollateral = await this.deriveTicketCollateral(marketUser)
    return await this.program.methods
      .marginLendOrder(params)
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        marginAccount: user.address,
        ticketCollateralMint: this.addresses.ticketCollateralMint,
        ticketCollateral,
        ticketSettlement,
        lenderTokens: userTokenVault,
        orderbookMut: this.orderbookMut(),
        payer,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .instruction()
  }

  async settle(user: MarginAccount) {
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.ticketMint, user.address, true)
    const marketUser = await this.deriveMarginUserAddress(user)
    const ticketCollateral = await this.deriveTicketCollateral(marketUser)
    const tokenCollateral = await this.deriveTokenCollateral(marketUser)
    const claims = await this.deriveMarginUserClaims(marketUser)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    return this.program.methods
      .settle()
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        marginAccount: user.address,
        ticketCollateral,
        tokenCollateral,
        tokenProgram: TOKEN_PROGRAM_ID,
        claims,
        underlyingSettlement,
        ticketSettlement
      })
      .instruction()
  }

  async repay({
    user,
    termLoan,
    nextTermLoan,
    payer,
    source,
    amount
  }: {
    user: MarginAccount
    termLoan: Address
    nextTermLoan: Address
    payer: Address
    source: Address
    amount: BN
  }) {
    const marketUser = await this.deriveMarginUserAddress(user)
    return this.program.methods
      .repay(amount)
      .accounts({
        marginUser: marketUser,
        termLoan,
        nextTermLoan,
        source,
        sourceAuthority: user.address,
        payer,
        underlyingTokenVault: this.addresses.underlyingTokenVault,
        claims: await this.deriveMarginUserClaims(marketUser),
        claimsMint: this.addresses.claimsMint,
        market: this.address,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async cancelOrderIx(user: MarginAccount, orderId: BN): Promise<TransactionInstruction> {
    return await this.program.methods
      .cancelOrder(orderId)
      .accounts({
        ...this.addresses,
        owner: user.address,
        orderbookMut: this.orderbookMut()
      })
      .instruction()
  }

  orderbookMut() {
    return {
      market: this.addresses.market,
      orderbookMarketState: this.addresses.orderbookMarketState,
      eventQueue: this.addresses.eventQueue,
      bids: this.addresses.bids,
      asks: this.addresses.asks
    }
  }

  async registerAccountWithMarket(user: MarginAccount, payer: Address): Promise<TransactionInstruction> {
    const ix: WasmTransactionInstruction = initializeMarginUserIx(
      user.address.toBase58(),
      this.addresses.market.toBase58(),
      this.info.airspace,
      payer.toString()
    )
    return translateWasmInstruction(ix)

    // const marginUser = await this.deriveMarginUserAddress(user)
    // const claims = await this.deriveMarginUserClaims(marginUser)
    // const ticketCollateral = await this.deriveTicketCollateral(marginUser)
    // const tokenCollateral = await this.deriveTokenCollateral(marginUser)

    // return await this.program.methods
    //   .initializeMarginUser()
    //   .accounts({
    //     ...this.addresses,
    //     marginUser,
    //     marginAccount: user.address,
    //     claims,
    //     ticketCollateral,
    //     tokenCollateral,
    //     payer,
    //     rent: SYSVAR_RENT_PUBKEY,
    //     systemProgram: SystemProgram.programId,
    //     tokenProgram: TOKEN_PROGRAM_ID
    //   })
    //   .instruction()
  }

  /**
   *
   * @param user the margin account to refresh
   * @param expectPrice in the edge case where we want to refresh an account with a broken oracle, set to false
   * @returns a `TransactionInstruction` to refresh the fixed term market related margin positions
   */
  async refreshPosition(user: MarginAccount, expectPrice: boolean): Promise<TransactionInstruction> {
    const marginUser = await this.deriveMarginUserAddress(user)
    return await this.program.methods
      .refreshPosition(expectPrice)
      .accounts({
        ...this.addresses,
        marginUser,
        marginAccount: user.address,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async fetchDebtSeed(user: MarginAccount): Promise<Uint8Array> {
    let userInfo = await this.fetchMarginUser(user)

    if (!userInfo) {
      return new BN(0).toArrayLike(Buffer, "le", 8)
    }

    return bigIntToBn(userInfo.debt.nextNewTermLoanSeqno).toArrayLike(Buffer, "le", 8)
  }

  async fetchDepositSeed(user: MarginAccount): Promise<Uint8Array> {
    let userInfo = await this.fetchMarginUser(user)

    if (!userInfo) {
      return new BN(0).toArrayLike(Buffer, "le", 8)
    }

    return bigIntToBn(userInfo.assets.nextDepositSeqno).toArrayLike(Buffer, "le", 8)
  }

  async deriveMarginUserAddress(user: MarginAccount): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["margin_user", this.address, user.address], this.program.programId)
  }

  async deriveMarginUserClaims(marginUser: Address): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["claim_notes", marginUser], this.program.programId)
  }

  async deriveTicketCollateral(marginUser: Address): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["ticket_collateral_notes", marginUser], this.program.programId)
  }

  async deriveTokenCollateral(marginUser: Address): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["underlying_collateral_notes", marginUser], this.program.programId)
  }

  async deriveTermLoanAddress(marginUser: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["term_loan", this.address, marginUser, seed], this.program.programId)
  }

  async deriveTermDepositAddress(marginAccount: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(
      ["term_deposit", this.address, marginAccount, seed],
      this.program.programId
    )
  }

  getOrderbookModel(tenor: bigint, snapshot: OrderbookSnapshot): OrderbookModel {
    const originationFee = this.info.originationFee
    const model = new OrderbookModel(BigInt(tenor), originationFee)
    model.refreshFromSnapshot(snapshot)
    this.orderbookModel = model

    return model
  }

  async fetchMarginUser(user: MarginAccount): Promise<MarginUserInfo | null> {
    let data = (await this.provider.connection.getAccountInfo(await this.deriveMarginUserAddress(user)))?.data
    const acc = data ? deserializeMarginUserFromBuffer(data) : null
    return acc
  }

  async configAutorollBorrow(marginAccount: MarginAccount, price: bigint, tenor: BN) {
    return translateWasmInstruction(
      configureAutoRollBorrowIx(
        this.addresses.market.toBase58(),
        marginAccount.address.toBase58(),
        marginAccount.owner.toBase58(),
        bnToBigInt(tenor),
        price
      )
    )
  }

  async configAutorollLend(marginAccount: MarginAccount, price: bigint) {
    return translateWasmInstruction(
      configureAutoRollLendIx(
        this.addresses.market.toBase58(),
        marginAccount.address.toBase58(),
        marginAccount.owner.toBase58(),

        price
      )
    )
  }

  async redeemDeposit(
    marginAccount: MarginAccount,
    deposit: {
      id: number
      address: string
      sequence_number: number
      maturation_timestamp: number
      principal: number
      interest: number
      rate: number
      payer: string
      created_timestamp: number
    },
    market: FixedTermMarket
  ) {
    const tokenAccount = AssociatedToken.derive(market.addresses.underlyingTokenMint, marginAccount.address)
    const marginUser = await this.deriveMarginUserAddress(marginAccount)
    const ticketCollateral = await this.deriveTicketCollateral(marginUser)
    const ticketCollateralMint = market.addresses.ticketCollateralMint

    const marginUserData = await market.fetchMarginUser(marginAccount)
    console.table({
      nextUnredeemedDepositSeqno: bigIntToBn(marginUserData?.assets.nextUnredeemedDepositSeqno).toNumber(),
      nextDepositSeqno: bigIntToBn(marginUserData?.assets.nextDepositSeqno).toNumber(),
      deposit_seq_no: deposit.sequence_number
    })

    console.table({
      deposit: deposit.address,
      owner: marginUser.toBase58(),
      authority: marginAccount.address.toBase58(),
      payer: deposit.payer,
      tokenAccount: tokenAccount.toBase58(),
      market: market.address.toBase58(),
      underlyingTokenVault: market.addresses.underlyingTokenVault.toBase58(),
      tokenProgram: TOKEN_PROGRAM_ID.toBase58(),
      ticketCollateral: ticketCollateral.toBase58(),
      ticketCollateralMint: ticketCollateralMint.toBase58()
    })
    return await this.program.methods
      .marginRedeemDeposit()
      .accounts({
        marginUser,
        ticketCollateral,
        ticketCollateralMint,
        inner: {
          permit: marginAccount.findAirspacePermitAddress(),
          deposit: deposit.address,
          owner: marginUser,
          authority: marginAccount.address,
          payer: deposit.payer,
          tokenAccount,
          market: market.address,
          underlyingTokenVault: market.addresses.underlyingTokenVault,
          tokenProgram: TOKEN_PROGRAM_ID
        }
      })
      .instruction()
  }

  async toggleAutorollDeposit(marginAccount: MarginAccount, deposit: Address) {
    return await this.program.methods
      .toggleAutoRollDeposit()
      .accounts({
        marginAccount: marginAccount.address,
        deposit
      })
      .instruction()
  }

  async toggleAutorollLoan(marginAccount: MarginAccount, loan: Address) {
    const marginUser = await this.deriveMarginUserAddress(marginAccount)
    return await this.program.methods
      .toggleAutoRollLoan()
      .accounts({
        marginAccount: marginAccount.address,
        marginUser: marginUser.toBase58(),
        loan
      })
      .instruction()
  }
}

export interface MarketAndConfig {
  market: FixedTermMarket
  config: FixedTermMarketConfig
  token: MarginTokenConfig
  name: string
}

export class FixedTermProductModel {
  private marginAccount: MarginAccount
  private pool: Pool
  private collateralWeight: number
  private requiredCollateralFactor: number

  public static fromMarginAccountPool(marginAccount: MarginAccount, pool: Pool): FixedTermProductModel {
    return new FixedTermProductModel(
      marginAccount,
      pool.depositNoteMetadata.valueModifier,
      pool.loanNoteMetadata.valueModifier,
      pool
    )
  }

  public constructor(
    marginAccount: MarginAccount,
    collateralWeight: number,
    requiredCollateralFactor: number,
    pool: Pool
  ) {
    this.marginAccount = marginAccount
    this.collateralWeight = collateralWeight
    this.requiredCollateralFactor = requiredCollateralFactor
    this.pool = pool
  }

  private tokenPrice(): number {
    return this.pool.info?.tokenPriceOracle.aggregate.price || NaN
  }

  private tokens(lamports: bigint): number {
    const tokenDecimals = this.pool.info?.tokenMint.decimals || 0

    return Number(lamports) / 10 ** tokenDecimals
  }

  private valueOf(lamports: bigint): number {
    return this.tokens(lamports) * this.tokenPrice()
  }

  takerAccountForecast(
    action: "lend" | "borrow",
    sim: TakerSimulation,
    mode: "setup" | "maintenance" = "maintenance"
  ): ValuationEstimate {
    let delta: ValuationDelta

    if (action == "lend") {
      const principalAmount = this.valueOf(sim.filledQuoteQty)
      const repaymentAmount = this.valueOf(sim.filledBaseQty)
      const principalWeight = this.collateralWeight
      const repaymentWeight = this.collateralWeight

      delta = this.accounting.termDeposit(principalAmount, repaymentAmount, principalWeight, repaymentWeight)
    } else if (action == "borrow") {
      // TODO Fees
      const receivedAmount = this.valueOf(sim.filledQuoteQty)
      const repaymentAmount = this.valueOf(sim.filledBaseQty)
      const receivedWeight = this.collateralWeight
      let repaymentFactor = this.requiredCollateralFactor
      if (mode == "setup") {
        repaymentFactor *= MarginAccount.SETUP_LEVERAGE_FRACTION
      }

      delta = this.accounting.termLoan(receivedAmount, repaymentAmount, receivedWeight, repaymentFactor)
    } else {
      throw Error("unreachable")
    }

    const estimate = this.accounting.apply(this.marginAccount, delta, mode)
    return estimate
  }

  makerAccountForecast(
    action: "lend" | "borrow",
    sim: MakerSimulation,
    mode: "setup" | "maintenance" = "maintenance"
  ): ValuationEstimate {
    let delta: ValuationDelta

    if (action == "lend") {
      const fillPrincipalAmount = this.valueOf(sim.filledQuoteQty)
      const fillRepaymentAmount = this.valueOf(sim.filledBaseQty)
      const postPrincipalAmount = this.valueOf(sim.postedQuoteQty)
      const postRepaymentAmount = this.valueOf(sim.postedBaseQty)

      const principalWeight = this.collateralWeight
      const repaymentWeight = this.requiredCollateralFactor

      delta = this.accounting.merge(
        this.accounting.termDeposit(fillPrincipalAmount, fillRepaymentAmount, principalWeight, repaymentWeight),
        this.accounting.loanOffer(postPrincipalAmount, postRepaymentAmount, principalWeight, repaymentWeight)
      )
    } else if (action == "borrow") {
      // TODO Fees
      const fillReceivedAmount = this.valueOf(sim.filledQuoteQty)
      const fillRepaymentAmount = this.valueOf(sim.filledBaseQty)
      const postReceivedAmount = this.valueOf(sim.postedQuoteQty)
      const postRepaymentAmount = this.valueOf(sim.postedBaseQty)

      const receivedWeight = this.collateralWeight
      let repaymentFactor = this.requiredCollateralFactor
      if (mode == "setup") {
        repaymentFactor *= MarginAccount.SETUP_LEVERAGE_FRACTION
      }

      delta = this.accounting.merge(
        this.accounting.termLoan(fillReceivedAmount, fillRepaymentAmount, receivedWeight, repaymentFactor),
        this.accounting.loanRequest(postReceivedAmount, postRepaymentAmount, receivedWeight, repaymentFactor)
      )
    } else {
      throw Error("unreachable")
    }

    const estimate = this.accounting.apply(this.marginAccount, delta, mode)

    return estimate
  }

  private accounting = {
    // FIXME Naive implementation below. Lending especially needs attention.

    termDeposit(
      principalAmount: number,
      repaymentAmount: number,
      principalWeight: number,
      repaymentWeight: number
    ): ValuationDelta {
      // Term deposits are not current credited as collateral, pending a good solution to the ALM
      // problem posed by their liquidation. Until then, use a weight of zero in the forecast.
      repaymentWeight = 0

      return {
        liabilities: 0,
        requiredCollateral: 0,
        weightedCollateral: repaymentWeight * repaymentAmount - principalWeight * principalAmount
      }
    },

    loanOffer(
      principalAmount: number,
      repaymentAmount: number,
      principalWeight: number,
      repaymentWeight: number
    ): ValuationDelta {
      return this.termDeposit(principalAmount, repaymentAmount, principalWeight, repaymentWeight)
    },

    termLoan(
      receivedAmount: number,
      repaymentAmount: number,
      receivedWeight: number,
      repaymentFactor: number
    ): ValuationDelta {
      return {
        liabilities: repaymentAmount,
        requiredCollateral: repaymentAmount / repaymentFactor,
        weightedCollateral: receivedWeight * receivedAmount
      }
    },

    loanRequest(
      receivedAmount: number,
      repaymentAmount: number,
      receivedWeight: number,
      repaymentFactor: number
    ): ValuationDelta {
      return this.termLoan(receivedAmount, repaymentAmount, receivedWeight, repaymentFactor)
    },

    apply(account: MarginAccount, delta: ValuationDelta, mode: "setup" | "maintenance"): ValuationEstimate {
      let assets = account.valuation.assets
      let liabilities = account.valuation.liabilities

      let weightedCollateral = account.valuation.weightedCollateral
      let requiredCollateral: number
      if (mode == "setup") {
        requiredCollateral = account.valuation.requiredSetupCollateral
      } else if (mode == "maintenance") {
        requiredCollateral = account.valuation.requiredCollateral
      } else {
        throw Error("unreachable")
      }

      liabilities += delta.liabilities
      requiredCollateral += delta.requiredCollateral
      weightedCollateral += delta.weightedCollateral

      const equity = assets - liabilities
      const availableCollateral = weightedCollateral - (liabilities + requiredCollateral)

      let riskIndicator = NaN
      if (requiredCollateral >= 0 && weightedCollateral >= 0 && liabilities >= 0) {
        riskIndicator = account.computeRiskIndicator(requiredCollateral, weightedCollateral, liabilities)
      } else {
        console.error("Unexpected state in forecast accounting")
      }

      return {
        assets,
        liabilities,
        equity,
        requiredCollateral,
        weightedCollateral,
        availableCollateral,
        riskIndicator
      }
    },

    merge(delta1: ValuationDelta, delta2: ValuationDelta): ValuationDelta {
      return {
        liabilities: delta1.liabilities + delta2.liabilities,
        requiredCollateral: delta1.requiredCollateral + delta2.requiredCollateral,
        weightedCollateral: delta1.weightedCollateral + delta2.weightedCollateral
      }
    }
  }
}

export interface ValuationEstimate {
  assets: number
  liabilities: number
  equity: number
  requiredCollateral: number
  weightedCollateral: number
  availableCollateral: number
  riskIndicator: number
}

// TODO When porting to the rust client, might be better to use position deltas.
export interface ValuationDelta {
  liabilities: number
  requiredCollateral: number
  weightedCollateral: number
}
