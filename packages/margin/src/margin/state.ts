import { BN, Idl } from "@project-serum/anchor"
import { IdlTypeDef } from "@project-serum/anchor/dist/cjs/idl"
import { AccountMap, AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { blob, Layout, s16, s32, seq, struct, u16, u32, u8 } from "@solana/buffer-layout"
import { i64Field, number128Field, pubkey, u64 } from "../utils"
import { JetMargin } from "../types"

/****************************
 * Anchor program type definitions.
 * Anchor 0.24.2 exports `AllAccountsMap` and `AllInstructionsMap`.
 * Here we export `AllTypesMap` to generate interfaces for types in `JetMargin` IDL.
 ****************************/

type AllTypes<IDL extends Idl> = IDL["types"] extends undefined ? IdlTypeDef : NonNullable<IDL["types"]>[number]
type AllTypesMap<IDL extends Idl> = AccountMap<AllTypes<IDL>>

/****************************
 * Program Accounts
 ****************************/

export type LiquidationData = TypeDef<AllTypesMap<JetMargin>["Liquidation"], IdlTypes<JetMargin>>
export type MarginAccountData = TypeDef<AllAccountsMap<JetMargin>["marginAccount"], IdlTypes<JetMargin>>

/****************************
 * Program Types
 ****************************/

export type AccountPositionInfo = TypeDef<AllTypesMap<JetMargin>["AccountPosition"], IdlTypes<JetMargin>>
export type AccountPositionKey = TypeDef<AllTypesMap<JetMargin>["AccountPositionKey"], IdlTypes<JetMargin>> & {
  index: BN
}
export type AccountPositionList = TypeDef<AllTypesMap<JetMargin>["AccountPositionList"], IdlTypes<JetMargin>> & {
  length: BN
  map: AccountPositionKey[]
  positions: AccountPositionInfo[]
}
export type PositionKindInfo = AccountPositionInfo["kind"]
export type AdapterResult = TypeDef<AllTypesMap<JetMargin>["AdapterResult"], IdlTypes<JetMargin>>
export type PositionChange = TypeDef<AllTypesMap<JetMargin>["PositionChange"], IdlTypes<JetMargin>>
export type PriceInfoData = TypeDef<AllTypesMap<JetMargin>["PriceInfo"], IdlTypes<JetMargin>>

export enum ErrorCode {
  InvalidPrice,
  OutdatedBalance,
  OutdatedPrice
}

export enum PositionKind {
  /** The position is not worth anything */
  NoValue,
  /** The position contains a balance of available collateral */
  Deposit,
  /** The position contains a balance of tokens that are owed as a part of some debt. */
  Claim,
  /** Different type of collateral currently used for fixed term markets */
  AdapterCollateral
}

export enum AdapterPositionFlags {
  /**
   * The position may never be removed by the user, even if the balance remains at zero,
   * until the adapter explicitly unsets this flag.
   */
  Required = 1 << 0,
  /**
   * Only applies to claims.
   * For any other position, this can be set, but it will be ignored.
   * The claim must be repaid immediately.
   * The account will be considered unhealty if there is any balance on this position.
   */
  PastDue = 1 << 1
}

const PriceInfoLayout = struct<PriceInfoData>([
  i64Field("value"),
  u64("timestamp"),
  s32("exponent"),
  u8("isValid"),
  blob(3, "_reserved") as any as Layout<number[]>
])
console.assert(PriceInfoLayout.span === 24, "Unexpected PriceInfoLayout span", PriceInfoLayout.span, "expected", 24)

const AccountPositionLayout = struct<AccountPositionInfo>([
  pubkey("token"),
  pubkey("address"),
  pubkey("adapter"),
  number128Field("value"),
  u64("balance"),
  u64("balanceTimestamp"),
  PriceInfoLayout.replicate("price"),
  u32("kind"),
  s16("exponent"),
  u16("valueModifier"),
  u64("maxStaleness"),
  u8("flags"),
  blob(23, "_reserved") as any as Layout<number[]>
])
console.assert(
  AccountPositionLayout.span === 192,
  "Unexpected AccountPositionLayout span",
  AccountPositionLayout.span,
  "expected",
  192
)

const AccountPositionKeyLayout = struct<AccountPositionKey>([pubkey("mint"), u64("index")])
console.assert(
  AccountPositionKeyLayout.span === 40,
  "Unexpected AccountPositionKeyLayout span",
  AccountPositionKeyLayout.span,
  "expected",
  40
)

export const MAX_POSITIONS = 32

export const AccountPositionListLayout = struct<AccountPositionList>([
  u64("length"),
  seq(AccountPositionKeyLayout, MAX_POSITIONS, "map"),
  seq(AccountPositionLayout, MAX_POSITIONS, "positions")
])
console.assert(
  AccountPositionListLayout.span === 7432,
  "Unexpected AccountPositionListLayout span",
  AccountPositionListLayout.span,
  "expected",
  7432
)
