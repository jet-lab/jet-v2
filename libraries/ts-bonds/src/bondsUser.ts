import { Address, BN } from "@project-serum/anchor";
import { ConfirmOptions, PublicKey, Signer } from "@solana/web3.js";
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { fetchData, findDerivedAccount } from "./utils";
import {
  BondMarket,
  OrderParams,
  OrderSideBorrow,
  OrderSideLend,
} from "./bondMarket";
import { calculate_limit_price } from "../wasm-utils/pkg/wasm_utils";

/**  The underlying token */
export const AssetKindToken = { underlyingToken: {} };
/** The bond tickets */
export const AssetKindTicket = { bondTicket: {} };
/** Bond tickets or their underlying token */
export type AssetKind = typeof AssetKindTicket | typeof AssetKindToken;

/** OrderbookUser account as found on-chain */
export interface OrderbookUserInfo {
  user: PublicKey;
  bondManager: PublicKey;
  eventAdapter: PublicKey;
  bondTicketsStored: BN;
  underlyingTokenStored: BN;
  outstandingObligations: BN;
  debt: DebtInfo;
  claims: PublicKey;
  nonce: BN;
}

export interface DebtInfo {
  pending: BN;
  committed: BN;
  pastDue: BN;
}

/**
 * A class for user level interaction with the bonds orderbook.
 *
 * Allows placing orders
 */
export class BondsUser {
  private info: OrderbookUserInfo;
  readonly bondMarket: BondMarket;
  readonly user?: Signer;
  readonly address: PublicKey;

  private constructor(
    info: OrderbookUserInfo,
    market: BondMarket,
    address: PublicKey,
    user?: Signer
  ) {
    this.info = info;
    this.bondMarket = market;
    this.address = address;
    this.user = user;
  }

  get provider() {
    return this.bondMarket.provider;
  }

  /**
   * Information stored on chain
   */
  get storedInfo() {
    return this.info;
  }

  /**
   *
   * @param market The `BondMarket` this user account belongs to
   * @param address The on-chain address of this user account
   * @param user (Optional) a signing type for transactions
   * @returns BondsUser
   */
  static async load(
    market: BondMarket,
    address: PublicKey,
    user?: Signer
  ): Promise<BondsUser> {
    let data = await fetchData(market.provider.connection, address);
    let info: OrderbookUserInfo = market.program.coder.accounts.decode(
      "OrderbookUser",
      data
    );

    return new BondsUser(info, market, address, user);
  }

  static async deriveAddress(
    key: PublicKey,
    market: PublicKey,
    programId: PublicKey
  ): Promise<PublicKey> {
    return (
      await findDerivedAccount(["orderbook_user", market, key], programId)
    ).address;
  }

