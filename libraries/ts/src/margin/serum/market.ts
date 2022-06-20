import assert from "assert"
import { blob, struct, u8 } from "buffer-layout"
import { AnchorProvider, BN, InstructionNamespace } from "@project-serum/anchor"
import {
  decodeEventQueue,
  decodeRequestQueue,
  getFeeTier,
  getLayoutVersion,
  Market as SerumMarket,
  OpenOrders,
  supportsSrmFeeDiscounts
} from "@project-serum/serum"
import { getMintDecimals, MarketOptions, Order, ORDERBOOK_LAYOUT, OrderParams } from "@project-serum/serum/lib/market"
import { Slab } from "@project-serum/serum/lib/slab"
import {
  closeAccount,
  initializeAccount,
  MSRM_DECIMALS,
  MSRM_MINT,
  SRM_DECIMALS,
  SRM_MINT,
  TOKEN_PROGRAM_ID,
  WRAPPED_SOL_MINT
} from "@project-serum/serum/lib/token-instructions"
import {
  Account,
  AccountInfo,
  Commitment,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  TransactionSignature
} from "@solana/web3.js"

import { buildInstructions } from "../../utils/idlBuilder"
import { IDL as JetControlIDL, JetControl } from "../../types/jetControl"
import { IDL as JetMarginIDL, JetMargin } from "../../types/jetMargin"
import { IDL as JetMarginSerumIDL, JetMarginSerum } from "../../types/jetMarginSerum"
import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { ZERO_BN } from "../../token"

export class Market {
  private _programs: MarginPrograms
  _decoded: any
  private _baseSplTokenDecimals: number
  private _quoteSplTokenDecimals: number
  private _skipPreflight: boolean
  private _commitment: Commitment
  private _serumProgramId: PublicKey
  private _openOrdersAccountsCache: {
    [publickKey: string]: { accounts: OpenOrders[]; ts: number }
  }
  private _layoutOverride?: any

  private _feeDiscountKeysCache: {
    [publicKey: string]: {
      accounts: Array<{
        balance: number
        mint: PublicKey
        pubkey: PublicKey
        feeTier: number
      }>
      ts: number
    }
  }

  private controlProgramId: PublicKey
  private controlInstructions: InstructionNamespace<JetControl>

  private marginInstructions: InstructionNamespace<JetMargin>

  private marginSerumProgramId: PublicKey
  private marginSerumInstructions: InstructionNamespace<JetMarginSerum>
  private marginSerumAdapterMetadata: PublicKey

  private metadataProgramId: PublicKey

  constructor(
    programs: MarginPrograms,
    decoded,
    baseMintDecimals: number,
    quoteMintDecimals: number,
    options: MarketOptions = {},
    serumProgramId: PublicKey,
    controlProgramId: PublicKey,
    marginProgramId: PublicKey,
    marginSerumProgramId: PublicKey,
    metadataProgramId: PublicKey,
    marginSerumAdapterMetadata: PublicKey,
    layoutOverride?: any
  ) {
    const { skipPreflight = false, commitment = "recent" } = options
    if (!decoded.accountFlags.initialized || !decoded.accountFlags.market) {
      throw new Error("Invalid market state")
    }
    this._programs = programs
    this._decoded = decoded
    this._baseSplTokenDecimals = baseMintDecimals
    this._quoteSplTokenDecimals = quoteMintDecimals
    this._skipPreflight = skipPreflight
    this._commitment = commitment
    this._serumProgramId = serumProgramId
    this._openOrdersAccountsCache = {}
    this._feeDiscountKeysCache = {}
    this._layoutOverride = layoutOverride

    assert(controlProgramId)
    this.controlProgramId = controlProgramId
    this.controlInstructions = buildInstructions(JetControlIDL, controlProgramId) as InstructionNamespace<JetControl>

    assert(marginProgramId)
    //this.marginProgramId = marginProgramId;
    this.marginInstructions = buildInstructions(JetMarginIDL, marginProgramId) as InstructionNamespace<JetMargin>

    assert(marginSerumProgramId)
    this.marginSerumProgramId = marginSerumProgramId
    this.marginSerumInstructions = buildInstructions(
      JetMarginSerumIDL,
      marginSerumProgramId
    ) as InstructionNamespace<JetMarginSerum>
    this.marginSerumAdapterMetadata = marginSerumAdapterMetadata

    assert(metadataProgramId)
    this.metadataProgramId = metadataProgramId
  }

  static getLayout(programId: PublicKey) {
    return SerumMarket.getLayout(programId)
  }

  static async findAccountsByMints(
    connection: Connection,
    baseMintAddress: PublicKey,
    quoteMintAddress: PublicKey,
    programId: PublicKey
  ) {
    return SerumMarket.findAccountsByMints(connection, baseMintAddress, quoteMintAddress, programId)
  }

