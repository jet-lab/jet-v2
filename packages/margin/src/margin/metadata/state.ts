import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"

/****************************
 * Program Accounts
 ****************************/

export type LiquidatorMetadata = TypeDef<AllAccountsMap<JetMetadataIDL>["liquidatorMetadata"], IdlTypes<JetMetadataIDL>>
export type MarginAdapterMetadata = TypeDef<
  AllAccountsMap<JetMetadataIDL>["marginAdapterMetadata"],
  IdlTypes<JetMetadataIDL>
>
export type TokenMetadata = TypeDef<AllAccountsMap<JetMetadataIDL>["tokenMetadata"], IdlTypes<JetMetadataIDL>>

export type TokenKind = {
  nonCollateral?: Record<string, never>
  collateral?: Record<string, never>
  claim?: Record<string, never>
}
