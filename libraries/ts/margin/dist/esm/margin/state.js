import { blob, s16, s32, seq, struct, u16, u32, u8 } from "@solana/buffer-layout";
import { i64Field, number128Field, pubkey, u64 } from "../utils/layout";
export var ErrorCode;
(function (ErrorCode) {
    ErrorCode[ErrorCode["InvalidPrice"] = 0] = "InvalidPrice";
    ErrorCode[ErrorCode["OutdatedBalance"] = 1] = "OutdatedBalance";
    ErrorCode[ErrorCode["OutdatedPrice"] = 2] = "OutdatedPrice";
})(ErrorCode || (ErrorCode = {}));
export var PositionKind;
(function (PositionKind) {
    /** The position is not worth anything */
    PositionKind[PositionKind["NoValue"] = 0] = "NoValue";
    /** The position contains a balance of available collateral */
    PositionKind[PositionKind["Deposit"] = 1] = "Deposit";
    /** The position contains a balance of tokens that are owed as a part of some debt. */
    PositionKind[PositionKind["Claim"] = 2] = "Claim";
})(PositionKind || (PositionKind = {}));
export var AdapterPositionFlags;
(function (AdapterPositionFlags) {
    /**
     * The position may never be removed by the user, even if the balance remains at zero,
     * until the adapter explicitly unsets this flag.
     */
    AdapterPositionFlags[AdapterPositionFlags["Required"] = 1] = "Required";
    /**
     * Only applies to claims.
     * For any other position, this can be set, but it will be ignored.
     * The claim must be repaid immediately.
     * The account will be considered unhealty if there is any balance on this position.
     */
    AdapterPositionFlags[AdapterPositionFlags["PastDue"] = 2] = "PastDue";
})(AdapterPositionFlags || (AdapterPositionFlags = {}));
const PriceInfoLayout = struct([
    i64Field("value"),
    u64("timestamp"),
    s32("exponent"),
    u8("isValid"),
    blob(3, "_reserved")
]);
console.assert(PriceInfoLayout.span === 24, "Unexpected PriceInfoLayout span", PriceInfoLayout.span, "expected", 24);
const AccountPositionLayout = struct([
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
    blob(23, "_reserved")
]);
console.assert(AccountPositionLayout.span === 192, "Unexpected AccountPositionLayout span", AccountPositionLayout.span, "expected", 192);
const AccountPositionKeyLayout = struct([pubkey("mint"), u64("index")]);
console.assert(AccountPositionKeyLayout.span === 40, "Unexpected AccountPositionKeyLayout span", AccountPositionKeyLayout.span, "expected", 40);
export const MAX_POSITIONS = 32;
export const AccountPositionListLayout = struct([
    u64("length"),
    seq(AccountPositionKeyLayout, MAX_POSITIONS, "map"),
    seq(AccountPositionLayout, MAX_POSITIONS, "positions")
]);
console.assert(AccountPositionListLayout.span === 7432, "Unexpected AccountPositionListLayout span", AccountPositionListLayout.span, "expected", 7432);
//# sourceMappingURL=state.js.map