  static async load(
    programs: MarginPrograms,
    address: PublicKey,
    options: MarketOptions = {},
    serumProgramId: PublicKey,
    controlProgramId: PublicKey,
    marginProgramId: PublicKey,
    marginSerumProgramId: PublicKey,
    metadataProgramId: PublicKey,
    layoutOverride?: any
  ) {
    const { owner, data } = throwIfNull(await programs.connection.getAccountInfo(address), "Market not found")
    if (!owner.equals(serumProgramId)) {
      throw new Error("Address not owned by program: " + owner.toBase58())
    }
    const decoded = (layoutOverride ?? this.getLayout(serumProgramId)).decode(data)
    if (!decoded.accountFlags.initialized || !decoded.accountFlags.market || !decoded.ownAddress.equals(address)) {
      throw new Error("Invalid market")
    }
    const [baseMintDecimals, quoteMintDecimals] = await Promise.all([
      getMintDecimals(programs.connection, decoded.baseMint),
      getMintDecimals(programs.connection, decoded.quoteMint)
    ])
    return new Market(
      programs,
      decoded,
      baseMintDecimals,
      quoteMintDecimals,
      options,
      serumProgramId,
      controlProgramId,
      marginProgramId,
      marginSerumProgramId,
      metadataProgramId,
      (await PublicKey.findProgramAddress([marginSerumProgramId.toBuffer()], metadataProgramId))[0],
      layoutOverride
    )
  }

  get address(): PublicKey {
    return this._decoded.ownAddress
  }

  get publicKey(): PublicKey {
    return this.address
  }

  get baseMintAddress(): PublicKey {
    return this._decoded.baseMint
  }

  get quoteMintAddress(): PublicKey {
    return this._decoded.quoteMint
  }

  get bidsAddress(): PublicKey {
    return this._decoded.bids
  }

  get asksAddress(): PublicKey {
    return this._decoded.asks
  }

  get decoded(): any {
    return this._decoded
  }

  async loadBids(connection: Connection): Promise<Orderbook> {
    const { data } = throwIfNull(await connection.getAccountInfo(this._decoded.bids))
    const { accountFlags, slab } = Orderbook.LAYOUT.decode(data)
    return new Orderbook(
      new SerumMarket(
        this._decoded,
        this._baseSplTokenDecimals,
        this._quoteSplTokenDecimals,
        { skipPreflight: this._skipPreflight, commitment: this._commitment },
        this._serumProgramId,
        this._layoutOverride
      ),
      accountFlags,
      slab
    )
  }

  async loadAsks(connection: Connection): Promise<Orderbook> {
    const { data } = throwIfNull(await connection.getAccountInfo(this._decoded.asks))
    const { accountFlags, slab } = Orderbook.LAYOUT.decode(data)
    return new Orderbook(
      new SerumMarket(
        this._decoded,
        this._baseSplTokenDecimals,
        this._quoteSplTokenDecimals,
        { skipPreflight: this._skipPreflight, commitment: this._commitment },
        this._serumProgramId,
        this._layoutOverride
      ),
      accountFlags,
      slab
    )
  }

  async loadOrdersForOwner(
    connection: Connection,
    marginAccount: MarginAccount,
    cacheDurationMs = 0
  ): Promise<Order[]> {
    const [bids, asks, openOrdersAccounts] = await Promise.all([
      this.loadBids(connection),
      this.loadAsks(connection),
      this.findOpenOrdersAccountsForOwner(connection, marginAccount, cacheDurationMs)
    ])
    return this.filterForOpenOrders(bids, asks, openOrdersAccounts)
  }

  filterForOpenOrders(bids: Orderbook, asks: Orderbook, openOrdersAccounts: OpenOrders[]): Order[] {
    return [...bids, ...asks].filter(order =>
      openOrdersAccounts.some(openOrders => order.openOrdersAddress.equals(openOrders.address))
    )
  }

  async findBaseTokenAccountsForOwner(
    connection: Connection,
    marginAccount: MarginAccount,
    includeUnwrappedSol = false
  ): Promise<Array<{ pubkey: PublicKey; account: AccountInfo<Buffer> }>> {
    if (this.baseMintAddress.equals(WRAPPED_SOL_MINT) && includeUnwrappedSol) {
      const [wrapped, unwrapped] = await Promise.all([
        this.findBaseTokenAccountsForOwner(connection, marginAccount, false),
        connection.getAccountInfo(marginAccount.address)
      ])
      if (unwrapped !== null) {
        return [{ pubkey: marginAccount.address, account: unwrapped }, ...wrapped]
      }
      return wrapped
    }
    return await this.getTokenAccountsByOwnerForMint(connection, marginAccount, this.baseMintAddress)
  }

  async getTokenAccountsByOwnerForMint(
    connection: Connection,
    marginAccount: MarginAccount,
    mintAddress: PublicKey
  ): Promise<Array<{ pubkey: PublicKey; account: AccountInfo<Buffer> }>> {
    return (
      await connection.getTokenAccountsByOwner(marginAccount.address, {
        mint: mintAddress
      })
    ).value
  }

