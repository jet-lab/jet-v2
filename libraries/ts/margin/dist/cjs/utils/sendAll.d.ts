import { AnchorProvider } from "@project-serum/anchor";
import { ConfirmOptions, Signer, TransactionInstruction, TransactionSignature } from "@solana/web3.js";
/**
 * Sends the given transaction, paid for and signed by the provider's wallet.
 *
 * @param tx      The transaction to send.
 * @param signers The signers of the transaction.
 * @param opts    Transaction confirmation options.
 */
export declare function sendAndConfirm(provider: AnchorProvider, instructions: TransactionInstruction[], signers?: Signer[], opts?: ConfirmOptions): Promise<TransactionSignature>;
/**
 * Sends all transactions. If an entry in the transactions array is
 * a sub-array, then transactions within the sub array are sent in parallel
 */
export declare function sendAll(provider: AnchorProvider, transactions: (TransactionInstruction[] | TransactionInstruction[][])[], opts?: ConfirmOptions): Promise<string>;
//# sourceMappingURL=sendAll.d.ts.map