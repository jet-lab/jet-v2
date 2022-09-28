import { Program, BN, Address } from "@project-serum/anchor"
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from "@solana/web3.js"
import { MarginAccount } from "@jet-lab/margin"
import { Orderbook } from "./orderbook"
import { JetBonds } from "./types"
import { fetchData, findDerivedAccount } from "./utils"
import { rate_to_price } from "../wasm-utils/pkg"

export const OrderSideBorrow = { borrow: {} }
export const OrderSideLend = { lend: {} }
export type OrderSide = typeof OrderSideBorrow | typeof OrderSideLend

export const U64_MAX = 18_446_744_073_709_551_615n

export interface OrderParams {
  maxBondTicketQty: BN
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
export interface BondManagerInfo {
  versionTag: BN
  programAuthority: PublicKey
  orderbookMarketState: PublicKey
  eventQueue: PublicKey
  asks: PublicKey
  bids: PublicKey
  underlyingTokenMint: PublicKey
  underlyingTokenVault: PublicKey
  bondTicketMint: PublicKey
  claimsMint: PublicKey
  collateralMint: PublicKey
  underlyingOracle: PublicKey
  ticketOracle: PublicKey
  seed: number[]
  bump: number[]
  orderbookPaused: boolean
  ticketsPaused: boolean
  reserved: number[]
  duration: BN
  nonce: BN
}

/** MarginUser account as found on-chain */
export interface MarginUserInfo {
  version: BN
  marginAccount: PublicKey
  bondManager: PublicKey
  claims: PublicKey
  collateral: PublicKey
  underlyingSettlement: PublicKey
  ticketSettlement: PublicKey
  debt: DebtInfo
  assets: AssetInfo
}

export interface DebtInfo {
  nextNewObligationSeqNo: BN
  nextUnpaidObligationSeqNo: BN
  nextObligationMaturity: BN
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
  bondManager: PublicKey
  maturationTimestamp: BN
  redeemable: BN
}

/**
 * Class for loading and interacting with a BondMarket
 */
export class BondMarket {
  readonly addresses: {
    bondManager: PublicKey
    orderbookMarketState: PublicKey
    eventQueue: PublicKey
    asks: PublicKey
    bids: PublicKey
    underlyingTokenMint: PublicKey
    underlyingTokenVault: PublicKey
    bondTicketMint: PublicKey
    claimsMint: PublicKey
    claimsMetadata: PublicKey
    collateralMint: PublicKey
    underlyingOracle: PublicKey
    ticketOracle: PublicKey
  }
  readonly info: BondManagerInfo
  readonly program: Program<JetBonds>

  private constructor(
    bondManager: PublicKey,
    claimsMetadata: PublicKey,
    program: Program<JetBonds>,
    info: BondManagerInfo
  ) {
    this.addresses = {
      ...info,
      claimsMetadata,
      bondManager
    }
    this.program = program
    this.info = info
  }

  get address() {
    return this.addresses.bondManager
  }

  get provider() {
    return this.program.provider
  }

  /**
   * Loads the program state from on chain and returns a `BondMarket` client
   * class for interaction with the market
   *
   * @param program The anchor `JetBonds` program
   * @param bondManager The address of the `bondManager` account
   * @returns
   */
  static async load(
    program: Program<JetBonds>,
    bondManager: Address,
    jetMetadataProgramId: Address
  ): Promise<BondMarket> {
    let data = await fetchData(program.provider.connection, bondManager)
    let info: BondManagerInfo = program.coder.accounts.decode("BondManager", data)
    const claimsMetadata = await findDerivedAccount([info.claimsMint], new PublicKey(jetMetadataProgramId))

    return new BondMarket(new PublicKey(bondManager), new PublicKey(claimsMetadata), program, info)
  }

