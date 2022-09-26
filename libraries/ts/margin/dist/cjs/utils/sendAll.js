"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.sendAll = exports.sendAndConfirm = void 0;
const web3_js_1 = require("@solana/web3.js");
const bs58_1 = __importDefault(require("bs58"));
/**
 * Sends the given transaction, paid for and signed by the provider's wallet.
 *
 * @param tx      The transaction to send.
 * @param signers The signers of the transaction.
 * @param opts    Transaction confirmation options.
 */
async function sendAndConfirm(provider, instructions, signers, opts) {
    if (opts === undefined) {
        opts = provider.opts;
    }
    const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash(opts.preflightCommitment);
    const transaction = new web3_js_1.Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(...instructions);
    transaction.feePayer = provider.wallet.publicKey;
    transaction.recentBlockhash = (await provider.connection.getRecentBlockhash(opts.preflightCommitment)).blockhash;
    if (signers?.length) {
        transaction.partialSign(...signers);
    }
    const signedTransaction = await provider.wallet.signTransaction(transaction);
    const rawTx = signedTransaction.serialize();
    try {
        return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts);
    }
    catch (err) {
        // thrown if the underlying 'confirmTransaction' encounters a failed tx
        // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
        // choose the shortest available commitment for 'getTransaction'
        // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
        // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
        // commitment `sendAndConfirmRawTransaction` used
        await provider.connection.confirmTransaction(bs58_1.default.encode(signedTransaction.signature), "confirmed");
        const failedTx = await provider.connection.getTransaction(bs58_1.default.encode(signedTransaction.signature), {
            commitment: "confirmed"
        });
        const logs = failedTx?.meta?.logMessages;
        const message = `${err.message}\n${JSON.stringify(logs, undefined, 2)}`;
        throw !logs ? err : new web3_js_1.SendTransactionError(message);
    }
}
exports.sendAndConfirm = sendAndConfirm;
/**
 * Sends all transactions. If an entry in the transactions array is
 * a sub-array, then transactions within the sub array are sent in parallel
 */
async function sendAll(provider, transactions, opts) {
    if (opts === undefined) {
        opts = provider.opts;
    }
    const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash(opts.preflightCommitment);
    const txs = transactions
        .map(tx => {
        if (Array.isArray(tx[0])) {
            return tx
                .map((tx) => {
                const ixs = tx;
                if (ixs.length > 0) {
                    return new web3_js_1.Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(...ixs);
                }
            })
                .filter(tx => !!tx);
        }
        else {
            const ixs = tx;
            if (ixs.length > 0) {
                return [new web3_js_1.Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(...ixs)];
            }
        }
    })
        .filter(tx => !!tx);
    let start = 0;
    const slices = txs.map(tx => {
        const length = Array.isArray(tx) ? tx.length : 1;
        const end = start + length;
        const slice = [start, end];
        start = end;
        return slice;
    });
    // signedTxs has been flattened. unflatten it
    const signedTxs = await provider.wallet.signAllTransactions(txs.flat(1));
    const signedUnflattened = slices.map(slice => signedTxs.slice(...slice));
    let lastTxn = "";
    for (let i = 0; i < signedUnflattened.length; i++) {
        const transactions = signedUnflattened[i];
        const txnArray = await Promise.all(transactions.map(async (tx) => {
            const rawTx = tx.serialize();
            try {
                return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts);
            }
            catch (err) {
                // Thrown if the underlying 'confirmTransaction' encounters a failed tx
                // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
                // Choose the shortest available commitment for 'getTransaction'
                // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
                // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
                // commitment `sendAndConfirmRawTransaction` used
                await provider.connection.confirmTransaction(bs58_1.default.encode(tx.signature), "confirmed");
                const failedTx = await provider.connection.getTransaction(bs58_1.default.encode(tx.signature), {
                    commitment: "confirmed"
                });
                const logs = failedTx?.meta?.logMessages;
                const message = `${err.message}\n${JSON.stringify(logs, undefined, 2)}`;
                throw !logs ? err : new web3_js_1.SendTransactionError(message);
            }
        }));
        // Return the txid of the final transaction in the array
        // TODO: We should return an array instead of only the final txn
        lastTxn = txnArray[txnArray.length - 1] ?? "";
    }
    return lastTxn;
}
exports.sendAll = sendAll;
// Copy of Connection.sendAndConfirmRawTransaction that throws
// a better error if 'confirmTransaction` returns an error status
async function sendAndConfirmRawTransaction(connection, rawTransaction, options) {
    const sendOptions = options && {
        skipPreflight: options.skipPreflight,
        preflightCommitment: options.preflightCommitment || options.commitment
    };
    const signature = await connection.sendRawTransaction(rawTransaction, sendOptions);
    const status = (await connection.confirmTransaction(signature, options && options.commitment)).value;
    if (status.err) {
        throw new ConfirmError(`Raw transaction ${signature} failed (${JSON.stringify(status)})`);
    }
    return signature;
}
class ConfirmError extends Error {
    constructor(message) {
        super(message);
    }
}
//# sourceMappingURL=sendAll.js.map