  async findQuoteTokenAccountsForOwner(
    connection: Connection,
    marginAccount: MarginAccount,
    includeUnwrappedSol = false
  ): Promise<{ pubkey: PublicKey; account: AccountInfo<Buffer> }[]> {
    if (this.quoteMintAddress.equals(WRAPPED_SOL_MINT) && includeUnwrappedSol) {
      const [wrapped, unwrapped] = await Promise.all([
        this.findQuoteTokenAccountsForOwner(connection, marginAccount, false),
        connection.getAccountInfo(marginAccount.address)
      ])
      if (unwrapped !== null) {
        return [{ pubkey: marginAccount.address, account: unwrapped }, ...wrapped]
      }
      return wrapped
    }
    return await this.getTokenAccountsByOwnerForMint(connection, marginAccount, this.quoteMintAddress)
  }

  async findOpenOrdersAccountsForOwner(
    connection: Connection,
    marginAccount: MarginAccount,
    cacheDurationMs = 0
  ): Promise<OpenOrders[]> {
    const strOwner = marginAccount.address.toBase58()
    const now = new Date().getTime()
    if (
      strOwner in this._openOrdersAccountsCache &&
      now - this._openOrdersAccountsCache[strOwner].ts < cacheDurationMs
    ) {
      return this._openOrdersAccountsCache[strOwner].accounts
    }
    const openOrdersAccountsForOwner = await OpenOrders.findForMarketAndOwner(
      connection,
      this.address,
      marginAccount.address,
      this._serumProgramId
    )
    this._openOrdersAccountsCache[strOwner] = {
      accounts: openOrdersAccountsForOwner,
      ts: now
    }
    return openOrdersAccountsForOwner
  }

  async placeOrder(
    marginAccount: MarginAccount,
    {
      owner,
      payer,
      side,
      price,
      size,
      orderType = "limit",
      clientId,
      openOrdersAddressKey,
      openOrdersAccount,
      feeDiscountPubkey
    }: OrderParams
  ) {
    const { transaction, signers } = await this.makePlaceOrderTransaction<Account>(
      marginAccount.provider.connection,
      marginAccount,
      {
        owner,
        payer,
        side,
        price,
        size,
        orderType,
        clientId,
        openOrdersAddressKey,
        openOrdersAccount,
        feeDiscountPubkey
      }
    )
    return await marginAccount.provider.sendAndConfirm(transaction, signers)
  }

  getSplTokenBalanceFromAccountInfo(accountInfo: AccountInfo<Buffer>, decimals: number): number {
    return divideBnToNumber(new BN(accountInfo.data.slice(64, 72), 10, "le"), new BN(10).pow(new BN(decimals)))
  }

  get supportsSrmFeeDiscounts() {
    return supportsSrmFeeDiscounts(this._serumProgramId)
  }

  get supportsReferralFees() {
    return getLayoutVersion(this._serumProgramId) > 1
  }

  get usesRequestQueue() {
    return getLayoutVersion(this._serumProgramId) <= 2
  }

  async findFeeDiscountKeys(
    connection: Connection,
    marginAccount: MarginAccount,
    cacheDurationMs = 0
  ): Promise<
    Array<{
      pubkey: PublicKey
      feeTier: number
      balance: number
      mint: PublicKey
    }>
  > {
    let sortedAccounts: Array<{
      balance: number
      mint: PublicKey
      pubkey: PublicKey
      feeTier: number
    }> = []
    const now = new Date().getTime()
    const strOwner = marginAccount.address.toBase58()
    if (strOwner in this._feeDiscountKeysCache && now - this._feeDiscountKeysCache[strOwner].ts < cacheDurationMs) {
      return this._feeDiscountKeysCache[strOwner].accounts
    }

    if (this.supportsSrmFeeDiscounts) {
      // Fee discounts based on (M)SRM holdings supported in newer versions
      const msrmAccounts = (await this.getTokenAccountsByOwnerForMint(connection, marginAccount, MSRM_MINT)).map(
        ({ pubkey, account }) => {
          const balance = this.getSplTokenBalanceFromAccountInfo(account, MSRM_DECIMALS)
          return {
            pubkey,
            mint: MSRM_MINT,
            balance,
            feeTier: getFeeTier(balance, 0)
          }
        }
      )
      const srmAccounts = (await this.getTokenAccountsByOwnerForMint(connection, marginAccount, SRM_MINT)).map(
        ({ pubkey, account }) => {
          const balance = this.getSplTokenBalanceFromAccountInfo(account, SRM_DECIMALS)
          return {
            pubkey,
            mint: SRM_MINT,
            balance,
            feeTier: getFeeTier(0, balance)
          }
        }
      )
      sortedAccounts = msrmAccounts.concat(srmAccounts).sort((a, b) => {
        if (a.feeTier > b.feeTier) {
          return -1
        } else if (a.feeTier < b.feeTier) {
          return 1
        } else {
          if (a.balance > b.balance) {
            return -1
          } else if (a.balance < b.balance) {
            return 1
          } else {
            return 0
          }
        }
      })
    }
    this._feeDiscountKeysCache[strOwner] = {
      accounts: sortedAccounts,
      ts: now
    }
    return sortedAccounts
  }

