import { BN, Idl } from "@project-serum/anchor"
import { IdlTypeDef } from "@project-serum/anchor/dist/cjs/idl"
import { AccountMap, AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { blob, Layout, s16, s32, seq, struct, u16, u32, u8 } from "@solana/buffer-layout"
import { JetMargin } from ".."
import { i64Field, number128, pubkey, u64 } from "../utils/layout"

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

export type LiquidationData = TypeDef<AllAccountsMap<JetMargin>["liquidation"], IdlTypes<JetMargin>>
export type MarginAccountData = TypeDef<AllAccountsMap<JetMargin>["marginAccount"], IdlTypes<JetMargin>>

/****************************
 * Program Types
 ****************************/

export type AccountPosition = TypeDef<AllTypesMap<JetMargin>["AccountPosition"], IdlTypes<JetMargin>>
export type AccountPositionKey = TypeDef<AllTypesMap<JetMargin>["AccountPositionKey"], IdlTypes<JetMargin>> & {
  index: BN
}
export type AccountPositionList = TypeDef<AllTypesMap<JetMargin>["AccountPositionList"], IdlTypes<JetMargin>> & {
  length: BN
  map: AccountPositionKey[]
  positions: AccountPosition[]
}
export type AdapterResult = TypeDef<AllTypesMap<JetMargin>["AdapterResult"], IdlTypes<JetMargin>>
export type CompactAccountMeta = TypeDef<AllTypesMap<JetMargin>["CompactAccountMeta"], IdlTypes<JetMargin>>
export type ErrorCode = TypeDef<AllTypesMap<JetMargin>["ErrorCode"], IdlTypes<JetMargin>>
export type PositionChange = TypeDef<AllTypesMap<JetMargin>["PositionChange"], IdlTypes<JetMargin>>
export type PositionKind = TypeDef<AllTypesMap<JetMargin>["PositionKind"], IdlTypes<JetMargin>>
export type PriceChangeInfo = TypeDef<AllTypesMap<JetMargin>["PriceChangeInfo"], IdlTypes<JetMargin>>
export type PriceInfo = TypeDef<AllTypesMap<JetMargin>["PriceInfo"], IdlTypes<JetMargin>>

const PriceInfoLayout = struct<PriceInfo>([
  i64Field("value"),
  u64("timestamp"),
  s32("exponent"),
  u8("isValid"),
  blob(3, "_reserved") as any as Layout<number[]>
])
console.assert(PriceInfoLayout.span === 24, "Unexpected PriceInfoLayout span", PriceInfoLayout.span, "expected", 24)

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
  blob(24, "_reserved")
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
