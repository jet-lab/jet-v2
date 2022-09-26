"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.AccountPosition = void 0;
const assert_1 = __importDefault(require("assert"));
const bn_js_1 = __importDefault(require("bn.js"));
const __1 = require("..");
const __2 = require("../");
const state_1 = require("./state");
class AccountPosition {
    constructor({ info, price }) {
        this.info = info;
        this.token = info.token;
        this.address = info.address;
        this.adapter = info.adapter;
        this.valueRaw = __2.Number128.fromBits(info.value);
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
        this.valueModifier = __2.Number128.fromDecimal(new bn_js_1.default(info.valueModifier), -2);
        this.maxStaleness = info.maxStaleness;
        this.flags = info.flags.flags;
        this.calculateValue();
    }
    get value() {
        return this.valueRaw.toNumber();
    }
    get price() {
        return __2.Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent);
    }
    calculateValue() {
        this.valueRaw = __2.Number128.fromDecimal(this.balance, this.exponent).mul(__2.Number128.fromDecimal(this.priceRaw.value, this.priceRaw.exponent));
    }
    collateralValue() {
        (0, assert_1.default)(this.kind === state_1.PositionKind.Deposit);
        return this.valueModifier.mul(this.valueRaw);
    }
    requiredCollateralValue(setupLeverageFraction = __2.Number128.ONE) {
        (0, assert_1.default)(this.kind === state_1.PositionKind.Claim);
        if (this.valueModifier.eq(__2.Number128.ZERO)) {
            console.log(`no leverage configured for claim ${this.token.toBase58()}`);
            return __2.Number128.MAX;
        }
        else {
            return this.valueRaw.div(this.valueModifier).div(setupLeverageFraction);
        }
    }
    setBalance(balance) {
        this.balance = balance;
        this.balanceTimestamp = (0, __1.getTimestamp)();
        this.calculateValue();
    }
    setPrice(price) {
        this.priceRaw = price;
        this.calculateValue();
    }
}
exports.AccountPosition = AccountPosition;
//# sourceMappingURL=accountPosition.js.map