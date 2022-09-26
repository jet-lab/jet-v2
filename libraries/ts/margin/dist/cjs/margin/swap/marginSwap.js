"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MarginSwap = void 0;
const assert_1 = __importDefault(require("assert"));
const BufferLayout = __importStar(require("@solana/buffer-layout"));
const anchor_1 = require("@project-serum/anchor");
const spl_token_1 = require("@solana/spl-token");
const web3_js_1 = require("@solana/web3.js");
const index_1 = require("./index");
class MarginSwap {
    constructor(tokenSwap) {
        this.tokenSwap = tokenSwap;
        (0, assert_1.default)(tokenSwap);
    }
    static async load(connection, tokenSwapAddress, payer, splTokenSwapProgramId) {
        const tokenSwap = await index_1.TokenSwap.loadTokenSwap(connection, tokenSwapAddress, splTokenSwapProgramId, payer);
        return new MarginSwap(tokenSwap);
    }
    static async create(connection, payer, tokenSwapAccount, authority, authorityNonce, tokenAccountA, tokenAccountB, tokenPool, mintA, mintB, feeAccount, tokenAccountPool, splTokenSwapProgramId, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType) {
        const tokenSwap = await index_1.TokenSwap.createTokenSwap(connection, payer, tokenSwapAccount, authority, authorityNonce, tokenAccountA, tokenAccountB, tokenPool, mintA, mintB, feeAccount, tokenAccountPool, splTokenSwapProgramId, spl_token_1.TOKEN_PROGRAM_ID, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType);
        return tokenSwap;
    }
    async approve(connection, account, delegate, owner, amount, payer) {
        const tx = new web3_js_1.Transaction();
        tx.add(this.createApproveInstruction(spl_token_1.TOKEN_PROGRAM_ID, account, delegate, owner.publicKey, amount));
        await (0, web3_js_1.sendAndConfirmTransaction)(connection, tx, [payer, owner]);
    }
    createApproveInstruction(programId, account, delegate, owner, amount) {
        const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction"), BufferLayout.blob(8, "amount")]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 4,
            amount: new anchor_1.BN(amount).toArrayLike(Buffer, "le", 8)
        }, data);
        const keys = [
            { pubkey: account, isSigner: false, isWritable: true },
            { pubkey: delegate, isSigner: false, isWritable: false },
            { pubkey: owner, isSigner: true, isWritable: false }
        ];
        return new web3_js_1.TransactionInstruction({
            keys,
            programId: programId,
            data
        });
    }
    static async createAssociatedTokenAccount(connection, payer, mint, owner) {
        const associatedToken = await (0, spl_token_1.getAssociatedTokenAddress)(mint, owner, true);
        const transaction = new web3_js_1.Transaction().add((0, spl_token_1.createAssociatedTokenAccountInstruction)(payer.publicKey, associatedToken, owner, mint, spl_token_1.TOKEN_PROGRAM_ID, spl_token_1.ASSOCIATED_TOKEN_PROGRAM_ID));
        await (0, web3_js_1.sendAndConfirmTransaction)(connection, transaction, [payer], {
            skipPreflight: true
        });
        return associatedToken;
    }
    createInitAccountInstruction(programId, mint, account, owner) {
        const keys = [
            { pubkey: account, isSigner: false, isWritable: true },
            { pubkey: mint, isSigner: false, isWritable: false },
            { pubkey: owner, isSigner: false, isWritable: false },
            { pubkey: web3_js_1.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
        ];
        const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction")]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 1 // InitializeAccount instruction
        }, data);
        return new web3_js_1.TransactionInstruction({
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
        if (!info.owner.equals(spl_token_1.TOKEN_PROGRAM_ID)) {
            throw new Error(`Invalid mint owner: ${JSON.stringify(info.owner)}`);
        }
        if (info.data.length != spl_token_1.MintLayout.span) {
            throw new Error(`Invalid mint size`);
        }
        const data = Buffer.from(info.data);
        return spl_token_1.MintLayout.decode(data);
    }
}
exports.MarginSwap = MarginSwap;
//# sourceMappingURL=marginSwap.js.map