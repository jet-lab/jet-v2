import { BN } from "@project-serum/anchor";
import {
  AllAccountsMap,
  IdlTypes,
  TypeDef,
} from "@project-serum/anchor/dist/cjs/program/namespace/types";
import {
  blob,
  s16,
  s32,
  seq,
  struct,
  u16,
  u32,
  u8,
} from "@solana/buffer-layout";
import { PublicKey } from "@solana/web3.js";
import { JetMargin } from "..";
import { i64Field, number128, pubkey, u64 } from "../utils/layout";

export type MarginAccountData = TypeDef<
  AllAccountsMap<JetMargin>["marginAccount"],
  IdlTypes<JetMargin>
>;

export interface PriceInfo {
  /** The current price. i64 */
  value: BN;

  /** The timestamp the price was valid at. u64 */
  timestamp: BN;

  /** The exponent for the price value */
  exponent: number;

  /** Flag indicating if the price is valid for the position  */
  isValid: number;

  _reserved: Uint8Array;
}

const PriceInfoLayout = struct<PriceInfo>([
  i64Field("value"),
  u64("timestamp"),
  s32("exponent"),
  u8("isValid"),
  blob(3, "_reserved"),
]);
console.assert(
  PriceInfoLayout.span === 24,
  "Unexpected PriceInfoLayout span",
  PriceInfoLayout.span,
  "expected",
  24
);

export interface AccountPosition {
  /// The address of the token/mint of the asset */
  token: PublicKey;

  /// The address of the account holding the tokens. */
  address: PublicKey;

  /// The address of the adapter managing the asset */
  adapter: PublicKey;

  /// The current value of this position */
  value: BN;

  /// The amount of tokens in the account */
  balance: BN;

  /// The timestamp of the last balance update */
  balanceTimestamp: BN;

  /// The current price/value of each token */
  price: PriceInfo;

  /// The kind of balance this position contains */
  kind: number;

  /// The exponent for the token value */
  exponent: number;

  /// A weight on the value of this asset when counting collateral */
  collateralWeight: number;

  /// The max staleness for the account balance (seconds) */
  collateralMaxStaleness: BN;

  _reserved: Uint8Array;
}

const AccountPositionLayout = struct<AccountPosition>([
  pubkey("token"),
  pubkey("address"),
  pubkey("adapter"),
  number128("value"),
  u64("balance"),
  u64("balanceTimestamp"),
  PriceInfoLayout.replicate("price"),
  u32("kind"),
  s16("exponent"),
  u16("collateralWeight"),
  u64("collateralMaxStaleness"),
  blob(24, "_reserved"),
]);
console.assert(
  AccountPositionLayout.span === 192,
  "Unexpected AccountPositionLayout span",
  AccountPositionLayout.span,
  "expected",
  192
);

export interface AccountPositionKey {
  /* The address of the mint for the position token */
  mint: PublicKey;

  /* The array index where the data for this position is located */
  index: BN;
}

const AccountPositionKeyLayout = struct<AccountPositionKey>([
  pubkey("mint"),
  u64("index"),
]);
console.assert(
  AccountPositionKeyLayout.span === 40,
  "Unexpected AccountPositionKeyLayout span",
  AccountPositionKeyLayout.span,
  "expected",
  40
);

export interface AccountPositionList {
  length: BN;
  map: AccountPositionKey[];
  positions: AccountPosition[];
}

export const MAX_POSITIONS = 32;

export const AccountPositionListLayout = struct<AccountPositionList>([
  u64("length"),
  seq(AccountPositionKeyLayout, MAX_POSITIONS, "map"),
  seq(AccountPositionLayout, MAX_POSITIONS, "positions"),
]);
console.assert(
  AccountPositionListLayout.span === 7432,
  "Unexpected AccountPositionListLayout span",
  AccountPositionListLayout.span,
  "expected",
  7432
);
