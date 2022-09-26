import { Program, AnchorProvider } from "@project-serum/anchor";
import { JetMargin, JetMarginPool, JetMarginSerum, JetMarginSwap, JetMetadata, TokenAmount, PoolAction } from "..";
import { JetControl } from "../types";
import { MarginCluster, MarginConfig } from "./config";
import { ConfirmedSignatureInfo, Connection, ParsedTransactionWithMeta, PublicKey, TransactionResponse } from "@solana/web3.js";
interface TokenMintsList {
    tokenMint: PublicKey;
    depositNoteMint: PublicKey;
    loanNoteMint: PublicKey;
}
declare type Mints = Record<string, TokenMintsList>;
declare type TxAndSig = {
    details: TransactionResponse;
    sig: ConfirmedSignatureInfo;
};
export interface AccountTransaction {
    timestamp: number;
    blockDate: string;
    blockTime: string;
    signature: string;
    sigIndex: number;
    slot: number;
    tradeAction: PoolAction;
    tradeAmount: TokenAmount;
    tradeAmountInput?: TokenAmount;
    tokenSymbol: string;
    tokenName: string;
    tokenSymbolInput?: string;
    tokenNameInput?: string;
    tokenDecimals: number;
    fromAccount?: PublicKey;
    toAccount?: PublicKey;
    status: "error" | "success";
}
export interface MarginPrograms {
    config: MarginConfig;
    connection: Connection;
    control: Program<JetControl>;
    margin: Program<JetMargin>;
    marginPool: Program<JetMarginPool>;
    marginSerum: Program<JetMarginSerum>;
    marginSwap: Program<JetMarginSwap>;
    metadata: Program<JetMetadata>;
}
export declare class MarginClient {
    static getPrograms(provider: AnchorProvider, config: MarginConfig): MarginPrograms;
    static getConfig(cluster: MarginCluster): Promise<MarginConfig>;
    static getSingleTransaction(provider: AnchorProvider, sig: ConfirmedSignatureInfo): Promise<TxAndSig | null>;
    static getTransactionsFromSignatures(provider: AnchorProvider, signatures: ConfirmedSignatureInfo[]): Promise<TxAndSig[]>;
    static filterTransactions(transactions: (ParsedTransactionWithMeta | null)[], config: MarginConfig): ParsedTransactionWithMeta[];
    static getTransactionData(parsedTx: ParsedTransactionWithMeta, mints: Mints, config: MarginConfig, sigIndex: number, provider: AnchorProvider): Promise<AccountTransaction | null>;
    static getTransactionHistory(provider: AnchorProvider, pubKey: PublicKey, mints: Mints, cluster: MarginCluster, pageSize?: number): Promise<AccountTransaction[]>;
}
export {};
//# sourceMappingURL=marginClient.d.ts.map