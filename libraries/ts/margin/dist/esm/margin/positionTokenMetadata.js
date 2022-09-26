import BN from "bn.js";
import { findDerivedAccount, Number128 } from "../utils";
import { PositionKind } from "./state";
export class PositionTokenMetadata {
    constructor({ programs, tokenMint }) {
        this.valueModifier = Number128.ZERO;
        this.tokenKind = PositionKind.NoValue;
        this.programs = programs;
        this.tokenMint = tokenMint;
        this.address = PositionTokenMetadata.derive(programs, tokenMint);
    }
    static derive(programs, tokenMint) {
        return findDerivedAccount(programs.config.metadataProgramId, tokenMint);
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
            this.valueModifier = Number128.ZERO;
            return;
        }
        this.info = this.programs.metadata.coder.accounts.decode("positionTokenMetadata", info.data);
        this.valueModifier = Number128.fromDecimal(new BN(this.info.valueModifier), -2);
        this.tokenKind = PositionTokenMetadata.decodeTokenKind(this.info.tokenKind);
    }
    static decodeTokenKind(kind) {
        if ("nonCollateral" in kind) {
            return PositionKind.NoValue;
        }
        else if ("collateral" in kind) {
            return PositionKind.Deposit;
        }
        else if ("claim" in kind) {
            return PositionKind.Claim;
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
        if (this.valueModifier.eq(Number128.ZERO)) {
            // No leverage configured for claim
            return Number128.MAX;
        }
        else {
            return value.div(this.valueModifier);
        }
    }
}
//# sourceMappingURL=positionTokenMetadata.js.map