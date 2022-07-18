import assert from "assert"
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from "@solana/web3.js"
import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { getLayoutVersion, Market as SerumMarket, Orderbook as SerumOrderbook, OpenOrders } from "@project-serum/serum"
import { MarketOptions, Order } from "@project-serum/serum/lib/market"
import {
  closeAccount,
  initializeAccount,
  TOKEN_PROGRAM_ID,
  WRAPPED_SOL_MINT
} from "@project-serum/serum/lib/token-instructions"
import { TokenAmount } from "src/token"
import { MarginMarketConfig, MarginMarkets, MarginTokens } from "../config"
import { PoolTokenChange } from "../pool"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"

export type SelfTradeBehavior = "decrementTake" | "cancelProvide" | "abortTransaction"
export type OrderSide = "sell" | "buy" | "ask" | "bid"
export type OrderType = "limit" | "ioc" | "postOnly"
export type OrderStatus = "open" | "partialFilled" | "filled" | "cancelled"

export class Market {
  programs: MarginPrograms
  marketConfig: MarginMarketConfig
  serum: SerumMarket

  get name(): string {
    return `${this.marketConfig.baseSymbol}/${this.marketConfig.quoteSymbol}`
  }
  get address(): PublicKey {
    return translateAddress(this.marketConfig.market)
  }
  get baseMint(): PublicKey {
    return translateAddress(this.marketConfig.baseMint)
  }
  get baseDecimals(): number {
    return this.marketConfig.baseDecimals
  }
  get baseSymbol(): MarginTokens {
    return this.marketConfig.baseSymbol
  }
  get quoteMint(): PublicKey {
    return translateAddress(this.marketConfig.quoteMint)
  }
  get quoteDecimals(): number {
    return this.marketConfig.quoteDecimals
  }
  get quoteSymbol(): MarginTokens {
    return this.marketConfig.quoteSymbol
  }
  get minOrderSize() {
    return this.baseSizeLotsToNumber(new BN(1))
  }
  get tickSize() {
    return this.priceLotsToNumber(new BN(1))
  }
  private get baseDecimalMultiplier() {
    return new BN(10).pow(new BN(this.baseDecimals))
  }
  private get quoteDecimalMultiplier() {
    return new BN(10).pow(new BN(this.baseDecimals))
  }

  /**
   * Creates a Margin Market
   * @param provider
   * @param programs
   * @param marketConfig
   * @param serum
   */
  constructor(programs: MarginPrograms, marketConfig: MarginMarketConfig, serum: SerumMarket) {
    this.programs = programs
    this.marketConfig = marketConfig
    this.serum = serum
    assert(this.programs.margin.programId)
    assert(this.programs.config.serumProgramId)
    if (!serum.decoded.accountFlags.initialized || !serum.decoded.accountFlags.market) {
      throw new Error("Invalid market state")
    }
  }

  /**
   * Load a Margin Market
   *
   * @param {{
   *     provider: AnchorProvider
   *     programs: MarginPrograms
   *     address: PublicKey
   *     options?: MarketOptions
   *   }}
   * @return {Promise<Market>}
   */
  static async load({
    provider,
    programs,
    address,
    options
  }: {
    provider: AnchorProvider
    programs: MarginPrograms
    address: PublicKey
    options?: MarketOptions
  }): Promise<Market> {
    const marketAccount = await programs.connection.getAccountInfo(address)
    if (!marketAccount) {
      throw new Error("Market not found")
    }
    if (marketAccount.owner.equals(SystemProgram.programId) && marketAccount.lamports === 0) {
      throw new Error("Market account not does not exist")
    }
    if (!marketAccount.owner.equals(translateAddress(programs.config.serumProgramId))) {
      throw new Error("Market address not owned by Serum program: " + marketAccount.owner.toBase58())
    }
    const serum = await SerumMarket.load(
      provider.connection,
      address,
      options,
      translateAddress(programs.config.serumProgramId)
    )
    if (
      !serum.decoded.accountFlags.initialized ||
      !serum.decoded.accountFlags.market ||
      !serum.decoded.ownAddress.equals(address)
    ) {
      throw new Error("Invalid market")
    }
    let marketConfig: MarginMarketConfig | undefined
    for (const market of Object.values(programs.config.markets)) {
      if (translateAddress(market.market).equals(address)) {
        marketConfig = market
      }
    }
    if (!marketConfig) {
      throw new Error("Unable to match market config")
    }

    return new Market(programs, marketConfig, serum)
  }