  async findBestFeeDiscountKey(
    connection: Connection,
    marginAccount: MarginAccount,
    cacheDurationMs = 30000
  ): Promise<{ pubkey: PublicKey | null; feeTier: number }> {
    const accounts = await this.findFeeDiscountKeys(connection, marginAccount, cacheDurationMs)
    if (accounts.length > 0) {
      return {
        pubkey: accounts[0].pubkey,
        feeTier: accounts[0].feeTier
      }
    }
    return {
      pubkey: null,
      feeTier: 0
    }
  }

  async makePlaceOrderTransaction<T extends PublicKey | Account>(
    connection: Connection,
    marginAccount: MarginAccount,
    {
      owner,
      payer,
      side,
      price,
      size,
      orderType = "limit",
      clientId,
      openOrdersAddressKey,
      openOrdersAccount,
      feeDiscountPubkey = undefined,
      selfTradeBehavior = "decrementTake"
    }: OrderParams<T>,
    cacheDurationMs = 0,
    feeDiscountPubkeyCacheDurationMs = 0
  ) {
    const openOrdersAccounts = await this.findOpenOrdersAccountsForOwner(connection, marginAccount, cacheDurationMs)
    const transaction = new Transaction()
    const signers: Account[] = []

    // Fetch an SRM fee discount key if the market supports discounts and it is not supplied
    let useFeeDiscountPubkey: PublicKey | null
    if (feeDiscountPubkey) {
      useFeeDiscountPubkey = feeDiscountPubkey
    } else if (feeDiscountPubkey === undefined && this.supportsSrmFeeDiscounts) {
      useFeeDiscountPubkey = (
        await this.findBestFeeDiscountKey(connection, marginAccount, feeDiscountPubkeyCacheDurationMs)
      ).pubkey
    } else {
      useFeeDiscountPubkey = null
    }

    let openOrdersAddress: PublicKey
    if (openOrdersAccounts.length === 0) {
      let account
      if (openOrdersAccount) {
        account = openOrdersAccount
      } else {
        account = new Account()
      }
      transaction.add(
        await OpenOrders.makeCreateAccountTransaction(
          connection,
          this.address,
          marginAccount.address,
          account.publicKey,
          this._serumProgramId
        )
      )
      openOrdersAddress = account.publicKey
      signers.push(account)
      // refresh the cache of open order accounts on next fetch
      this._openOrdersAccountsCache[marginAccount.address.toBase58()].ts = 0
    } else if (openOrdersAccount) {
      openOrdersAddress = openOrdersAccount.publicKey
    } else if (openOrdersAddressKey) {
      openOrdersAddress = openOrdersAddressKey
    } else {
      openOrdersAddress = openOrdersAccounts[0].address
    }

    let wrappedSolAccount: Account | null = null
    if (payer.equals(marginAccount.address)) {
      if (
        (side === "buy" && this.quoteMintAddress.equals(WRAPPED_SOL_MINT)) ||
        (side === "sell" && this.baseMintAddress.equals(WRAPPED_SOL_MINT))
      ) {
        wrappedSolAccount = new Account()
        let lamports
        if (side === "buy") {
          lamports = Math.round(price * size * 1.01 * LAMPORTS_PER_SOL)
          if (openOrdersAccounts.length > 0) {
            lamports -= openOrdersAccounts[0].quoteTokenFree.toNumber()
          }
        } else {
          lamports = Math.round(size * LAMPORTS_PER_SOL)
          if (openOrdersAccounts.length > 0) {
            lamports -= openOrdersAccounts[0].baseTokenFree.toNumber()
          }
        }
        lamports = Math.max(lamports, 0) + 1e7
        transaction.add(
          SystemProgram.createAccount({
            fromPubkey: marginAccount.owner,
            newAccountPubkey: wrappedSolAccount.publicKey,
            lamports,
            space: 165,
            programId: TOKEN_PROGRAM_ID
          })
        )
        transaction.add(
          initializeAccount({
            account: wrappedSolAccount.publicKey,
            mint: WRAPPED_SOL_MINT,
            owner: marginAccount.address
          })
        )
        signers.push(wrappedSolAccount)
      } else {
        throw new Error("Invalid payer account")
      }
    }

    const placeOrderInstruction = this.makePlaceOrderInstruction(connection, marginAccount, {
      owner: marginAccount.address,
      payer: wrappedSolAccount?.publicKey ?? payer,
      side,
      price,
      size,
      orderType,
      clientId,
      openOrdersAddressKey: openOrdersAddress,
      feeDiscountPubkey: useFeeDiscountPubkey,
      selfTradeBehavior
    })
    transaction.add(placeOrderInstruction)

    if (wrappedSolAccount) {
      transaction.add(
        closeAccount({
          source: wrappedSolAccount.publicKey,
          destination: marginAccount.address,
          owner: marginAccount.address
        })
      )
    }

    return { transaction, signers, payer: marginAccount.owner }
  }

