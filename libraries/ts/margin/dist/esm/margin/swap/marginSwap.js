import assert from "assert";
import * as BufferLayout from "@solana/buffer-layout";
import { BN } from "@project-serum/anchor";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccountInstruction, getAssociatedTokenAddress, MintLayout, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { sendAndConfirmTransaction, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js";
import { TokenSwap } from "./index";
export class MarginSwap {
    constructor(tokenSwap) {
        this.tokenSwap = tokenSwap;
        assert(tokenSwap);
    }
    static async load(connection, tokenSwapAddress, payer, splTokenSwapProgramId) {
        const tokenSwap = await TokenSwap.loadTokenSwap(connection, tokenSwapAddress, splTokenSwapProgramId, payer);
        return new MarginSwap(tokenSwap);
    }
    static async create(connection, payer, tokenSwapAccount, authority, authorityNonce, tokenAccountA, tokenAccountB, tokenPool, mintA, mintB, feeAccount, tokenAccountPool, splTokenSwapProgramId, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType) {
        const tokenSwap = await TokenSwap.createTokenSwap(connection, payer, tokenSwapAccount, authority, authorityNonce, tokenAccountA, tokenAccountB, tokenPool, mintA, mintB, feeAccount, tokenAccountPool, splTokenSwapProgramId, TOKEN_PROGRAM_ID, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType);
        return tokenSwap;
    }
    async approve(connection, account, delegate, owner, amount, payer) {
        const tx = new Transaction();
        tx.add(this.createApproveInstruction(TOKEN_PROGRAM_ID, account, delegate, owner.publicKey, amount));
        await sendAndConfirmTransaction(connection, tx, [payer, owner]);
    }
    createApproveInstruction(programId, account, delegate, owner, amount) {
        const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction"), BufferLayout.blob(8, "amount")]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 4,
            amount: new BN(amount).toArrayLike(Buffer, "le", 8)
        }, data);
        const keys = [
            { pubkey: account, isSigner: false, isWritable: true },
            { pubkey: delegate, isSigner: false, isWritable: false },
            { pubkey: owner, isSigner: true, isWritable: false }
        ];
        return new TransactionInstruction({
            keys,
            programId: programId,
            data
        });
    }
    static async createAssociatedTokenAccount(connection, payer, mint, owner) {
        const associatedToken = await getAssociatedTokenAddress(mint, owner, true);
        const transaction = new Transaction().add(createAssociatedTokenAccountInstruction(payer.publicKey, associatedToken, owner, mint, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID));
        await sendAndConfirmTransaction(connection, transaction, [payer], {
            skipPreflight: true
        });
        return associatedToken;
    }
    createInitAccountInstruction(programId, mint, account, owner) {
        const keys = [
            { pubkey: account, isSigner: false, isWritable: true },
            { pubkey: mint, isSigner: false, isWritable: false },
            { pubkey: owner, isSigner: false, isWritable: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
        ];
        const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction")]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 1 // InitializeAccount instruction
        }, data);
        return new TransactionInstruction({
            keys,
            programId,
            data
        });
    }
    static async getMintInfo(connection, mint) {
        const info = await connection.getAccountInfo(mint);
        if (info === null) {
            throw new Error("Failed to find mint account");
        }
        if (!info.owner.equals(TOKEN_PROGRAM_ID)) {
            throw new Error(`Invalid mint owner: ${JSON.stringify(info.owner)}`);
        }
        if (info.data.length != MintLayout.span) {
            throw new Error(`Invalid mint size`);
        }
        const data = Buffer.from(info.data);
        return MintLayout.decode(data);
    }
}
//# sourceMappingURL=marginSwap.js.map