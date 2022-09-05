import { Address, BN } from "@project-serum/anchor";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { fetchData, findDerivedAccount } from "./utils";
import { BondMarket, ClaimTicket } from "./bondMarket";
import { calculate_price } from "wasm-utils";

/**  The underlying token */
export const AssetKindToken = { underlyingToken: {} };
/** The bond tickets */
export const AssetKindTicket = { bondTicket: {} };
/** Bond tickets or their underlying token */
export type AssetKind = typeof AssetKindTicket | typeof AssetKindToken;

/** MarginUser account as found on-chain */
export interface MarginUserInfo {
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
  readonly bondMarket: BondMarket;
  readonly user: Address;
  readonly addresses: {
    ticketAccount: Address;
  };

  private constructor(
    market: BondMarket,
    user: Address,
    ticketAccount: Address
  ) {
    this.bondMarket = market;
    this.user = user;
    this.addresses = {
      ticketAccount,
    };
  }

  get provider() {
    return this.bondMarket.provider;
  }

  /**
   *
   * @param market The `BondMarket` this user account belongs to
   * @param user the pubkey of the signer that interacts with the market
   * @returns BondsUser
   */
  static async load(market: BondMarket, user: Address): Promise<BondsUser> {
    const ticketAccount = await market.deriveTicketAddress(user);
    return new BondsUser(market, user, ticketAccount);
  }

  async exchangeTokensForTicketsIx(
    amount: BN
  ): Promise<TransactionInstruction> {
    return await this.bondMarket.exchangeTokensForTicketsIx({
      amount,
      user: this.user,
      userBondTicketVault: this.addresses.ticketAccount,
    });
  }

  async loadClaimTicket(seed: BN): Promise<ClaimTicket> {
    const key = await this.bondMarket.deriveClaimTicketKey(this.user, seed);
    const data = (await this.bondMarket.provider.connection.getAccountInfo(
      key
    ))!.data;

    return await this.bondMarket.program.coder.accounts.decode(
      "ClaimTicket",
      data
    );
  }
}

const calculateBasePrice = (amount: BN, interest: BN): [BN, BN] => {
  const bpsUnit = new BN(10_000);
  const base = amount.mul(interest.add(bpsUnit)).div(bpsUnit);
  const limitPrice = calculate_price(
    BigInt(base.toNumber()),
    BigInt(amount.toNumber())
  );

  return [base, new BN(limitPrice.toString())];
};
