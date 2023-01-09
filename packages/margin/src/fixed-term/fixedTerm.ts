import { Program, BN, Address } from "@project-serum/anchor"
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from "@solana/web3.js"
import { FixedTermMarketConfig, MarginAccount, MarginTokenConfig } from "../margin"
import { Orderbook } from "./orderbook"
import { JetFixedTerm } from "./types"
import { fetchData, findFixedTermDerivedAccount } from "./utils"
import { rate_to_price } from "../wasm"
import { bigIntToBn, bnToBigInt } from "../token"

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

/**
 * The raw struct as found on chain
 */
export interface MarketInfo {
  versionTag: BN
  airspace: PublicKey
  orderbookMarketState: PublicKey
  eventQueue: PublicKey
  asks: PublicKey
  bids: PublicKey
  underlyingTokenMint: PublicKey
  underlyingTokenVault: PublicKey
  ticketMint: PublicKey
  claimsMint: PublicKey
  ticketCollateralMint: PublicKey
  underlyingOracle: PublicKey
  ticketOracle: PublicKey
  seed: number[]
  bump: number[]
  orderbookPaused: boolean
  ticketsPaused: boolean
  reserved: number[]
  borrowTenor: BN
  lendTenor: BN
  nonce: BN
}

/** MarginUser account as found on-chain */
export interface MarginUserInfo {
  version: BN
  marginAccount: PublicKey
  market: PublicKey
  claims: PublicKey
  collateral: PublicKey
  underlyingSettlement: PublicKey
  ticketSettlement: PublicKey
  debt: DebtInfo
  assets: AssetInfo
}

export interface DebtInfo {
  nextNewTermLoanSeqno: BN
  nextUnpaidTermLoanSeqno: BN
  nextTermLoanMaturity: BN
  pending: BN
  committed: BN
  borrowRollConfig: AutoRollConfig
  lendRollConfig: AutoRollConfig
}

export interface AssetInfo {
  entitledTokens: BN
  entitledTickets: BN
  nextDepositSeqno: BN
  nextUnredeemedDepositSeqno: BN
  _reserved0: number[]
}

export interface AutoRollConfig {
  limit_price: BN
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
    ticketMint: PublicKey
    claimsMint: PublicKey
    claimsMetadata: PublicKey
    ticketCollateralMint: PublicKey
    ticketCollateralMetadata: PublicKey
    underlyingOracle: PublicKey
    ticketOracle: PublicKey
    marginAdapterMetadata: PublicKey
  }
  readonly info: MarketInfo
  readonly program: Program<JetFixedTerm>
  private constructor(
    market: PublicKey,
    claimsMetadata: PublicKey,
    ticketCollateralMetadata: PublicKey,
    marginAdapterMetadata: PublicKey,
    program: Program<JetFixedTerm>,
    info: MarketInfo
  ) {
    this.addresses = {
      ...info,
      claimsMetadata,
      ticketCollateralMetadata,
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
    let data = await fetchData(program.provider.connection, market)
    let info: MarketInfo = program.coder.accounts.decode("market", data)
    const claimsMetadata = await findFixedTermDerivedAccount(
      ["token-config", info.airspace, info.claimsMint],
      new PublicKey(jetMarginProgramId)
    )
    const ticketCollateralMetadata = await findFixedTermDerivedAccount(
      ["token-config", info.airspace, info.ticketCollateralMint],
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
    const ticketCollateral = await this.deriveTicketCollateral(marginUser)

    return this.program.methods
      .marginBorrowOrder(params)
      .accounts({
        ...this.addresses,
        orderbookMut: this.orderbookMut(),
        marginUser,
        marginAccount: user.address,
        termLoan,
        claims,
        ticketCollateral,
        payer,
        underlyingSettlement: await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true),
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID
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
      ticketSettlement = await this.deriveTermDepositAddress(marketUser, seed)
    }
    const ticketCollateral = await this.deriveTicketCollateral(marketUser)
    return await this.program.methods
      .marginLendOrder(params)
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        ticketCollateralMint: this.addresses.ticketCollateralMint,
        ticketCollateral,
        inner: {
          ...this.addresses,
          orderbookMut: this.orderbookMut(),
          authority: user.address,
          payer,
          ticketMint: this.addresses.ticketMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          lenderTokens: userTokenVault,
          ticketSettlement
        }
      })
      .instruction()
  }

  async settle(user: MarginAccount) {
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.ticketMint, user.address, true)
    const marketUser = await this.deriveMarginUserAddress(user)
    const ticketCollateral = await this.deriveTicketCollateral(marketUser)
    const claims = await this.deriveMarginUserClaims(marketUser)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    return this.program.methods
      .settle()
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        ticketCollateral,
        tokenProgram: TOKEN_PROGRAM_ID,
        claims,
        underlyingSettlement,
        ticketSettlement
      })
      .instruction()
  }

  async repay({
    user, termLoan, nextTermLoan, payer, source, amount
  }: {
    user: MarginAccount,
    termLoan: Address,
    nextTermLoan: Address,
    payer: Address,
    source: Address,
    amount: BN
  }) {
    const marketUser = await this.deriveMarginUserAddress(user)
    return this.program.methods.repay(amount)
      .accounts({
        marginUser: marketUser,
        termLoan,
        nextTermLoan,
        source,
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
    const marginUser = await this.deriveMarginUserAddress(user)
    const claims = await this.deriveMarginUserClaims(marginUser)
    const ticketCollateral = await this.deriveTicketCollateral(marginUser)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.ticketMint, user.address, true)
    return await this.program.methods
      .initializeMarginUser()
      .accounts({
        ...this.addresses,
        marginUser,
        marginAccount: user.address,
        claims,
        ticketCollateral,
        underlyingSettlement,
        ticketSettlement,
        payer,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
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

    return userInfo.debt.nextNewTermLoanSeqno.toArrayLike(Buffer, "le", 8)
  }

  async fetchDepositSeed(user: MarginAccount): Promise<Uint8Array> {
    let userInfo = await this.fetchMarginUser(user)

    if (!userInfo) {
      return new BN(0).toArrayLike(Buffer, "le", 8)
    }

    return userInfo.assets.nextDepositSeqno.toArrayLike(Buffer, "le", 8)
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

  async deriveTermLoanAddress(marginUser: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["term_loan", this.address, marginUser, seed], this.program.programId)
  }

  async deriveTermDepositAddress(marginUser: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findFixedTermDerivedAccount(["term_deposit", this.address, marginUser, seed], this.program.programId)
  }

  async fetchOrderbook(): Promise<Orderbook> {
    return await Orderbook.load(this)
  }

  async fetchMarginUser(user: MarginAccount): Promise<MarginUserInfo | null> {
    let data = (await this.provider.connection.getAccountInfo(await this.deriveMarginUserAddress(user)))?.data

    return data ? await this.program.coder.accounts.decode("marginUser", data) : null
  }
}

export interface MarketAndconfig {
  market: FixedTermMarket
  config: FixedTermMarketConfig
  token: MarginTokenConfig
  name: string
}
