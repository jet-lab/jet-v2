import { Program, BN, Address } from "@project-serum/anchor";
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { calculate_order_amount } from "../wasm-utils/pkg/wasm_utils";
import { Orderbook } from "./orderbook";
import { JetBonds } from "./types";
import { fetchData, findDerivedAccount } from "./utils";

export const OrderSideBorrow = { borrow: {} };
export const OrderSideLend = { lend: {} };
export type OrderSide = typeof OrderSideBorrow | typeof OrderSideLend;

export interface OrderParams {
  maxBondTicketQty: BN;
  maxUnderlyingTokenQty: BN;
  limitPrice: BN;
  matchLimit: BN;
  postOnly: boolean;
  postAllowed: boolean;
  autoStake: boolean;
}

/**
 * The raw struct as found on chain
 */
export interface BondManagerInfo {
  versionTag: BN;
  programAuthority: PublicKey;
  orderbookMarketState: PublicKey;
  eventQueue: PublicKey;
  asks: PublicKey;
  bids: PublicKey;
  underlyingTokenMint: PublicKey;
  underlyingTokenVault: PublicKey;
  bondTicketMint: PublicKey;
  claimsMint: PublicKey;
  depositsMint: PublicKey;
  underlyingOracle: PublicKey;
  ticketOracle: PublicKey;
  seed: number[];
  bump: number[];
  orderbookPaused: boolean;
  ticketsPaused: boolean;
  reserved: number[];
  duration: BN;
  nonce: BN;
}

export interface ClaimTicket {
  owner: PublicKey;
  bondManager: PublicKey;
  maturationTimestamp: BN;
  redeemable: BN;
}

/**
 * Class for loading and interacting with a BondMarket
 */
export class BondMarket {
  readonly addresses: {
    bondManager: PublicKey;
    orderbookMarketState: PublicKey;
    eventQueue: PublicKey;
    asks: PublicKey;
    bids: PublicKey;
    underlyingTokenMint: PublicKey;
    underlyingTokenVault: PublicKey;
    bondTicketMint: PublicKey;
    claimsMint: PublicKey;
    depositsMint: PublicKey;
    underlyingOracle: PublicKey;
    ticketOracle: PublicKey;
  };
  readonly info: BondManagerInfo;
  readonly program: Program<JetBonds>;

  private constructor(
    address: Address,
    program: Program<JetBonds>,
    info: BondManagerInfo
  ) {
    this.addresses = {
      ...info,
      bondManager: new PublicKey(address),
    };
    this.program = program;
    this.info = info;
  }

  get address() {
    return this.addresses.bondManager;
  }

  get provider() {
    return this.program.provider;
  }

  /**
   * Loads the program state from on chain and returns a `BondMarket` client
   * class for interaction with the market
   *
   * @param program The anchor `JetBonds` program
   * @param address The address of the `bondManager` account
   * @returns
   */
  static async load(
    program: Program<JetBonds>,
    address: Address
  ): Promise<BondMarket> {
    let data = await fetchData(program.provider.connection, address);
    let info: BondManagerInfo = program.coder.accounts.decode(
      "BondManager",
      data
    );

    return new BondMarket(address, program, info);
  }