  /**
   * Load all Margin Markets
   *
   * @param {{
   *     provider: AnchorProvider
   *     programs: MarginPrograms
   *     options?: MarketOptions
   *   }}
   * @return {Promise<Record<MarginMarkets, Market>>}
   */
  static async loadAll({
    provider,
    programs,
    options
  }: {
    provider: AnchorProvider
    programs: MarginPrograms
    options?: MarketOptions
  }): Promise<Record<MarginMarkets, Market>> {
    const markets: Record<MarginMarkets, Market> = {} as Record<MarginMarkets, Market>
    for (const marketConfig of Object.values(programs.config.markets)) {
      const market = await this.load({
        provider,
        programs,
        address: translateAddress(marketConfig.market),
        options
      })
      markets[market.name] = new Market(programs, marketConfig, market.serum)
    }

    return markets
  }

  static encodeOrderSide(side: OrderSide): number {
    switch (side) {
      case "bid":
      case "buy":
        return 0
      case "ask":
      case "sell":
        return 1
    }
  }

  static encodeOrderType(type: OrderType): number {
    switch (type) {
      case "limit":
        return 0
      case "ioc":
        return 1 // market order
      case "postOnly":
        return 2
    }
  }

  static encodeSelfTradeBehavior(behavior: SelfTradeBehavior): number {
    switch (behavior) {
      case "decrementTake":
        return 0
      case "cancelProvide":
        return 1
      case "abortTransaction":
        return 2
    }
  }

  async placeOrder({
    marginAccount,
    orderSide,
    orderType,
    orderPrice,
    orderSize,
    selfTradeBehavior,
    clientOrderId,
    payer
  }: {
    marginAccount: MarginAccount
    orderSide: OrderSide
    orderType: OrderType
    orderPrice: number
    orderSize: TokenAmount
    selfTradeBehavior?: SelfTradeBehavior
    clientOrderId?: BN
    payer?: PublicKey
  }) {
    const instructions: TransactionInstruction[] = []
    const orderAmount = orderSize.divn(orderPrice)
    const accountPoolPosition = marginAccount.poolPositions[this.baseSymbol]

    // If trading on margin
    if (orderAmount.gt(accountPoolPosition.depositBalance) && marginAccount.pools) {
      const difference = orderAmount.sub(accountPoolPosition.depositBalance)
      const pool = marginAccount.pools[this.baseSymbol]
      if (pool) {
        await pool.marginBorrow({
          marginAccount: this,
          pools: Object.values(marginAccount.pools),
          change: PoolTokenChange.setTo(accountPoolPosition.loanBalance.add(difference))
        })
      }
    }

    await this.withPlaceOrder({
      instructions,
      marginAccount,
      orderSide,
      orderType,
      orderPrice,
      orderSize,
      selfTradeBehavior: selfTradeBehavior ?? "decrementTake",
      clientOrderId: clientOrderId ?? new BN(Date.now()),
      payer: payer ?? marginAccount.address
    })
    return await marginAccount.sendAndConfirm(instructions)
  }