  /**
   * Deposit the token that is loaned and borrowed into the user account
   * @param amount Lamport amount of token to deposit
   * @param payer Transaction fee payer
   * @param tokensOrTickets Which `AssetKind` to deposit
   * @param userTokenVault Token wallet containing tokens to deposit
   * @param userTokenVaultAuthority Authority over the token wallet
   * @param opts Confirmation options
   */
  async deposit(args: {
    amount: BN;
    payer: Signer;
    tokensOrTickets: AssetKind;
    userTokenVault?: Address;
    userTokenVaultAuthority?: Signer;
    opts?: ConfirmOptions;
  }) {
    const authority = args.userTokenVaultAuthority ?? this.user!;
    const mint =
      args.tokensOrTickets == AssetKindTicket
        ? this.bondMarket.addresses.bondTicketMint
        : this.bondMarket.addresses.underlyingTokenMint;
    const vault =
      args.userTokenVault ??
      (await getAssociatedTokenAddress(mint, authority.publicKey));
    let tx = await this.bondMarket.program.methods
      .deposit(args.amount, args.tokensOrTickets)
      .accounts({
        ...this.bondMarket.addresses,
        orderbookUserAccount: this.address,
        userTokenVault: new PublicKey(vault),
        userTokenVaultAuthority: authority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .transaction();
    await this.provider.sendAndConfirm!(tx, [args.payer, authority], args.opts);
  }

  /**
   * Place a `Lend` order to the orderbook
   * @param amount Total token to lend. In lamports
   * @param interest Interest desired in basis points. Used to derive order price
   * @param matchLimit Maximum number of orders to match against in the book
   * @param postOnly Transaction fails if order crosses the spread
   * @param postAllowed Should the order be posted to the book
   * @param autoStake Should the tickets be automatically staked to the program
   * @param user The signer with account control
   * @param maxBondTicketQty (optional) Maximum amount of bond tickets to order fill
   */
  async lend(args: {
    amount: BN;
    interest: BN;
    matchLimit?: BN;
    postOnly?: boolean;
    postAllowed?: boolean;
    autoStake?: boolean;
    user?: Signer;
    maxBondTicketQty?: BN;
    opts?: ConfirmOptions;
  }) {
    const [base, limitPrice] = calculateBasePrice(args.amount, args.interest);

    let params: OrderParams = {
      maxBondTicketQty: args.maxBondTicketQty ?? base,
      maxUnderlyingTokenQty: args.amount,
      limitPrice,
      matchLimit: args.matchLimit ?? new BN(100),
      postOnly: args.postOnly ?? false,
      postAllowed: args.postAllowed ?? true,
      autoStake: args.autoStake ?? true,
    };
    const authority = args.user ?? this.user!;
    let tx = await this.bondMarket.program.methods
      .placeOrder(OrderSideLend, params)
      .accounts({
        ...this.bondMarket.addresses,
        orderbookUserAccount: this.address,
        user: authority.publicKey,
      })
      .transaction();

    await this.provider.sendAndConfirm!(tx, [authority], args.opts);
  }

  /**
   * Place a `Borrow` order to the orderbook
   * @param amount Total amount of tokens to borrow. In lamports
   * @param interest Interest desired in basis points. Used to derive order pricing on the book
   * @param matchLimit Maximum number of orders to match against in the book
   * @param postOnly Transaction fails if order crosses the spread
   * @param postAllowed Should the order be posted to the book
   * @param autoStake Should the tickets be automatically staked to the program
   * @param user The signer with account control
   * @param maxBondTicketQty (optional) Maximum amount of bond tickets to order fill
   * @param opts confirmation options
   */
  async borrow(args: {
    amount: BN;
    interest: BN;
    matchLimit?: BN;
    postOnly?: boolean;
    postAllowed?: boolean;
    autoStake?: boolean;
    user?: Signer;
    maxBondTicketQty?: BN;
    opts?: ConfirmOptions;
  }) {
    const [base, limitPrice] = calculateBasePrice(args.amount, args.interest);

    let params: OrderParams = {
      maxBondTicketQty: args.maxBondTicketQty ?? base,
      maxUnderlyingTokenQty: args.amount,
      limitPrice,
      matchLimit: args.matchLimit ?? new BN(100),
      postOnly: args.postOnly ?? false,
      postAllowed: args.postAllowed ?? true,
      autoStake: args.autoStake ?? true,
    };
    const authority = args.user ?? this.user!;
    let tx = await this.bondMarket.program.methods
      .placeOrder(OrderSideBorrow, params)
      .accounts({
        ...this.bondMarket.addresses,
        orderbookUserAccount: this.address,
        user: authority.publicKey,
      })
      .transaction();

    await this.provider.sendAndConfirm!(tx, [authority], args.opts);
  }

  async withdraw(args: {
    amount: BN;
    payer: Signer;
    tokensOrTickets: AssetKind;
    user?: Signer;
    userTokenVault?: Address;
    opts?: ConfirmOptions;
  }) {
    const authority = args.user ?? this.user!;
    const mint =
      args.tokensOrTickets == AssetKindToken
        ? this.bondMarket.addresses.underlyingTokenMint
        : this.bondMarket.addresses.bondTicketMint;

    const vault =
      args.userTokenVault ??
      (await getAssociatedTokenAddress(mint, authority.publicKey));

    let tx = await this.bondMarket.program.methods
      .withdraw(args.amount, args.tokensOrTickets)
      .accounts({
        ...this.bondMarket.addresses,
        orderbookUserAccount: this.address,
        user: authority.publicKey,
        userTokenVault: new PublicKey(vault),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .transaction();
    await this.provider.sendAndConfirm!(tx, [args.payer, authority], args.opts);
  }

  async refresh() {
    const data = await fetchData(this.provider.connection, this.address);
    const info: OrderbookUserInfo =
      this.bondMarket.program.coder.accounts.decode("OrderbookUser", data);

    this.info = info;
  }
}

const calculateBasePrice = (amount: BN, interest: BN): [BN, BN] => {
  const bpsUnit = new BN(10_000);
  const base = amount.mul(interest.add(bpsUnit)).div(bpsUnit);
  const limitPrice = calculate_limit_price(
    BigInt(base.toNumber()),
    BigInt(amount.toNumber())
  );

  return [base, new BN(limitPrice.toString())];
};