  makePlaceOrderInstruction<T extends PublicKey | Account>(
    connection: Connection,
    marginAccount: MarginAccount,
    params: OrderParams<T>
  ): TransactionInstruction {
    const {
      owner,
      payer,
      side,
      price,
      size,
      orderType = "limit",
      clientId,
      openOrdersAddressKey,
      openOrdersAccount,
      feeDiscountPubkey = null
    } = params
    if (this.baseSizeNumberToLots(size).lte(ZERO_BN)) {
      throw new Error("size too small")
    }
    if (this.priceNumberToLots(price).lte(ZERO_BN)) {
      throw new Error("invalid price")
    }
    if (this.usesRequestQueue) {
      throw new Error("Not supported.")
    } else {
      return this.makeAdapterInvokeInstruction(
        marginAccount.owner,
        marginAccount.address,
        this.marginSerumProgramId,
        this.marginSerumAdapterMetadata,
        this.makeNewOrderV3Instruction(marginAccount, params)
      )
    }
  }

  makeNewOrderV3Instruction<T extends PublicKey | Account>(
    marginAccount: MarginAccount,
    params: OrderParams<T>
  ): TransactionInstruction {
    const {
      owner,
      payer,
      side,
      price,
      size,
      orderType = "limit",
      clientId,
      openOrdersAddressKey,
      openOrdersAccount,
      feeDiscountPubkey,
      selfTradeBehavior = "decrementTake",
      programId
    } = params
    assert(this.supportsSrmFeeDiscounts)
    const limit = 65535
    return this.marginSerumInstructions.newOrderV3(
      this.encodeSide(side),
      this.priceNumberToLots(price), // limitPrice
      this.baseSizeNumberToLots(size), // maxBaseQuantity
      new BN(this._decoded.quoteLotSize.toNumber()).mul(
        this.baseSizeNumberToLots(size).mul(this.priceNumberToLots(price))
      ), // maxQuoteQuantity
      this.encodeSelfTradeBehavior(selfTradeBehavior),
      this.encodeOrderType(orderType),
      clientId!,
      limit,
      {
        accounts: {
          marginAccount: marginAccount.address,
          market: this.address,
          openOrdersAccount: openOrdersAccount ? openOrdersAccount.publicKey : openOrdersAddressKey!,
          requestQueue: this._decoded.requestQueue,
          eventQueue: this._decoded.eventQueue,
          bids: this._decoded.bids,
          asks: this._decoded.asks,
          payer,
          baseVault: this._decoded.baseVault,
          quoteVault: this._decoded.quoteVault,
          splTokenProgramId: TOKEN_PROGRAM_ID,
          rentSysvarId: SYSVAR_RENT_PUBKEY,
          serumProgramId: this._serumProgramId
        },
        remainingAccounts: feeDiscountPubkey ? [{ pubkey: feeDiscountPubkey, isSigner: false, isWritable: true }] : []
      }
    )
  }

  private encodeOrderType(orderType: string) {
    switch (orderType) {
      case "limit":
        return 0
      case "ioc":
        return 1
      case "postOnly":
        return 2
      default:
        throw new Error(`Invalid order type: ${orderType}`)
    }
  }

  private encodeSelfTradeBehavior(selfTradeBehavior: string) {
    switch (selfTradeBehavior) {
      case "decrementTake":
        return 0
      case "cancelProvide":
        return 1
      case "abortTransaction":
        return 2
      default:
        throw new Error(`Invalid self trade behavior: ${selfTradeBehavior}`)
    }
  }

  private encodeSide(side: string) {
    switch (side) {
      case "bid":
      case "buy":
        return 0
      case "ask":
      case "sell":
        return 1
      default:
        throw new Error(`Invalid side: ${side}`)
    }
  }

  async cancelOrderByClientId(marginAccount: MarginAccount, openOrders: PublicKey, clientId: BN) {
    const transaction = await this.makeCancelOrderByClientIdTransaction(marginAccount, openOrders, clientId)
    return await marginAccount.provider.sendAndConfirm(transaction)
  }

  async makeCancelOrderByClientIdTransaction(marginAccount: MarginAccount, openOrders: PublicKey, clientId: BN) {
    const transaction = new Transaction()
    if (this.usesRequestQueue) {
      throw new Error("Not supported.")
    } else {
      transaction.add(
        this.makeAdapterInvokeInstruction(
          marginAccount.owner,
          marginAccount.address,
          this.marginSerumProgramId,
          this.marginSerumAdapterMetadata,
          this.marginSerumInstructions.cancelOrderByClientIdV2(clientId, {
            accounts: {
              marginAccount: marginAccount.address,
              market: this.address,
              openOrdersAccount: openOrders,
              marketBids: this._decoded.bids,
              marketAsks: this._decoded.asks,
              eventQueue: this._decoded.eventQueue,
              serumProgramId: this._serumProgramId
            }
          })
        )
      )
    }
    return transaction
  }

