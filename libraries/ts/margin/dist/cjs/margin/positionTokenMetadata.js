"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.PositionTokenMetadata = void 0;
const bn_js_1 = __importDefault(require("bn.js"));
const utils_1 = require("../utils");
const state_1 = require("./state");
class PositionTokenMetadata {
    constructor({ programs, tokenMint }) {
        this.valueModifier = utils_1.Number128.ZERO;
        this.tokenKind = state_1.PositionKind.NoValue;
        this.programs = programs;
        this.tokenMint = tokenMint;
        this.address = PositionTokenMetadata.derive(programs, tokenMint);
    }
    static derive(programs, tokenMint) {
        return (0, utils_1.findDerivedAccount)(programs.config.metadataProgramId, tokenMint);
    }
    static async load(programs, tokenMint) {
        const metadata = new PositionTokenMetadata({ programs, tokenMint: tokenMint });
        await metadata.refresh();
        return metadata;
    }
    async refresh() {
        const info = await this.programs.connection.getAccountInfo(this.address);
        this.decode(info);
    }
    decode(info) {
        if (!info) {
            this.info = undefined;
            this.valueModifier = utils_1.Number128.ZERO;
            return;
        }
        this.info = this.programs.metadata.coder.accounts.decode("positionTokenMetadata", info.data);
        this.valueModifier = utils_1.Number128.fromDecimal(new bn_js_1.default(this.info.valueModifier), -2);
        this.tokenKind = PositionTokenMetadata.decodeTokenKind(this.info.tokenKind);
    }
    static decodeTokenKind(kind) {
        if ("nonCollateral" in kind) {
            return state_1.PositionKind.NoValue;
        }
        else if ("collateral" in kind) {
            return state_1.PositionKind.Deposit;
        }
        else if ("claim" in kind) {
            return state_1.PositionKind.Claim;
        }
        else {
            throw new Error("Unrecognized TokenKind.");
        }
    }
    getLiability(value) {
        return value;
    }
    collateralValue(value) {
        return this.valueModifier.mul(value);
    }
    requiredCollateralValue(value) {
        if (this.valueModifier.eq(utils_1.Number128.ZERO)) {
            // No leverage configured for claim
            return utils_1.Number128.MAX;
        }
        else {
            return value.div(this.valueModifier);
        }
    }
}
exports.PositionTokenMetadata = PositionTokenMetadata;
//# sourceMappingURL=positionTokenMetadata.js.map