import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types";
import { JetMetadata } from "../../types";
/****************************
 * Program Accounts
 ****************************/
export declare type LiquidatorMetadata = TypeDef<AllAccountsMap<JetMetadata>["liquidatorMetadata"], IdlTypes<JetMetadata>>;
export declare type MarginAdapterMetadata = TypeDef<AllAccountsMap<JetMetadata>["marginAdapterMetadata"], IdlTypes<JetMetadata>>;
export declare type PositionTokenMetadataInfo = TypeDef<AllAccountsMap<JetMetadata>["positionTokenMetadata"], IdlTypes<JetMetadata>>;
export declare type TokenMetadata = TypeDef<AllAccountsMap<JetMetadata>["tokenMetadata"], IdlTypes<JetMetadata>>;
export declare type TokenKind = {
    nonCollateral?: Record<string, never>;
    collateral?: Record<string, never>;
    claim?: Record<string, never>;
};
//# sourceMappingURL=state.d.ts.map