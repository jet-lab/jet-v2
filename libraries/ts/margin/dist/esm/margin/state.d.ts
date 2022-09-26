import { BN, Idl } from "@project-serum/anchor";
import { IdlTypeDef } from "@project-serum/anchor/dist/cjs/idl";
import { AccountMap, AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types";
import { JetMargin } from "..";
/****************************
 * Anchor program type definitions.
 * Anchor 0.24.2 exports `AllAccountsMap` and `AllInstructionsMap`.
 * Here we export `AllTypesMap` to generate interfaces for types in `JetMargin` IDL.
 ****************************/
declare type AllTypes<IDL extends Idl> = IDL["types"] extends undefined ? IdlTypeDef : NonNullable<IDL["types"]>[number];
declare type AllTypesMap<IDL extends Idl> = AccountMap<AllTypes<IDL>>;
/****************************
 * Program Accounts
 ****************************/
export declare type LiquidationData = TypeDef<AllAccountsMap<JetMargin>["liquidation"], IdlTypes<JetMargin>>;
export declare type MarginAccountData = TypeDef<AllAccountsMap<JetMargin>["marginAccount"], IdlTypes<JetMargin>>;
/****************************
 * Program Types
 ****************************/
export declare type AccountPositionInfo = TypeDef<AllTypesMap<JetMargin>["AccountPosition"], IdlTypes<JetMargin>>;
export declare type AccountPositionKey = TypeDef<AllTypesMap<JetMargin>["AccountPositionKey"], IdlTypes<JetMargin>> & {
    index: BN;
};
export declare type AccountPositionList = TypeDef<AllTypesMap<JetMargin>["AccountPositionList"], IdlTypes<JetMargin>> & {
    length: BN;
    map: AccountPositionKey[];
    positions: AccountPositionInfo[];
};
export declare type PositionKindInfo = AccountPositionInfo["kind"];
export declare type AdapterResult = TypeDef<AllTypesMap<JetMargin>["AdapterResult"], IdlTypes<JetMargin>>;
export declare type PositionChange = TypeDef<AllTypesMap<JetMargin>["PositionChange"], IdlTypes<JetMargin>>;
export declare type PriceInfoData = TypeDef<AllTypesMap<JetMargin>["PriceInfo"], IdlTypes<JetMargin>>;
export declare enum ErrorCode {
    InvalidPrice = 0,
    OutdatedBalance = 1,
    OutdatedPrice = 2
}
export declare enum PositionKind {
    /** The position is not worth anything */
    NoValue = 0,
    /** The position contains a balance of available collateral */
    Deposit = 1,
    /** The position contains a balance of tokens that are owed as a part of some debt. */
    Claim = 2
}
export declare enum AdapterPositionFlags {
    /**
     * The position may never be removed by the user, even if the balance remains at zero,
     * until the adapter explicitly unsets this flag.
     */
    Required = 1,
    /**
     * Only applies to claims.
     * For any other position, this can be set, but it will be ignored.
     * The claim must be repaid immediately.
     * The account will be considered unhealty if there is any balance on this position.
     */
    PastDue = 2
}
export declare const MAX_POSITIONS = 32;
export declare const AccountPositionListLayout: import("@solana/buffer-layout").Structure<AccountPositionList>;
export {};
//# sourceMappingURL=state.d.ts.map