  /** Get instruction to submit an order to Serum
   *
   * @param {{
   *    instructions: TransactionInstruction[]
   *    marginAccount: MarginAccount
   *    orderSide: orderSide
   *    orderType: orderType
   *    orderPrice: number
   *    orderSize: TokenAmount
   *    selfTradeBehavior: selfTradeBehavior
   *    clientOrderId: BN
   *    payer: PublicKey
   *  }}
   */
  async withPlaceOrder({
    instructions,
    marginAccount,
    orderSide,
    orderType,
    orderPrice,
    orderSize,
    selfTradeBehavior,
    clientOrderId,
    payer
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    orderSide: OrderSide
    orderType: OrderType
    orderPrice: number
    orderSize: TokenAmount
    selfTradeBehavior: SelfTradeBehavior
    clientOrderId: BN
    payer: PublicKey
  }): Promise<void> {
    const limitPrice = new BN(
      Math.round(
        (orderPrice * Math.pow(10, this.quoteDecimals) * this.marketConfig.baseLotSize) /
          (Math.pow(10, this.baseDecimals) * this.marketConfig.quoteLotSize)
      )
    )
    const maxCoinQty = orderSize.lamports
    const baseSizeLots = maxCoinQty.toNumber() / this.marketConfig.baseLotSize
    const maxNativePcQtyIncludingFees = new BN(this.marketConfig.quoteLotSize * baseSizeLots).mul(limitPrice)
    const openOrdersAccounts = await this.serum.findOpenOrdersAccountsForOwner(
      marginAccount.provider.connection,
      this.address
    )
    const feeDiscountPubkey = (await this.serum.findBestFeeDiscountKey(marginAccount.provider.connection, this.address))
      .pubkey

    const ix = await this.programs.marginSerum.methods
      .newOrderV3(
        Market.encodeOrderSide(orderSide),
        limitPrice,
        maxCoinQty,
        maxNativePcQtyIncludingFees,
        Market.encodeSelfTradeBehavior(selfTradeBehavior),
        Market.encodeOrderType(orderType),
        clientOrderId,
        65535
      )
      .accounts({
        marginAccount: marginAccount.address,
        market: this.address,
        openOrdersAccount: openOrdersAccounts && openOrdersAccounts[0].publicKey,
        requestQueue: this.marketConfig.requestQueue,
        eventQueue: this.marketConfig.eventQueue,
        bids: this.marketConfig.bids,
        asks: this.marketConfig.asks,
        payer,
        baseVault: this.marketConfig.baseVault,
        quoteVault: this.marketConfig.quoteVault,
        splTokenProgramId: TOKEN_PROGRAM_ID,
        rentSysvarId: SYSVAR_RENT_PUBKEY,
        serumProgramId: this.programs.config.serumProgramId
      })
      .remainingAccounts(feeDiscountPubkey ? [{ pubkey: feeDiscountPubkey, isSigner: false, isWritable: true }] : [])
      .instruction()
    instructions.push(ix)
  }

  async cancelOrder({ marginAccount, order }: { marginAccount: MarginAccount; order: Order }) {
    const instructions: TransactionInstruction[] = []
    await this.withCancelOrder({ instructions, marginAccount, order })
    return await marginAccount.sendAndConfirm(instructions)
  }

  /**
   * Get instruction to cancel an order on Serum
   * @param {{
   *    instructions: TransactionInstruction[]
   *    marginAccount: MarginAccount
   *    order: Order
   *  }}
   */
  async withCancelOrder({
    instructions,
    marginAccount,
    order
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    order: Order
  }) {
    const ix = await this.programs.marginSerum.methods
      .cancelOrderV2(Market.encodeOrderSide(order.side), order.orderId)
      .accounts({
        marginAccount: marginAccount.address,
        market: this.address,
        openOrdersAccount: order.openOrdersAddress,
        marketBids: this.marketConfig.bids,
        marketAsks: this.marketConfig.asks,
        eventQueue: this.marketConfig.eventQueue,
        serumProgramId: this.programs.config.serumProgramId
      })
      .instruction()
    instructions.push(ix)
  }

  async cancelOrderByClientId(marginAccount: MarginAccount, orderId: BN) {
    const instructions: TransactionInstruction[] = []
    await this.withCancelOrderByClientId({ instructions, marginAccount, orderId })
    return await marginAccount.sendAndConfirm(instructions)
  }

  /**
   * Get instruction to cancel an order on Serum by its clientId
   * @param {{
   *    instructions: TransactionInstruction[]
   *    marginAccount: MarginAccount
   *    orderId: BN
   *  }}
   */
  async withCancelOrderByClientId({
    instructions,
    marginAccount,
    orderId
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    orderId: BN
  }) {
    const ix = await this.programs.marginSerum.methods
      .cancelOrderByClientIdV2(orderId)
      .accounts({
        marginAccount: marginAccount.address,
        market: this.address,
        marketBids: this.marketConfig.bids,
        marketAsks: this.marketConfig.asks,
        eventQueue: this.marketConfig.eventQueue,
        serumProgramId: this.programs.config.serumProgramId
      })
      .instruction()
    instructions.push(ix)
  }

