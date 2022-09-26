"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AccountPositionListLayout = exports.MAX_POSITIONS = exports.AdapterPositionFlags = exports.PositionKind = exports.ErrorCode = void 0;
const buffer_layout_1 = require("@solana/buffer-layout");
const layout_1 = require("../utils/layout");
var ErrorCode;
(function (ErrorCode) {
    ErrorCode[ErrorCode["InvalidPrice"] = 0] = "InvalidPrice";
    ErrorCode[ErrorCode["OutdatedBalance"] = 1] = "OutdatedBalance";
    ErrorCode[ErrorCode["OutdatedPrice"] = 2] = "OutdatedPrice";
})(ErrorCode = exports.ErrorCode || (exports.ErrorCode = {}));
var PositionKind;
(function (PositionKind) {
    /** The position is not worth anything */
    PositionKind[PositionKind["NoValue"] = 0] = "NoValue";
    /** The position contains a balance of available collateral */
    PositionKind[PositionKind["Deposit"] = 1] = "Deposit";
    /** The position contains a balance of tokens that are owed as a part of some debt. */
    PositionKind[PositionKind["Claim"] = 2] = "Claim";
})(PositionKind = exports.PositionKind || (exports.PositionKind = {}));
var AdapterPositionFlags;
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
})(AdapterPositionFlags = exports.AdapterPositionFlags || (exports.AdapterPositionFlags = {}));
const PriceInfoLayout = (0, buffer_layout_1.struct)([
    (0, layout_1.i64Field)("value"),
    (0, layout_1.u64)("timestamp"),
    (0, buffer_layout_1.s32)("exponent"),
    (0, buffer_layout_1.u8)("isValid"),
    (0, buffer_layout_1.blob)(3, "_reserved")
]);
console.assert(PriceInfoLayout.span === 24, "Unexpected PriceInfoLayout span", PriceInfoLayout.span, "expected", 24);
const AccountPositionLayout = (0, buffer_layout_1.struct)([
    (0, layout_1.pubkey)("token"),
    (0, layout_1.pubkey)("address"),
    (0, layout_1.pubkey)("adapter"),
    (0, layout_1.number128Field)("value"),
    (0, layout_1.u64)("balance"),
    (0, layout_1.u64)("balanceTimestamp"),
    PriceInfoLayout.replicate("price"),
    (0, buffer_layout_1.u32)("kind"),
    (0, buffer_layout_1.s16)("exponent"),
    (0, buffer_layout_1.u16)("valueModifier"),
    (0, layout_1.u64)("maxStaleness"),
    (0, buffer_layout_1.u8)("flags"),
    (0, buffer_layout_1.blob)(23, "_reserved")
]);
console.assert(AccountPositionLayout.span === 192, "Unexpected AccountPositionLayout span", AccountPositionLayout.span, "expected", 192);
const AccountPositionKeyLayout = (0, buffer_layout_1.struct)([(0, layout_1.pubkey)("mint"), (0, layout_1.u64)("index")]);
console.assert(AccountPositionKeyLayout.span === 40, "Unexpected AccountPositionKeyLayout span", AccountPositionKeyLayout.span, "expected", 40);
exports.MAX_POSITIONS = 32;
exports.AccountPositionListLayout = (0, buffer_layout_1.struct)([
    (0, layout_1.u64)("length"),
    (0, buffer_layout_1.seq)(AccountPositionKeyLayout, exports.MAX_POSITIONS, "map"),
    (0, buffer_layout_1.seq)(AccountPositionLayout, exports.MAX_POSITIONS, "positions")
]);
console.assert(exports.AccountPositionListLayout.span === 7432, "Unexpected AccountPositionListLayout span", exports.AccountPositionListLayout.span, "expected", 7432);
//# sourceMappingURL=state.js.map