  async cancelOrder(marginAccount: MarginAccount, order: Order) {
    const transaction = await this.makeCancelOrderTransaction(marginAccount, order)
    return await marginAccount.provider.sendAndConfirm(transaction)
  }

  async makeCancelOrderTransaction(marginAccount: MarginAccount, order: Order) {
    const transaction = new Transaction()
    transaction.add(
      this.makeAdapterInvokeInstruction(
        marginAccount.owner,
        marginAccount.address,
        this.marginSerumProgramId,
        this.marginSerumAdapterMetadata,
        this.makeCancelOrderInstruction(marginAccount, order)
      )
    )
    return transaction
  }

  makeCancelOrderInstruction(marginAccount: MarginAccount, order: Order) {
    if (this.usesRequestQueue) {
      throw new Error("Not supported.")
    } else {
      return this.makeAdapterInvokeInstruction(
        marginAccount.owner,
        marginAccount.address,
        this.marginSerumProgramId,
        this.marginSerumAdapterMetadata,
        this.marginSerumInstructions.cancelOrderV2(this.encodeSide(order.side), order.orderId, {
          accounts: {
            marginAccount: marginAccount.address,
            market: this.address,
            openOrdersAccount: order.openOrdersAddress,
            marketBids: this._decoded.bids,
            marketAsks: this._decoded.asks,
            eventQueue: this._decoded.eventQueue,
            serumProgramId: this._serumProgramId
          }
        })
      )
    }
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
    if (referrerQuoteWallet && !this.supportsReferralFees) {
      throw new Error("This program ID does not support referrerQuoteWallet")
    }
    const { transaction, signers } = await this.makeSettleFundsTransaction(
      marginAccount,
      openOrders,
      baseWallet,
      quoteWallet,
      referrerQuoteWallet
    )
    return await marginAccount.provider.sendAndConfirm(transaction, signers)
  }

  async makeSettleFundsTransaction(
    marginAccount: MarginAccount,
    openOrders: OpenOrders,
    baseWallet: PublicKey,
    quoteWallet: PublicKey,
    referrerQuoteWallet: PublicKey | null = null
  ) {
    const vaultSigner = await PublicKey.createProgramAddress(
      [this.address.toBuffer(), this._decoded.vaultSignerNonce.toArrayLike(Buffer, "le", 8)],
      this._serumProgramId
    )

    const transaction = new Transaction()
    const signers: Account[] = []

    let wrappedSolAccount: Account | null = null
    if (
      (this.baseMintAddress.equals(WRAPPED_SOL_MINT) && baseWallet.equals(openOrders.owner)) ||
      (this.quoteMintAddress.equals(WRAPPED_SOL_MINT) && quoteWallet.equals(openOrders.owner))
    ) {
      wrappedSolAccount = new Account()
      transaction.add(
        SystemProgram.createAccount({
          fromPubkey: openOrders.owner,
          newAccountPubkey: wrappedSolAccount.publicKey,
          lamports: await marginAccount.provider.connection.getMinimumBalanceForRentExemption(165),
          space: 165,
          programId: TOKEN_PROGRAM_ID
        })
      )
      transaction.add(
        initializeAccount({
          account: wrappedSolAccount.publicKey,
          mint: WRAPPED_SOL_MINT,
          owner: openOrders.owner
        })
      )
      signers.push(wrappedSolAccount)
    }

    transaction.add(
      this.makeAdapterInvokeInstruction(
        marginAccount.owner,
        marginAccount.address,
        this.marginSerumProgramId,
        this.marginSerumAdapterMetadata,
        this.marginSerumInstructions.settleFunds({
          accounts: {
            marginAccount: marginAccount.address,
            market: this.address,
            splTokenProgramId: TOKEN_PROGRAM_ID,
            openOrdersAccount: openOrders.address,
            coinVault: this._decoded.baseVault,
            pcVault: this._decoded.quoteVault,
            coinWallet:
              baseWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : baseWallet,
            pcWallet:
              quoteWallet.equals(openOrders.owner) && wrappedSolAccount ? wrappedSolAccount.publicKey : quoteWallet,
            vaultSigner,
            serumProgramId: this._serumProgramId
          },
          remainingAccounts: referrerQuoteWallet
            ? [{ pubkey: referrerQuoteWallet, isSigner: false, isWritable: true }]
            : []
        })
      )
    )

    if (wrappedSolAccount) {
      transaction.add(
        closeAccount({
          source: wrappedSolAccount.publicKey,
          destination: openOrders.owner,
          owner: openOrders.owner
        })
      )
    }

    return { transaction, signers, payer: openOrders.owner }
  }

