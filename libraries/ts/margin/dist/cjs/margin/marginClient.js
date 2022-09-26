"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MarginClient = void 0;
const spl_token_1 = require("@solana/spl-token");
const anchor_1 = require("@project-serum/anchor");
const __1 = require("..");
const types_1 = require("../types");
const config_1 = require("./config");
const web3_js_1 = require("@solana/web3.js");
class MarginClient {
    static getPrograms(provider, config) {
        const programs = {
            config,
            connection: provider.connection,
            control: new anchor_1.Program(types_1.JetControlIdl, config.controlProgramId, provider),
            margin: new anchor_1.Program(types_1.JetMarginIdl, config.marginProgramId, provider),
            marginPool: new anchor_1.Program(types_1.JetMarginPoolIdl, config.marginPoolProgramId, provider),
            marginSerum: new anchor_1.Program(types_1.JetMarginSerumIdl, config.marginSerumProgramId, provider),
            marginSwap: new anchor_1.Program(types_1.JetMarginSwapIdl, config.marginSwapProgramId, provider),
            metadata: new anchor_1.Program(types_1.JetMetadataIdl, config.metadataProgramId, provider)
        };
        return programs;
    }
    static async getConfig(cluster) {
        if (typeof cluster === "string") {
            return await (0, config_1.getLatestConfig)(cluster);
        }
        else {
            return cluster;
        }
    }
    static async getSingleTransaction(provider, sig) {
        const details = await provider.connection.getTransaction(sig.signature, { commitment: "confirmed" });
        if (details) {
            return {
                details,
                sig
            };
        }
        else {
            return null;
        }
    }
    static async getTransactionsFromSignatures(provider, signatures) {
        const responses = await Promise.all(signatures.map(sig => MarginClient.getSingleTransaction(provider, sig)));
        return responses.filter(res => res !== null);
    }
    static filterTransactions(transactions, config) {
        return transactions.filter(t => {
            if (t?.meta?.logMessages?.some(tx => tx.includes(config.marginPoolProgramId.toString()))) {
                return true;
            }
            else {
                return false;
            }
        });
    }
    static async getTransactionData(parsedTx, mints, config, sigIndex, provider) {
        if (!parsedTx.meta?.logMessages || !parsedTx.blockTime) {
            return null;
        }
        const instructions = {
            deposit: "Instruction: Deposit",
            withdraw: "Instruction: Withdraw",
            borrow: "Instruction: MarginBorrow",
            "margin repay": "Instruction: MarginRepay",
            repay: "Instruction: Repay",
            swap: "Instruction: MarginSwap"
        };
        let tradeAction = "";
        // Check to see if logMessage string contains relevant instruction
        // If it does, set tradeAction to that element
        const isTradeInstruction = (logLine) => {
            for (const action of Object.keys(instructions)) {
                if (logLine.includes(instructions[action])) {
                    tradeAction = action;
                    return true;
                }
            }
        };
        const setupAccountTx = (token, amount, parsedTx, amountIn, tokenIn) => {
            tx.tokenSymbol = token.symbol;
            tx.tokenName = token.name;
            tx.tokenDecimals = token.decimals;
            tx.tradeAmount = __1.TokenAmount.lamports(amount, token.decimals);
            // tokenIn applies if the trade type is a swap
            // For the input token only
            // Default is the output token
            if (tokenIn) {
                tx.tokenSymbolInput = tokenIn.symbol;
                tx.tokenNameInput = tokenIn.name;
                tx.tradeAmountInput = __1.TokenAmount.lamports(amountIn, tokenIn.decimals);
            }
            const dateTime = new Date(parsedTx.blockTime * 1000);
            tx.timestamp = parsedTx.blockTime;
            tx.blockDate = dateTime.toLocaleDateString();
            tx.blockTime = dateTime.toLocaleTimeString("en-US", { hour12: false });
            tx.slot = parsedTx.slot;
            tx.sigIndex = sigIndex ? sigIndex : 0;
            tx.signature = parsedTx.transaction.signatures[0];
            tx.status = parsedTx.meta?.err ? "error" : "success";
            return tx;
        };
        // Check each logMessage string for instruction
        for (let i = 0; i < parsedTx.meta.logMessages.length; i++) {
            if (isTradeInstruction(parsedTx.meta?.logMessages[i])) {
                // Break after finding the first logMessage for which above is true
                break;
            }
        }
        if (!tradeAction || !parsedTx.meta?.postTokenBalances || !parsedTx.meta?.preTokenBalances) {
            return null;
        }
        const tx = {
            tradeAction
        };
        for (let i = 0; i < parsedTx.meta.preTokenBalances?.length; i++) {
            const pre = parsedTx.meta.preTokenBalances[i];
            const matchingPost = parsedTx.meta.postTokenBalances?.find(post => post.mint === pre.mint && post.owner === pre.owner);
            if (matchingPost && matchingPost.uiTokenAmount.amount !== pre.uiTokenAmount.amount) {
                let token = null;
                let tokenIn = null;
                const ixs = parsedTx.meta.innerInstructions;
                const parsedIxnArray = [];
                let amount = new anchor_1.BN(0);
                let amountIn = new anchor_1.BN(0);
                ixs?.forEach((ix) => {
                    ix.instructions.forEach((inst) => {
                        if ("parsed" in inst) {
                            if (inst.parsed && inst.parsed.type === "transfer" && inst?.parsed.info.amount !== "0") {
                                parsedIxnArray.push(inst);
                                // Default amount is the value of the final parsed instruction
                                amount = new anchor_1.BN(inst.parsed.info.amount);
                            }
                        }
                    });
                });
                // If trade action is swap, set up input amount as well
                // Get value of amount in the first parsed instruction
                if (tradeAction === "swap" && parsedIxnArray[0]) {
                    amountIn = new anchor_1.BN(parsedIxnArray[0].parsed.info.amount);
                }
                // if we could not find a token transfer, default to token values changes
                if (amount.eq(new anchor_1.BN(0))) {
                    const postAmount = new anchor_1.BN(matchingPost.uiTokenAmount.amount);
                    const preAmount = new anchor_1.BN(pre.uiTokenAmount.amount);
                    amount = postAmount.sub(preAmount).abs();
                }
                for (let j = 0; j < Object.entries(mints).length; j++) {
                    const tokenAbbrev = Object.entries(mints)[j][0];
                    const tokenMints = Object.entries(mints)[j][1];
                    if (Object.values(tokenMints)
                        .map((t) => t.toBase58())
                        .includes(matchingPost.mint)) {
                        if (tradeAction === "swap") {
                            // If trade action is swap,
                            // Set up correct target mint
                            const transferIxs = [];
                            ixs?.forEach((ix) => {
                                ix.instructions.forEach((inst) => {
                                    if ("parsed" in inst) {
                                        if (inst.parsed && inst.parsed.type === "transfer") {
                                            transferIxs.push(inst);
                                        }
                                    }
                                });
                            });
                            const firstTransferIxSource = transferIxs[0].parsed.info.source;
                            const finalTransferIxSource = transferIxs[transferIxs.length - 1].parsed.info.source;
                            const firstMint = await (0, spl_token_1.getAccount)(provider.connection, new web3_js_1.PublicKey(firstTransferIxSource));
                            const sourceAccountMint = await (0, spl_token_1.getAccount)(provider.connection, new web3_js_1.PublicKey(finalTransferIxSource));
                            const tokenConfig = Object.values(config.tokens).find(config => sourceAccountMint.mint.equals(new web3_js_1.PublicKey(config.mint)));
                            const firstTokenConfig = Object.values(config.tokens).find(config => firstMint.mint.equals(new web3_js_1.PublicKey(config.mint)));
                            token = tokenConfig;
                            tokenIn = firstTokenConfig;
                        }
                        else {
                            token = config.tokens[tokenAbbrev];
                        }
                        if ((0, anchor_1.translateAddress)(token.mint).equals(spl_token_1.NATIVE_MINT) &&
                            (tradeAction === "withdraw" || tradeAction === "borrow") &&
                            matchingPost.uiTokenAmount.amount === "0") {
                            break;
                        }
                        return setupAccountTx(token, amount, parsedTx, amountIn, tokenIn);
                    }
                }
            }
        }
        return null;
    }
    static async getTransactionHistory(provider, pubKey, mints, cluster, pageSize = 100) {
        const config = await MarginClient.getConfig(cluster);
        const signatures = await provider.connection.getSignaturesForAddress(pubKey, undefined, "confirmed");
        const jetTransactions = [];
        let page = 0;
        let processed = 0;
        while (processed < signatures.length) {
            const paginatedSignatures = signatures.slice(page * pageSize, (page + 1) * pageSize);
            const transactions = await provider.connection.getParsedTransactions(paginatedSignatures.map(s => s.signature), "confirmed");
            const filteredTxs = MarginClient.filterTransactions(transactions, config);
            jetTransactions.push(...filteredTxs);
            page++;
            processed += paginatedSignatures.length;
        }
        const parsedTransactions = await Promise.all(jetTransactions.map(async (t, idx) => await MarginClient.getTransactionData(t, mints, config, idx, provider)));
        const filteredParsedTransactions = parsedTransactions.filter(tx => !!tx);
        return filteredParsedTransactions.sort((a, b) => a.slot - b.slot);
    }
}
exports.MarginClient = MarginClient;
//# sourceMappingURL=marginClient.js.map