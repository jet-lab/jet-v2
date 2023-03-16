import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { JetMetadata } from "../../types"

/****************************
 * Program Accounts
 ****************************/

export type LiquidatorMetadata = TypeDef<AllAccountsMap<JetMetadata>["liquidatorMetadata"], IdlTypes<JetMetadata>>
export type MarginAdapterMetadata = TypeDef<AllAccountsMap<JetMetadata>["marginAdapterMetadata"], IdlTypes<JetMetadata>>
export type TokenMetadata = TypeDef<AllAccountsMap<JetMetadata>["tokenMetadata"], IdlTypes<JetMetadata>>

export type TokenKind = {
  nonCollateral?: Record<string, never>
  collateral?: Record<string, never>
  claim?: Record<string, never>
}
