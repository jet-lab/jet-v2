import { Program, BN, Address } from "@project-serum/anchor";
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  ConfirmOptions,
  PublicKey,
  Signer,
  SystemProgram,
} from "@solana/web3.js";
import { BondsUser } from "./bondsUser";
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
  asksSlab: PublicKey;
  bidsSlab: PublicKey;
  underlyingTokenMint: PublicKey;
  underlyingTokenVault: PublicKey;
  bondTicketMint: PublicKey;
  claimsMint: PublicKey;
  oracle: PublicKey;
  seed: number[];
  bump: number[];
  conversionFactor: number;
  reserved: number[];
  duration: BN;
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
    oracle: PublicKey;
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
      asks: info.asksSlab,
      bids: info.bidsSlab,
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
  /**
   * Creates a `BondUser` client class for interaction with the orderbook
   *
   * @param user A signer that will hold authority over the account
   * @param payer A payer for PDA initialization
   * @param opts (optional) confirm options
   * @returns `BondsUser`
   */
  async createBondsUser(args: {
    user: Signer;
    payer: Signer;
    opts?: ConfirmOptions;
  }): Promise<BondsUser> {
    const userAddress = await BondsUser.deriveAddress(
      args.user.publicKey,
      this.address,
      this.program.programId
    );

    const claims = (
      await findDerivedAccount(
        ["user_claims", userAddress],
        this.program.programId
      )
    ).address;

    let tx = await this.program.methods
      .initializeOrderbookUser()
      .accounts({
        ...this.addresses,
        orderbookUserAccount: userAddress,
        claims: claims,
        user: args.user.publicKey,
        payer: args.payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .transaction();

    await this.provider.sendAndConfirm!(tx, [args.user, args.payer], args.opts);

    return await BondsUser.load(this, userAddress, args.user);
  }

  async exchangeTokensForTickets(args: {
    amount: BN;
    user: Signer;
    payer: Signer;
    userTokenVault?: Address;
    userTokenVaultAuthority?: Signer;
    userBondTicketVault?: Address;
    opts?: ConfirmOptions;
  }) {
    const authority = args.userTokenVaultAuthority ?? args.user;
    const tokenVault =
      args.userTokenVault ??
      (await getAssociatedTokenAddress(
        this.addresses.underlyingTokenMint,
        authority.publicKey
      ));
    const ticketVault =
      args.userBondTicketVault ??
      (await getAssociatedTokenAddress(
        this.addresses.bondTicketMint,
        authority.publicKey
      ));
    let tx = await this.program.methods
      .exchangeTokens(args.amount)
      .accounts({
        ...this.addresses,
        userBondTicketVault: new PublicKey(ticketVault),
        userUnderlyingTokenVault: new PublicKey(tokenVault),
        userAuthority: args.user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .transaction();

    await this.provider.sendAndConfirm!(tx, [args.user, args.payer], args.opts);
  }

  async fetchOrderbook(): Promise<Orderbook> {
    return await Orderbook.load(this);
  }
}
