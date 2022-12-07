import { Program, BN, Address } from "@project-serum/anchor"
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from "@solana/web3.js"
import { bigIntToBn, bnToBigInt, MarginAccount } from "@jet-lab/margin"
import { Orderbook } from "./orderbook"
import { JetMarket } from "./types"
import { fetchData, findDerivedAccount } from "./utils"
import { order_id_to_string, rate_to_price } from "./wasm-utils/wasm_utils"

export const U64_MAX = 18_446_744_073_709_551_615n
export interface OrderParams {
  maxMarketTicketQty: BN
  maxUnderlyingTokenQty: BN
  limitPrice: BN
  matchLimit: BN
  postOnly: boolean
  postAllowed: boolean
  autoStake: boolean
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
  marketTicketMint: PublicKey
  claimsMint: PublicKey
  collateralMint: PublicKey
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
  nextNewTermLoanSeqNo: BN
  nextUnpaidTermLoanSeqNo: BN
  nextTermLoanMaturity: BN
  pending: BN
  committed: BN
}

export interface AssetInfo {
  entitledTokens: BN
  entitledTickets: BN
  _reserved0: number[]
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
    marketTicketMint: PublicKey
    claimsMint: PublicKey
    claimsMetadata: PublicKey
    collateralMint: PublicKey
    collateralMetadata: PublicKey
    underlyingOracle: PublicKey
    ticketOracle: PublicKey
    marginAdapterMetadata: PublicKey
  }
  readonly info: MarketInfo
  readonly program: Program<JetMarket>
  private constructor(
    market: PublicKey,
    claimsMetadata: PublicKey,
    collateralMetadata: PublicKey,
    marginAdapterMetadata: PublicKey,
    program: Program<JetMarket>,
    info: MarketInfo
  ) {
    this.addresses = {
      ...info,
      claimsMetadata,
      collateralMetadata,
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
   * @param program The anchor `JetMarket` program
   * @param market The address of the `market` account
   * @returns
   */
  static async load(program: Program<JetMarket>, market: Address, jetMarginProgramId: Address): Promise<FixedTermMarket> {
    let data = await fetchData(program.provider.connection, market)
    let info: MarketInfo = program.coder.accounts.decode("Market", data)
    const claimsMetadata = await findDerivedAccount(
      ["token-config", info.airspace, info.claimsMint],
      new PublicKey(jetMarginProgramId)
    )
    const collateralMetadata = await findDerivedAccount(
      ["token-config", info.airspace, info.collateralMint],
      new PublicKey(jetMarginProgramId)
    )
    const marginAdapterMetadata = await findDerivedAccount([program.programId], new PublicKey(jetMarginProgramId))

    return new FixedTermMarket(
      new PublicKey(market),
      new PublicKey(claimsMetadata),
      new PublicKey(collateralMetadata),
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
    seed: Uint8Array,
    tenor: number
  ): Promise<TransactionInstruction> {
    const limitPrice = new BN(rate_to_price(BigInt(rate.toString()), BigInt(tenor)).toString())
    const params: OrderParams = {
      maxMarketTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: true,
      autoStake: true
    }
    return await this.borrowIx(user, payer, params, seed)
  }

  async borrowNowIx(
    user: MarginAccount,
    payer: Address,
    amount: BN,
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    // TODO: rethink amounts here, current is placeholder
    const params: OrderParams = {
      maxMarketTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice: new BN(0.00001),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: false,
      autoStake: true
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
    const collateral = await this.deriveMarginUserCollateral(marginUser)

    return this.program.methods
      .marginBorrowOrder(params, Buffer.from(seed))
      .accounts({
        ...this.addresses,
        orderbookMut: this.orderbookMut(),
        marginUser,
        marginAccount: user.address,
        termLoan,
        claims,
        collateral,
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
    seed: Uint8Array,
    tenor: number
  ): Promise<TransactionInstruction> {
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.marketTicketMint, user.address, true)
    const limitPrice = bigIntToBn(rate_to_price(bnToBigInt(rate), BigInt(tenor)))
    const params: OrderParams = {
      maxMarketTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: true,
      autoStake: true
    }
    return await this.lendIx(user, userTicketVault, userTokenVault, payer, params, seed)
  }

  async lendNowIx(user: MarginAccount, amount: BN, payer: Address, seed: Uint8Array): Promise<TransactionInstruction> {
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.marketTicketMint, user.address, true)
    const params: OrderParams = {
      maxMarketTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice: new BN(2 ** 32),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: false,
      autoStake: true
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
      ticketSettlement = await this.deriveSplitTicket(marketUser, seed)
    }
    const collateral = await this.deriveMarginUserCollateral(marketUser)
    return await this.program.methods
      .marginLendOrder(params, Buffer.from(seed))
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        collateralMint: this.addresses.collateralMint,
        collateral,
        inner: {
          ...this.addresses,
          orderbookMut: this.orderbookMut(),
          authority: user.address,
          payer,
          ticketMint: this.addresses.marketTicketMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          lenderTokens: userTokenVault,
          ticketSettlement
        }
      })
      .instruction()
  }

  async settle(user: MarginAccount) {
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.marketTicketMint, user.address, true)
    const marketUser = await this.deriveMarginUserAddress(user)
    const collateral = await this.deriveMarginUserCollateral(marketUser)
    const claims = await this.deriveMarginUserClaims(marketUser)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    return this.program.methods
      .settle()
      .accounts({
        ...this.addresses,
        marginUser: marketUser,
        collateral,
        tokenProgram: TOKEN_PROGRAM_ID,
        claims,
        underlyingSettlement,
        ticketSettlement
      })
      .instruction()
  }

  async cancelOrderIx(user: MarginAccount, orderId: Uint8Array): Promise<TransactionInstruction> {
    const bnOrderId = new BN(order_id_to_string(orderId))
    return await this.program.methods
      .cancelOrder(bnOrderId)
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
    const borrowerAccount = await this.deriveMarginUserAddress(user)
    const claims = await this.deriveMarginUserClaims(borrowerAccount)
    const collateral = await this.deriveMarginUserCollateral(borrowerAccount)
    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.marketTicketMint, user.address, true)
    return await this.program.methods
      .initializeMarginUser()
      .accounts({
        ...this.addresses,
        borrowerAccount,
        marginAccount: user.address,
        claims,
        collateral,
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

  async deriveMarginUserAddress(user: MarginAccount): Promise<PublicKey> {
    return await findDerivedAccount(["margin_borrower", this.address, user.address], this.program.programId)
  }

  async deriveMarginUserClaims(borrowerAccount: Address): Promise<PublicKey> {
    return await findDerivedAccount(["claim_notes", borrowerAccount], this.program.programId)
  }

  async deriveMarginUserCollateral(borrowerAccount: Address): Promise<PublicKey> {
    return await findDerivedAccount(["collateral_notes", borrowerAccount], this.program.programId)
  }

  async deriveTermLoanAddress(borrowerAccount: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findDerivedAccount(["term_loan", borrowerAccount, seed], this.program.programId)
  }

  async deriveClaimTicketKey(ticketHolder: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findDerivedAccount(
      ["claim_ticket", this.address, new PublicKey(ticketHolder), seed],
      this.program.programId
    )
  }

  async deriveSplitTicket(user: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findDerivedAccount(["split_ticket", user, seed], this.program.programId)
  }

  async fetchOrderbook(): Promise<Orderbook> {
    return await Orderbook.load(this)
  }

  async fetchMarginUser(user: MarginAccount): Promise<MarginUserInfo | null> {
    let data = (await this.provider.connection.getAccountInfo(await this.deriveMarginUserAddress(user)))?.data

    return data ? await this.program.coder.accounts.decode("MarginUser", data) : null
  }
}