  public makeConsumeEventsInstruction(openOrdersAccounts: Array<PublicKey>, limit: number): TransactionInstruction {
    return this.marginSerumInstructions.consumeEvents(limit, {
      accounts: {
        serumProgramId: this._serumProgramId,
        market: this.address,
        eventQueue: this._decoded.eventQueue,
        coinFeeReceivableAccount: this._decoded.eventQueue,
        pcFeeReceivableAccount: this._decoded.eventQueue
      },
      remainingAccounts: openOrdersAccounts.map(account => ({
        pubkey: account,
        isSigner: false,
        isWritable: true
      }))
    })
  }

  async matchOrders(connection: Connection, feePayer: Account, limit: number) {
    const tx = this.makeMatchOrdersTransaction(limit)
    return await this.sendTransaction(connection, tx, [feePayer])
  }

  makeMatchOrdersTransaction(limit: number): Transaction {
    const tx = new Transaction()
    tx.add(
      this.marginSerumInstructions.matchOrders(limit, {
        accounts: {
          market: this.address,
          requestQueue: this._decoded.requestQueue,
          eventQueue: this._decoded.eventQueue,
          bids: this._decoded.bids,
          asks: this._decoded.asks,
          coinFeeReceivableAccount: this._decoded.baseVault,
          pcFeeReceivableAccount: this._decoded.quoteVault,
          serumProgramId: this._serumProgramId
        }
      })
    )
    return tx
  }

  makeAdapterInvokeInstruction(
    owner: PublicKey,
    marginAccount: PublicKey,
    adapterProgram: PublicKey,
    adapterMetadata: PublicKey,
    adapterInstruction: TransactionInstruction
  ): TransactionInstruction {
    return this.marginInstructions.adapterInvoke(
      adapterInstruction.keys.slice(1).map(accountMeta => {
        return { isSigner: false, isWritable: accountMeta.isWritable }
      }),
      adapterInstruction.data,
      {
        accounts: {
          owner,
          marginAccount,
          adapterProgram,
          adapterMetadata
        },
        remainingAccounts: adapterInstruction.keys.slice(1).map(accountMeta => {
          return { pubkey: accountMeta.pubkey, isSigner: false, isWritable: accountMeta.isWritable }
        })
        //TODO append the existing remaining accounts
      }
    )
  }

  private async sendTransaction(
    connection: Connection,
    transaction: Transaction,
    signers: Array<Account>
  ): Promise<TransactionSignature> {
    const signature = await connection.sendTransaction(transaction, signers, {
      skipPreflight: this._skipPreflight
    })
    const { value } = await connection.confirmTransaction(signature, this._commitment)
    if (value?.err) {
      throw new Error(JSON.stringify(value.err))
    }
    return signature
  }

  async loadRequestQueue(connection: Connection) {
    const { data } = throwIfNull(await connection.getAccountInfo(this._decoded.requestQueue))
    return decodeRequestQueue(data)
  }

  async loadEventQueue(connection: Connection) {
    const { data } = throwIfNull(await connection.getAccountInfo(this._decoded.eventQueue))
    return decodeEventQueue(data)
  }

  async loadFills(connection: Connection, limit = 100) {
    // TODO: once there's a separate source of fills use that instead
    const { data } = throwIfNull(await connection.getAccountInfo(this._decoded.eventQueue))
    const events = decodeEventQueue(data, limit)
    return events
      .filter(event => event.eventFlags.fill && event.nativeQuantityPaid.gtn(0))
      .map(this.parseFillEvent.bind(this))
  }

  parseFillEvent(event) {
    let size, price, side, priceBeforeFees
    if (event.eventFlags.bid) {
      side = "buy"
      priceBeforeFees = event.eventFlags.maker
        ? event.nativeQuantityPaid.add(event.nativeFeeOrRebate)
        : event.nativeQuantityPaid.sub(event.nativeFeeOrRebate)
      price = divideBnToNumber(
        priceBeforeFees.mul(this._baseSplTokenMultiplier),
        this._quoteSplTokenMultiplier.mul(event.nativeQuantityReleased)
      )
      size = divideBnToNumber(event.nativeQuantityReleased, this._baseSplTokenMultiplier)
    } else {
      side = "sell"
      priceBeforeFees = event.eventFlags.maker
        ? event.nativeQuantityReleased.sub(event.nativeFeeOrRebate)
        : event.nativeQuantityReleased.add(event.nativeFeeOrRebate)
      price = divideBnToNumber(
        priceBeforeFees.mul(this._baseSplTokenMultiplier),
        this._quoteSplTokenMultiplier.mul(event.nativeQuantityPaid)
      )
      size = divideBnToNumber(event.nativeQuantityPaid, this._baseSplTokenMultiplier)
    }
    return {
      ...event,
      side,
      price,
      feeCost: this.quoteSplSizeToNumber(event.nativeFeeOrRebate) * (event.eventFlags.maker ? -1 : 1),
      size
    }
  }