  async settleFunds(
    marginAccount: MarginAccount,
    openOrders: OpenOrders,
    baseWallet: PublicKey,
    quoteWallet: PublicKey,
    referrerQuoteWallet: PublicKey | null = null
  ) {
    if (!openOrders.owner.equals(marginAccount.address)) {
      throw new Error("Invalid open orders account")
    }
    const supportsReferralFees = getLayoutVersion(translateAddress(this.programs.config.serumProgramId)) > 1
    if (referrerQuoteWallet && !supportsReferralFees) {
      throw new Error("This program ID does not support referrerQuoteWallet")
    }
    const instructions: TransactionInstruction[] = []
    const signers = await this.withSettleFunds({
      instructions,
      marginAccount,
      openOrders,
      baseWallet,
      quoteWallet,
      referrerQuoteWallet
    })

    return await marginAccount.sendAndConfirm(instructions, signers)
  }

  /**
   * Get instruction to settle funds
   * @param {{
   *    instructions: TransactionInstruction[]
   *    marginAccount: MarginAccount
   *    openOrders: OpenOrders
   *    baseWallet: PublicKey
   *    quoteWallet: PublicKey
   *    referrerQuoteWallet: PublicKey | null
   *  }}
   */
  async withSettleFunds({
    instructions,
    marginAccount,
    openOrders,
    baseWallet,
    quoteWallet,
    referrerQuoteWallet = null
  }: {
    instructions: TransactionInstruction[]
    marginAccount: MarginAccount
    openOrders: OpenOrders
    baseWallet: PublicKey
    quoteWallet: PublicKey
    referrerQuoteWallet: PublicKey | null
  }) {
    const vaultSigner = await PublicKey.createProgramAddress(
      [this.address.toBuffer(), this.serum.decoded.vaultSignerNonce.toArrayLike(Buffer, "le", 8)],
      translateAddress(this.programs.config.serumProgramId)
    )
    const signers: Keypair[] = []
    let wrappedSolAccount: Keypair | null = null
    if (
      (this.baseMint.equals(WRAPPED_SOL_MINT) && baseWallet.equals(openOrders.owner)) ||
      (this.quoteMint.equals(WRAPPED_SOL_MINT) && quoteWallet.equals(openOrders.owner))
    ) {
      wrappedSolAccount = new Keypair()
      instructions.push(
        SystemProgram.createAccount({
          fromPubkey: openOrders.owner,
          newAccountPubkey: wrappedSolAccount.publicKey,
          lamports: await marginAccount.provider.connection.getMinimumBalanceForRentExemption(165),
          space: 165,
          programId: TOKEN_PROGRAM_ID
        })
      )
      instructions.push(
        initializeAccount({
          account: wrappedSolAccount.publicKey,
          mint: WRAPPED_SOL_MINT,
          owner: openOrders.owner
        })
      )
      signers.push(wrappedSolAccount)
    }

    const ix = await this.programs.marginSerum.methods
      .settleFunds()
      .accounts({
        marginAccount: marginAccount.address,
        market: this.address,
        splTokenProgramId: TOKEN_PROGRAM_ID,
        openOrdersAccount: openOrders.address,
        coinVault: this.marketConfig.baseVault,
        pcVault: this.marketConfig.quoteVault,
        coinWallet: baseWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : baseWallet,
        pcWallet: quoteWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : quoteWallet,
        vaultSigner,
        serumProgramId: this.programs.config.serumProgramId
      })
      .remainingAccounts(
        referrerQuoteWallet ? [{ pubkey: referrerQuoteWallet, isSigner: false, isWritable: true }] : []
      )
      .instruction()

    instructions.push(ix)
    if (wrappedSolAccount) {
      instructions.push(
        closeAccount({
          source: wrappedSolAccount.publicKey,
          destination: openOrders.owner,
          owner: openOrders.owner
        })
      )
    }

    return signers
  }