  async requestBorrowIx(
    user: MarginAccount,
    payer: Address,
    amount: BN,
    rate: BN,
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    const limitPrice = new BN(rate_to_price(BigInt(rate.toString()), BigInt(this.info.duration.toString())).toString())
    const params: OrderParams = {
      maxBondTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: true,
      postAllowed: false,
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
    // TODO: determine best rate values here
    // const limitPrice = new BN(rate_to_price(U64_MAX, BigInt(this.info.duration.toString())).toString())
    const params: OrderParams = {
      maxBondTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: amount,
      limitPrice: new BN(U64_MAX.toString()),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: true,
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
    const borrowerAccount = await this.deriveMarginUserAddress(user)
    const obligation = await this.deriveObligationAddress(borrowerAccount, seed)
    const claims = await this.deriveMarginUserClaims(borrowerAccount)

    return this.program.methods
      .marginBorrowOrder(params, Buffer.from(seed))
      .accounts({
        ...this.addresses,
        borrowerAccount,
        marginAccount: user.address,
        obligation,
        claims,
        payer,
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
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.bondTicketMint, user.address, true)
    const limitPrice = new BN(rate_to_price(BigInt(rate.toString()), BigInt(this.info.duration.toString())).toString())
    const params: OrderParams = {
      maxBondTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice,
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: true,
      autoStake: true
    }
    return await this.lendIx(user.address, userTicketVault, userTokenVault, payer, params, seed)
  }

  async lendNowIx(user: MarginAccount, amount: BN, payer: Address, seed: Uint8Array): Promise<TransactionInstruction> {
    const userTokenVault = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const userTicketVault = await getAssociatedTokenAddress(this.addresses.bondTicketMint, user.address, true)
    const params: OrderParams = {
      maxBondTicketQty: new BN(U64_MAX.toString()),
      maxUnderlyingTokenQty: new BN(amount),
      limitPrice: new BN(0),
      matchLimit: new BN(U64_MAX.toString()),
      postOnly: false,
      postAllowed: false,
      autoStake: true
    }

    return await this.lendIx(user.address, userTicketVault, userTokenVault, payer, params, seed)
  }

  async lendIx(
    user: Address,
    userTicketVault: Address,
    userTokenVault: Address,
    payer: Address,
    params: OrderParams,
    seed: Uint8Array
  ): Promise<TransactionInstruction> {
    const splitTicket = await this.deriveSplitTicket(user, seed)
    return await this.program.methods
      .lendOrder(params, Buffer.from(seed))
      .accounts({
        ...this.addresses,
        user,
        userTicketVault,
        userTokenVault,
        splitTicket,
        payer,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async cancelOrderIx(user: MarginAccount, orderId: BN, side: OrderSide): Promise<TransactionInstruction> {
    const userVault =
      side === OrderSideBorrow
        ? await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address)
        : await getAssociatedTokenAddress(this.addresses.bondTicketMint, user.address)
    const marketAccount = side === OrderSideBorrow ? this.addresses.underlyingTokenVault : this.addresses.bondTicketMint

    return await this.program.methods
      .cancelOrder(orderId)
      .accounts({
        ...this.addresses,
        user: user.address,
        userVault,
        marketAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async registerAccountWithMarket(user: MarginAccount, payer: Address): Promise<TransactionInstruction> {
    const borrowerAccount = await this.deriveMarginUserAddress(user)
    const claims = await this.deriveMarginUserClaims(borrowerAccount)
    const collateral = await this.deriveMarginUserCollateral(borrowerAccount)

    const underlyingSettlement = await getAssociatedTokenAddress(this.addresses.underlyingTokenMint, user.address, true)
    const ticketSettlement = await getAssociatedTokenAddress(this.addresses.bondTicketMint, user.address, true)
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
   * @returns a `TransactionInstruction` to refresh the bonds related margin positions
   */
  async refreshPosition(user: MarginAccount, expectPrice: boolean): Promise<TransactionInstruction> {
    const borrowerAccount = await this.deriveMarginUserAddress(user)
    return await this.program.methods
      .refreshPosition(expectPrice)
      .accounts({
        ...this.addresses,
        borrowerAccount,
        marginAccount: user.address,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async deriveMarginUserAddress(user: MarginAccount): Promise<PublicKey> {
    return await findDerivedAccount(["margin_borrower", this.address, user.address], this.program.programId)
  }

  async deriveMarginUserClaims(borrowerAccount: Address): Promise<PublicKey> {
    return await findDerivedAccount(["user_claims", borrowerAccount], this.program.programId)
  }

  async deriveMarginUserCollateral(borrowerAccount: Address): Promise<PublicKey> {
    return await findDerivedAccount(["deposit_notes", borrowerAccount], this.program.programId)
  }

  async deriveObligationAddress(borrowerAccount: Address, seed: Uint8Array): Promise<PublicKey> {
    return await findDerivedAccount(["obligation", borrowerAccount, seed], this.program.programId)
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

  async fetchMarginUser(user: MarginAccount): Promise<MarginUserInfo> {
    let data = (await this.provider.connection.getAccountInfo(await this.deriveMarginUserAddress(user)))!.data

    return await this.program.coder.accounts.decode("MarginUser", data)
  }
}