  private get _baseSplTokenMultiplier() {
    return new BN(10).pow(new BN(this._baseSplTokenDecimals))
  }

  private get _quoteSplTokenMultiplier() {
    return new BN(10).pow(new BN(this._quoteSplTokenDecimals))
  }

  priceLotsToNumber(price: BN) {
    return divideBnToNumber(
      price.mul(this._decoded.quoteLotSize).mul(this._baseSplTokenMultiplier),
      this._decoded.baseLotSize.mul(this._quoteSplTokenMultiplier)
    )
  }

  priceNumberToLots(price: number): BN {
    return new BN(
      Math.round(
        (price * Math.pow(10, this._quoteSplTokenDecimals) * this._decoded.baseLotSize.toNumber()) /
          (Math.pow(10, this._baseSplTokenDecimals) * this._decoded.quoteLotSize.toNumber())
      )
    )
  }

  baseSplSizeToNumber(size: BN) {
    return divideBnToNumber(size, this._baseSplTokenMultiplier)
  }

  quoteSplSizeToNumber(size: BN) {
    return divideBnToNumber(size, this._quoteSplTokenMultiplier)
  }

  baseSizeLotsToNumber(size: BN) {
    return divideBnToNumber(size.mul(this._decoded.baseLotSize), this._baseSplTokenMultiplier)
  }

  baseSizeNumberToLots(size: number): BN {
    const native = new BN(Math.round(size * Math.pow(10, this._baseSplTokenDecimals)))
    // rounds down to the nearest lot size
    return native.div(this._decoded.baseLotSize)
  }

  quoteSizeLotsToNumber(size: BN) {
    return divideBnToNumber(size.mul(this._decoded.quoteLotSize), this._quoteSplTokenMultiplier)
  }

  quoteSizeNumberToLots(size: number): BN {
    const native = new BN(Math.round(size * Math.pow(10, this._quoteSplTokenDecimals)))
    // rounds down to the nearest lot size
    return native.div(this._decoded.quoteLotSize)
  }

  get minOrderSize() {
    return this.baseSizeLotsToNumber(new BN(1))
  }

  get tickSize() {
    return this.priceLotsToNumber(new BN(1))
  }
}

export class Orderbook {
  market: SerumMarket
  isBids: boolean
  slab: Slab

  constructor(market: SerumMarket, accountFlags, slab: Slab) {
    if (!accountFlags.initialized || !(accountFlags.bids ^ accountFlags.asks)) {
      throw new Error("Invalid orderbook")
    }
    this.market = market
    this.isBids = accountFlags.bids
    this.slab = slab
  }

  static get LAYOUT() {
    return ORDERBOOK_LAYOUT
  }

  static decode(market: SerumMarket, buffer: Buffer) {
    const { accountFlags, slab } = ORDERBOOK_LAYOUT.decode(buffer)
    return new Orderbook(market, accountFlags, slab)
  }

  getL2(depth: number): [number, number, BN, BN][] {
    const descending = this.isBids
    const levels: [BN, BN][] = [] // (price, size)
    for (const { key, quantity } of this.slab.items(descending)) {
      const price = getPriceFromKey(key)
      if (levels.length > 0 && levels[levels.length - 1][0].eq(price)) {
        levels[levels.length - 1][1].iadd(quantity)
      } else if (levels.length === depth) {
        break
      } else {
        levels.push([price, quantity])
      }
    }
    return levels.map(([priceLots, sizeLots]) => [
      this.market.priceLotsToNumber(priceLots),
      this.market.baseSizeLotsToNumber(sizeLots),
      priceLots,
      sizeLots
    ])
  }

  [Symbol.iterator]() {
    return this.items(false)
  }

  *items(descending = false): Generator<Order> {
    for (const { key, ownerSlot, owner, quantity, feeTier, clientOrderId } of this.slab.items(descending)) {
      const price = getPriceFromKey(key)
      yield {
        orderId: key,
        clientId: clientOrderId,
        openOrdersAddress: owner,
        openOrdersSlot: ownerSlot,
        feeTier,
        price: this.market.priceLotsToNumber(price),
        priceLots: price,
        size: this.market.baseSizeLotsToNumber(quantity),
        sizeLots: quantity,
        side: (this.isBids ? "buy" : "sell") as "buy" | "sell"
      }
    }
  }
}

function getPriceFromKey(key) {
  return key.ushrn(64)
}

function divideBnToNumber(numerator: BN, denominator: BN): number {
  const quotient = numerator.div(denominator).toNumber()
  const rem = numerator.umod(denominator)
  const gcd = rem.gcd(denominator)
  return quotient + rem.div(gcd).toNumber() / denominator.div(gcd).toNumber()
}

const MINT_LAYOUT = struct([blob(44), u8("decimals"), blob(37)])

function throwIfNull<T>(value: T | null, message = "account not found"): T {
  if (value === null) {
    throw new Error(message)
  }
  return value
}