  async exchangeTokensForTicketsIx(args: {
    amount: BN;
    user: Address;
    userTokenVault?: Address;
    userTokenVaultAuthority?: Address;
    userBondTicketVault?: Address;
  }): Promise<TransactionInstruction> {
    let authority = args.userTokenVaultAuthority ?? args.user;
    authority = new PublicKey(authority);

    const tokenVault =
      args.userTokenVault ??
      (await getAssociatedTokenAddress(
        this.addresses.underlyingTokenMint,
        authority
      ));
    const ticketVault =
      args.userBondTicketVault ??
      (await getAssociatedTokenAddress(
        this.addresses.bondTicketMint,
        authority
      ));

    return await this.program.methods
      .exchangeTokens(args.amount)
      .accounts({
        ...this.addresses,
        userBondTicketVault: new PublicKey(ticketVault),
        userUnderlyingTokenVault: new PublicKey(tokenVault),
        userAuthority: args.user,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  /**
   * Creates a `Lend` order instruction. Amount is underlying token lamports. Interest is basis points
   * @param amount Total amount of underlying to lend
   * @param interest Minimum interest, in basis points
   * @param seed BN used to seed a `SplitTicket` intialization. (If auto_stake is enabled)
   * @param payer Payer for PDA initialization. Counted as `vaultAuthority` if not provided
   * @param vaultAuthority Authority over the token vault
   * @param ticketVault Ticket vault to receive matched funds
   * @param tokenVault Token vault containing funds for the order
   * @param matchLimit Maximum number of orders to match with
   * @param postOnly Only succeed if order did not match
   * @param postAllowed Post remaining unfilled as an order on the book
   * @param autoStake Automatically stake any matched bond tickets
   * @param maxBondTicketQty Maximum quantity of bond tickets to order fill
   * @param maxUnderlyingTokenQty Maximum quantity of underlying to lend
   * @returns `TransactionInstruction`
   */
  async lendOrderIx(args: {
    amount: BN;
    interest: BN;
    seed: Uint8Array;
    payer: Address;
    vaultAuthority?: Address;
    ticketVault?: Address;
    tokenVault?: Address;
    matchLimit?: BN;
    postOnly?: boolean;
    postAllowed?: boolean;
    autoStake?: boolean;
    maxBondTicketQty?: BN;
    maxUnderlyingTokenQty?: BN;
  }): Promise<TransactionInstruction> {
    const orderAmount = calculateOrderAmount(args.amount, args.interest);
    let params: OrderParams = {
      maxBondTicketQty: args.maxBondTicketQty ?? orderAmount.base,
      maxUnderlyingTokenQty: args.maxUnderlyingTokenQty ?? orderAmount.quote,
      limitPrice: orderAmount.price,
      matchLimit: args.matchLimit ?? new BN(100),
      postOnly: args.postOnly ?? false,
      postAllowed: args.postAllowed ?? true,
      autoStake: args.autoStake ?? true,
    };
    const authority = args.vaultAuthority ?? args.payer;
    const ticketVault =
      args.ticketVault ??
      (await getAssociatedTokenAddress(
        this.info.bondTicketMint,
        new PublicKey(authority)
      ));
    const tokenVault =
      args.tokenVault ??
      (await getAssociatedTokenAddress(
        this.info.underlyingTokenMint,
        new PublicKey(authority)
      ));

    const splitTicket = await findDerivedAccount(
      ["split_ticket", authority, Buffer.from(args.seed)],
      this.program.programId
    );

    return await this.program.methods
      .lendOrder(params, Buffer.from(args.seed))
      .accounts({
        ...this.addresses,
        user: authority,
        userTicketVault: ticketVault,
        userTokenVault: tokenVault,
        splitTicket: splitTicket,
        payer: args.payer,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  /**
   * Creates a `Borrow` order instruction. Amount is underlying token lamports. Interest is basis points
   * @param amount Total amount of underlying to borrow
   * @param interest MAximum interest, in basis points
   * @param vaultAuthority Authority over the token vault
   * @param ticketVault Ticket vault to pay for matched funds
   * @param tokenVault Token vault containing to receive matched funds
   * @param matchLimit Maximum number of orders to match with
   * @param postOnly Only succeed if order did not match
   * @param postAllowed Post remaining unfilled as an order on the book
   * @param autoStake Automatically stake any matched bond tickets
   * @param maxBondTicketQty Maximum quantity of bond tickets to order fill
   * @param maxUnderlyingTokenQty Maximum quantity of underlying to lend
   * @returns `TransactionInstruction`
   */
  async sellTicketsOrderIx(args: {
    amount: BN;
    interest: BN;
    vaultAuthority: Address;
    ticketVault?: Address;
    tokenVault?: Address;
    limitPrice?: BN;
    matchLimit?: BN;
    postOnly?: boolean;
    postAllowed?: boolean;
    autoStake?: boolean;
    maxBondTicketQty?: BN;
    maxUnderlyingTokenQty?: BN;
  }): Promise<TransactionInstruction> {
    const orderAmount = calculateOrderAmount(args.amount, args.interest);
    let params: OrderParams = {
      maxBondTicketQty: args.maxBondTicketQty ?? orderAmount.base,
      maxUnderlyingTokenQty: args.maxUnderlyingTokenQty ?? orderAmount.quote,
      limitPrice: args.limitPrice ?? orderAmount.price,
      matchLimit: args.matchLimit ?? new BN(100),
      postOnly: args.postOnly ?? false,
      postAllowed: args.postAllowed ?? true,
      autoStake: args.autoStake ?? true,
    };
    const ticketVault =
      args.ticketVault ??
      (await getAssociatedTokenAddress(
        this.info.bondTicketMint,
        new PublicKey(args.vaultAuthority)
      ));
    const tokenVault =
      args.tokenVault ??
      (await getAssociatedTokenAddress(
        this.info.underlyingTokenMint,
        new PublicKey(args.vaultAuthority)
      ));

    return await this.program.methods
      .sellTicketsOrder(params)
      .accounts({
        ...this.addresses,
        user: args.vaultAuthority,
        userTicketVault: ticketVault,
        userTokenVault: tokenVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  async cancelOrderIx(args: {
    orderId: BN;
    side: OrderSide;
    user: Address;
    userVault?: Address;
  }): Promise<TransactionInstruction> {
    const userVault =
      args.userVault ?? args.side === OrderSideBorrow
        ? await getAssociatedTokenAddress(
            this.addresses.underlyingTokenMint,
            new PublicKey(args.user)
          )
        : await getAssociatedTokenAddress(
            this.addresses.bondTicketMint,
            new PublicKey(args.user)
          );
    const marketAccount =
      args.side === OrderSideBorrow
        ? this.addresses.underlyingTokenVault
        : this.addresses.bondTicketMint;

    return await this.program.methods
      .cancelOrder(args.orderId)
      .accounts({
        ...this.addresses,
        user: args.user,
        userVault,
        marketAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  async stakeTicketsIx(args: {
    amount: BN;
    seed: Uint8Array;
    user: Address;
    ticketAccount?: Address;
  }): Promise<TransactionInstruction> {
    const claimTicket = await this.deriveClaimTicketKey(args.user, args.seed);
    const bondTicketTokenAccount =
      args.ticketAccount ??
      (await getAssociatedTokenAddress(
        this.addresses.bondTicketMint,
        new PublicKey(args.user)
      ));
    return await this.program.methods
      .stakeBondTickets({
        amount: args.amount,
        ticketSeed: Buffer.from(args.seed),
      })
      .accounts({
        ...this.addresses,
        claimTicket,
        bondTicketTokenAccount,
        ticketHolder: args.user,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();
  }

  async deriveTicketAddress(user: Address): Promise<PublicKey> {
    return await getAssociatedTokenAddress(
      this.addresses.bondTicketMint,
      new PublicKey(user)
    );
  }

  async deriveClaimTicketKey(
    ticketHolder: Address,
    seed: Uint8Array
  ): Promise<PublicKey> {
    return await findDerivedAccount(
      ["claim_ticket", this.address, new PublicKey(ticketHolder), seed],
      this.program.programId
    );
  }

  async fetchOrderbook(): Promise<Orderbook> {
    return await Orderbook.load(this);
  }
}

export interface OrderAmountJS {
  base: BN;
  quote: BN;
  price: BN;
}

export const calculateOrderAmount = (
  amount: BN,
  interestRate: BN
): OrderAmountJS => {
  const orderAmount = calculate_order_amount(
    BigInt(amount.toString()),
    BigInt(interestRate.toString())
  );

  return {
    base: new BN(orderAmount.base.toString()),
    quote: new BN(orderAmount.quote.toString()),
    price: new BN(orderAmount.price.toString()),
  };
};
