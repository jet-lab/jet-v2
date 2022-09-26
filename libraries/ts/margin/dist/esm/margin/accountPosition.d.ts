import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Number128 } from "../";
import { AccountPositionInfo, AdapterPositionFlags, PositionKindInfo } from "./state";
export interface PriceInfo {
    value: BN;
    exponent: number;
    timestamp: BN;
    isValid: number;
}
export declare class AccountPosition {
    /** The raw account position deserialized by anchor */
    info: AccountPositionInfo;
    /** The address of the token/mint of the asset */
    token: PublicKey;
    /** The address of the account holding the tokens. */
    address: PublicKey;
    /** The address of the adapter managing the asset */
    adapter: PublicKey;
    /** The current value of this position. */
    valueRaw: Number128;
    get value(): number;
    /** The amount of tokens in the account */
    balance: BN;
    /** The timestamp of the last balance update */
    balanceTimestamp: BN;
    /** The current price/value of each token */
    priceRaw: PriceInfo;
    get price(): Number128;
    /** The kind of balance this position contains */
    kind: PositionKindInfo;
    /** The exponent for the token value */
    exponent: number;
    /** A weight on the value of this asset when counting collateral */
    valueModifier: Number128;
    /** The max staleness for the account balance (seconds) */
    maxStaleness: BN;
    /** Flags that are set by the adapter */
    flags: AdapterPositionFlags;
    constructor({ info, price }: {
        info: AccountPositionInfo;
        price?: PriceInfo;
    });
    calculateValue(): void;
    collateralValue(): Number128;
    requiredCollateralValue(setupLeverageFraction?: Number128): Number128;
    setBalance(balance: BN): void;
    setPrice(price: PriceInfo): void;
}
//# sourceMappingURL=accountPosition.d.ts.map