  /**
   * Loads the Orderbook
   */
  async loadOrderbook(provider: AnchorProvider): Promise<Orderbook> {
    const bidsBuffer = (await provider.connection.getAccountInfo(translateAddress(this.marketConfig.bids)))?.data
    const asksBuffer = (await provider.connection.getAccountInfo(translateAddress(this.marketConfig.asks)))?.data
    if (!bidsBuffer || !asksBuffer) {
      throw new Error("Orderbook sides not found")
    }

    const bids = SerumOrderbook.decode(this.serum, bidsBuffer)
    const asks = SerumOrderbook.decode(this.serum, asksBuffer)
    return new Orderbook(this.serum, bids, asks)
  }

  /**
   * Divide two BN's and return a number
   * @param numerator
   * @param denominator
   */
  divideBnToNumber(numerator: BN, denominator: BN): number {
    const quotient = numerator.div(denominator).toNumber()
    const rem = numerator.umod(denominator)
    const gcd = rem.gcd(denominator)
    return quotient + rem.div(gcd).toNumber() / denominator.div(gcd).toNumber()
  }

  /**
   * Price helper functions
   * @param price
   */
  priceLotsToNumber(price: BN) {
    return this.divideBnToNumber(
      price.mul(this.serum.decoded.quoteLotSize).mul(this.baseDecimalMultiplier),
      this.serum.decoded.baseLotSize.mul(this.quoteDecimalMultiplier)
    )
  }
  priceNumberToLots(price: number): BN {
    return new BN(
      Math.round(
        (price * Math.pow(10, this.quoteDecimals) * this.serum.decoded.baseLotSize.toNumber()) /
          (Math.pow(10, this.baseDecimals) * this.serum.decoded.quoteLotSize.toNumber())
      )
    )
  }

  /**
   * Base size helper functions
   * @param size
   */
  baseSizeToNumber(size: BN) {
    return this.divideBnToNumber(size, this.baseDecimalMultiplier)
  }

  baseSizeLotsToNumber(size: BN) {
    return this.divideBnToNumber(size.mul(this.serum.decoded.baseLotSize), this.baseDecimalMultiplier)
  }

  baseSizeNumberToLots(size: number): BN {
    const native = new BN(Math.round(size * Math.pow(10, this.baseDecimals)))
    // rounds down to the nearest lot size
    return native.div(this.serum.decoded.baseLotSize)
  }

  /**
   * Quote size helper functions
   * @param size
   */
  quoteSizeToNumber(size: BN) {
    return this.divideBnToNumber(size, this.quoteDecimalMultiplier)
  }
  quoteSizeLotsToNumber(size: BN) {
    return this.divideBnToNumber(size.mul(this.serum.decoded.quoteLotSize), this.quoteDecimalMultiplier)
  }
  quoteSizeNumberToLots(size: number): BN {
    const native = new BN(Math.round(size * Math.pow(10, this.quoteDecimals)))
    // rounds down to the nearest lot size
    return native.div(this.serum.decoded.quoteLotSize)
  }
}

export class Orderbook {
  market: SerumMarket
  bids: SerumOrderbook
  asks: SerumOrderbook

  /**
   * Creates a Margin Orderbook
   * @param market
   * @param bids
   * @param asks
   */
  constructor(market: SerumMarket, bids: SerumOrderbook, asks: SerumOrderbook) {
    this.market = market
    this.bids = bids
    this.asks = asks
  }

  /**
   * Load an Orderbook for a given market
   * @param market
   * @param bidsBuffer
   * @param asksBuffer
   */
  static load(market: SerumMarket, bidsBuffer: Buffer, asksBuffer: Buffer) {
    const bids = SerumOrderbook.decode(market, bidsBuffer)
    const asks = SerumOrderbook.decode(market, asksBuffer)
    return new Orderbook(market, bids, asks)
  }

  /**
   * Return bids for a given depth
   * @param depth
   */
  getBids(depth = 8) {
    return this.bids.getL2(depth).map(([price, size]) => [price, size])
  }
  /**
   * Return asks for a given depth
   * @param depth
   */
  getAsks(depth = 8) {
    return this.asks.getL2(depth).map(([price, size]) => [price, size])
  }
}
