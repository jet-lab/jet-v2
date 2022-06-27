import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { JetMetadata } from "src/types"

/****************************
 * Program Accounts
 ****************************/

export type LiquidatorMetadata = TypeDef<AllAccountsMap<JetMetadata>["liquidatorMetadata"], IdlTypes<JetMetadata>>
export type MarginAdapterMetadata = TypeDef<AllAccountsMap<JetMetadata>["marginAdapterMetadata"], IdlTypes<JetMetadata>>
export type PositionTokenMetadata = TypeDef<AllAccountsMap<JetMetadata>["positionTokenMetadata"], IdlTypes<JetMetadata>>
export type TokenMetadata = TypeDef<AllAccountsMap<JetMetadata>["tokenMetadata"], IdlTypes<JetMetadata>>
