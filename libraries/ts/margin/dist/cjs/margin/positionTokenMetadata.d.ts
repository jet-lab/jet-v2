/// <reference types="node" />
import { AccountInfo, PublicKey } from "@solana/web3.js";
import { Number128 } from "../utils";
import { MarginPrograms } from "./marginClient";
import { PositionTokenMetadataInfo, TokenKind } from "./metadata";
import { PositionKind } from "./state";
export declare class PositionTokenMetadata {
    private programs;
    tokenMint: PublicKey;
    address: PublicKey;
    info: PositionTokenMetadataInfo | undefined;
    valueModifier: Number128;
    tokenKind: PositionKind;
    static derive(programs: MarginPrograms, tokenMint: PublicKey): PublicKey;
    constructor({ programs, tokenMint }: {
        programs: MarginPrograms;
        tokenMint: PublicKey;
    });
    static load(programs: MarginPrograms, tokenMint: PublicKey): Promise<PositionTokenMetadata>;
    refresh(): Promise<void>;
    decode(info: AccountInfo<Buffer> | null): void;
    static decodeTokenKind(kind: TokenKind): PositionKind;
    getLiability(value: Number128): Number128;
    collateralValue(value: Number128): Number128;
    requiredCollateralValue(value: Number128): Number128;
}
//# sourceMappingURL=positionTokenMetadata.d.ts.map