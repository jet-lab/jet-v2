import { BN } from "@project-serum/anchor";
import { Account, Connection, PublicKey, Signer, TransactionInstruction } from "@solana/web3.js";
import { TokenSwap } from "./index";
export declare class MarginSwap {
    tokenSwap: TokenSwap;
    constructor(tokenSwap: TokenSwap);
    static load(connection: Connection, tokenSwapAddress: PublicKey, payer: Account, splTokenSwapProgramId: PublicKey): Promise<MarginSwap>;
    static create(connection: Connection, payer: Account, tokenSwapAccount: Account, authority: PublicKey, authorityNonce: number, tokenAccountA: PublicKey, tokenAccountB: PublicKey, tokenPool: PublicKey, mintA: PublicKey, mintB: PublicKey, feeAccount: PublicKey, tokenAccountPool: PublicKey, splTokenSwapProgramId: PublicKey, tradeFeeNumerator: number, tradeFeeDenominator: number, ownerTradeFeeNumerator: number, ownerTradeFeeDenominator: number, ownerWithdrawFeeNumerator: number, ownerWithdrawFeeDenominator: number, hostFeeNumerator: number, hostFeeDenominator: number, curveType: number): Promise<TokenSwap>;
    approve(connection: Connection, account: PublicKey, delegate: PublicKey, owner: Account, amount: BN, payer: Account): Promise<void>;
    createApproveInstruction(programId: PublicKey, account: PublicKey, delegate: PublicKey, owner: PublicKey, amount: BN): TransactionInstruction;
    static createAssociatedTokenAccount(connection: Connection, payer: Signer, mint: PublicKey, owner: PublicKey): Promise<PublicKey>;
    createInitAccountInstruction(programId: PublicKey, mint: PublicKey, account: PublicKey, owner: PublicKey): TransactionInstruction;
    static getMintInfo(connection: Connection, mint: PublicKey): Promise<import("@solana/spl-token").RawMint>;
}
//# sourceMappingURL=marginSwap.d.ts.map