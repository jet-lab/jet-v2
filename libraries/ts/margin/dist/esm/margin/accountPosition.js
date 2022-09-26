import assert from "assert";
import BN from "bn.js";
import { getTimestamp } from "..";
import { Number128 } from "../";
import { PositionKind } from "./state";
export class AccountPosition {
    constructor({ info, price }) {
        this.info = info;
        this.token = info.token;
        this.address = info.address;
        this.adapter = info.adapter;
        this.valueRaw = Number128.fromBits(info.value);
        this.balance = info.balance;
        this.balanceTimestamp = info.balanceTimestamp;
        this.priceRaw = {
            value: price?.value ?? info.price.value,
            exponent: price?.exponent ?? info.price.exponent,
            timestamp: price?.timestamp ?? info.price.timestamp,
            isValid: price ? Number(price.isValid) : info.price.isValid
        };
        this.kind = info.kind;
        this.exponent = info.exponent;
        this.valueModifier = Number128.fromDecimal(new BN(info.valueModifier), -2);
        this.maxStaleness = info.maxStaleness;
        this.flags = info.flags.flags;
        this.calculateValue();
    }
    get value() {
        return this.valueRaw.toNumber();
    }
    get price() {
        return Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent);
    }
    calculateValue() {
        this.valueRaw = Number128.fromDecimal(this.balance, this.exponent).mul(Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent));
    }
    collateralValue() {
        assert(this.kind === PositionKind.Deposit);
        return this.valueModifier.mul(this.valueRaw);
    }
    requiredCollateralValue(setupLeverageFraction = Number128.ONE) {
        assert(this.kind === PositionKind.Claim);
        if (this.valueModifier.eq(Number128.ZERO)) {
            console.log(`no leverage configured for claim ${this.token.toBase58()}`);
            return Number128.MAX;
        }
        else {
            return this.valueRaw.div(this.valueModifier).div(setupLeverageFraction);
        }
    }
    setBalance(balance) {
        this.balance = balance;
        this.balanceTimestamp = getTimestamp();
        this.calculateValue();
    }
    setPrice(price) {
        this.priceRaw = price;
        this.calculateValue();
    }
}
//# sourceMappingURL=accountPosition.js.map