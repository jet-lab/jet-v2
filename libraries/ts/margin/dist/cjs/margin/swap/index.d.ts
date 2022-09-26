/// <reference types="node" />
import * as BufferLayout from "@solana/buffer-layout";
import BN from "bn.js";
import { Account, ConfirmOptions, Connection, PublicKey, TransactionInstruction, TransactionSignature } from "@solana/web3.js";
export * from "./marginSwap";
export declare const TokenSwapLayout: BufferLayout.Structure<any>;
export declare const CurveType: Readonly<{
    ConstantProduct: number;
    ConstantPrice: number;
    Offset: number;
}>;
/**
 * A program to exchange tokens against a pool of liquidity
 */
export declare class TokenSwap {
    private connection;
    tokenSwap: PublicKey;
    swapProgramId: PublicKey;
    tokenProgramId: PublicKey;
    poolToken: PublicKey;
    feeAccount: PublicKey;
    authority: PublicKey;
    nonce: number;
    tokenAccountA: PublicKey;
    tokenAccountB: PublicKey;
    mintA: PublicKey;
    mintB: PublicKey;
    tradeFeeNumerator: BN;
    tradeFeeDenominator: BN;
    ownerTradeFeeNumerator: BN;
    ownerTradeFeeDenominator: BN;
    ownerWithdrawFeeNumerator: BN;
    ownerWithdrawFeeDenominator: BN;
    hostFeeNumerator: BN;
    hostFeeDenominator: BN;
    curveType: number;
    payer: Account;
    /**
     * Create a Token object attached to the specific token
     *
     * @param connection The connection to use
     * @param tokenSwap The token swap account
     * @param swapProgramId The program ID of the token-swap program
     * @param tokenProgramId The program ID of the token program
     * @param poolToken The pool token
     * @param authority The authority over the swap and accounts
     * @param tokenAccountA The token swap's Token A account
     * @param tokenAccountB The token swap's Token B account
     * @param mintA The mint of Token A
     * @param mintB The mint of Token B
     * @param tradeFeeNumerator The trade fee numerator
     * @param tradeFeeDenominator The trade fee denominator
     * @param ownerTradeFeeNumerator The owner trade fee numerator
     * @param ownerTradeFeeDenominator The owner trade fee denominator
     * @param ownerWithdrawFeeNumerator The owner withdraw fee numerator
     * @param ownerWithdrawFeeDenominator The owner withdraw fee denominator
     * @param hostFeeNumerator The host fee numerator
     * @param hostFeeDenominator The host fee denominator
     * @param curveType The curve type
     * @param payer Pays for the transaction
     */
    constructor(connection: Connection, tokenSwap: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, poolToken: PublicKey, feeAccount: PublicKey, authority: PublicKey, nonce: number, tokenAccountA: PublicKey, tokenAccountB: PublicKey, mintA: PublicKey, mintB: PublicKey, tradeFeeNumerator: BN, tradeFeeDenominator: BN, ownerTradeFeeNumerator: BN, ownerTradeFeeDenominator: BN, ownerWithdrawFeeNumerator: BN, ownerWithdrawFeeDenominator: BN, hostFeeNumerator: BN, hostFeeDenominator: BN, curveType: number, payer: Account);
    /**
     * Get the minimum balance for the token swap account to be rent exempt
     *
     * @return Number of lamports required
     */
    static getMinBalanceRentForExemptTokenSwap(connection: Connection): Promise<number>;
    static createInitSwapInstruction(tokenSwapAccount: Account, authority: PublicKey, nonce: number, tokenAccountA: PublicKey, tokenAccountB: PublicKey, tokenPool: PublicKey, feeAccount: PublicKey, tokenAccountPool: PublicKey, tokenProgramId: PublicKey, swapProgramId: PublicKey, tradeFeeNumerator: number, tradeFeeDenominator: number, ownerTradeFeeNumerator: number, ownerTradeFeeDenominator: number, ownerWithdrawFeeNumerator: number, ownerWithdrawFeeDenominator: number, hostFeeNumerator: number, hostFeeDenominator: number, curveType: number, curveParameters?: BN): TransactionInstruction;
    static loadAccount(connection: Connection, address: PublicKey, programId: PublicKey): Promise<Buffer>;
    static loadTokenSwap(connection: Connection, address: PublicKey, programId: PublicKey, payer: Account): Promise<TokenSwap>;
    /**
     * Create a new Token Swap
     *
     * @param connection The connection to use
     * @param payer Pays for the transaction
     * @param tokenSwapAccount The token swap account
     * @param authority The authority over the swap and accounts
     * @param tokenAccountA: The token swap's Token A account
     * @param tokenAccountB: The token swap's Token B account
     * @param poolToken The pool token
     * @param tokenAccountPool The token swap's pool token account
     * @param tokenProgramId The program ID of the token program
     * @param swapProgramId The program ID of the token-swap program
     * @param feeNumerator Numerator of the fee ratio
     * @param feeDenominator Denominator of the fee ratio
     * @return Token object for the newly minted token, Public key of the account holding the total supply of new tokens
     */
    static createTokenSwap(connection: Connection, payer: Account, tokenSwapAccount: Account, authority: PublicKey, nonce: number, tokenAccountA: PublicKey, tokenAccountB: PublicKey, poolToken: PublicKey, mintA: PublicKey, mintB: PublicKey, feeAccount: PublicKey, tokenAccountPool: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, tradeFeeNumerator: number, tradeFeeDenominator: number, ownerTradeFeeNumerator: number, ownerTradeFeeDenominator: number, ownerWithdrawFeeNumerator: number, ownerWithdrawFeeDenominator: number, hostFeeNumerator: number, hostFeeDenominator: number, curveType: number, curveParameters?: BN, confirmOptions?: ConfirmOptions): Promise<TokenSwap>;
    /**
     * Swap token A for token B
     *
     * @param userSource User's source token account
     * @param poolSource Pool's source token account
     * @param poolDestination Pool's destination token account
     * @param userDestination User's destination token account
     * @param hostFeeAccount Host account to gather fees
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param amountIn Amount to transfer from source account
     * @param minimumAmountOut Minimum amount of tokens the user will receive
     */
    swap(userSource: PublicKey, poolSource: PublicKey, poolDestination: PublicKey, userDestination: PublicKey, hostFeeAccount: PublicKey | null, userTransferAuthority: Account, amountIn: BN, minimumAmountOut: BN, confirmOptions?: ConfirmOptions): Promise<TransactionSignature>;
    static swapInstruction(tokenSwap: PublicKey, authority: PublicKey, userTransferAuthority: PublicKey, userSource: PublicKey, poolSource: PublicKey, poolDestination: PublicKey, userDestination: PublicKey, poolMint: PublicKey, feeAccount: PublicKey, hostFeeAccount: PublicKey | null, swapProgramId: PublicKey, tokenProgramId: PublicKey, amountIn: BN, minimumAmountOut: BN): TransactionInstruction;
    /**
     * Deposit tokens into the pool
     * @param userAccountA User account for token A
     * @param userAccountB User account for token B
     * @param poolAccount User account for pool token
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param poolTokenAmount Amount of pool tokens to mint
     * @param maximumTokenA The maximum amount of token A to deposit
     * @param maximumTokenB The maximum amount of token B to deposit
     */
    depositAllTokenTypes(userAccountA: PublicKey, userAccountB: PublicKey, poolAccount: PublicKey, userTransferAuthority: Account, poolTokenAmount: BN, maximumTokenA: BN, maximumTokenB: BN, confirmOptions?: ConfirmOptions): Promise<TransactionSignature>;
    static depositAllTokenTypesInstruction(tokenSwap: PublicKey, authority: PublicKey, userTransferAuthority: PublicKey, sourceA: PublicKey, sourceB: PublicKey, intoA: PublicKey, intoB: PublicKey, poolToken: PublicKey, poolAccount: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, poolTokenAmount: BN, maximumTokenA: BN, maximumTokenB: BN): TransactionInstruction;
    /**
     * Withdraw tokens from the pool
     *
     * @param userAccountA User account for token A
     * @param userAccountB User account for token B
     * @param poolAccount User account for pool token
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param poolTokenAmount Amount of pool tokens to burn
     * @param minimumTokenA The minimum amount of token A to withdraw
     * @param minimumTokenB The minimum amount of token B to withdraw
     */
    withdrawAllTokenTypes(userAccountA: PublicKey, userAccountB: PublicKey, poolAccount: PublicKey, userTransferAuthority: Account, poolTokenAmount: BN, minimumTokenA: BN, minimumTokenB: BN, confirmOptions?: ConfirmOptions): Promise<TransactionSignature>;
    static withdrawAllTokenTypesInstruction(tokenSwap: PublicKey, authority: PublicKey, userTransferAuthority: PublicKey, poolMint: PublicKey, feeAccount: PublicKey, sourcePoolAccount: PublicKey, fromA: PublicKey, fromB: PublicKey, userAccountA: PublicKey, userAccountB: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, poolTokenAmount: BN, minimumTokenA: BN, minimumTokenB: BN): TransactionInstruction;
    /**
     * Deposit one side of tokens into the pool
     * @param userAccount User account to deposit token A or B
     * @param poolAccount User account to receive pool tokens
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param sourceTokenAmount The amount of token A or B to deposit
     * @param minimumPoolTokenAmount Minimum amount of pool tokens to mint
     */
    depositSingleTokenTypeExactAmountIn(userAccount: PublicKey, poolAccount: PublicKey, userTransferAuthority: Account, sourceTokenAmount: BN, minimumPoolTokenAmount: BN, confirmOptions?: ConfirmOptions): Promise<TransactionSignature>;
    static depositSingleTokenTypeExactAmountInInstruction(tokenSwap: PublicKey, authority: PublicKey, userTransferAuthority: PublicKey, source: PublicKey, intoA: PublicKey, intoB: PublicKey, poolToken: PublicKey, poolAccount: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, sourceTokenAmount: BN, minimumPoolTokenAmount: BN): TransactionInstruction;
    /**
     * Withdraw tokens from the pool
     *
     * @param userAccount User account to receive token A or B
     * @param poolAccount User account to burn pool token
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param destinationTokenAmount The amount of token A or B to withdraw
     * @param maximumPoolTokenAmount Maximum amount of pool tokens to burn
     */
    withdrawSingleTokenTypeExactAmountOut(userAccount: PublicKey, poolAccount: PublicKey, userTransferAuthority: Account, destinationTokenAmount: BN, maximumPoolTokenAmount: BN, confirmOptions?: ConfirmOptions): Promise<TransactionSignature>;
    static withdrawSingleTokenTypeExactAmountOutInstruction(tokenSwap: PublicKey, authority: PublicKey, userTransferAuthority: PublicKey, poolMint: PublicKey, feeAccount: PublicKey, sourcePoolAccount: PublicKey, fromA: PublicKey, fromB: PublicKey, userAccount: PublicKey, swapProgramId: PublicKey, tokenProgramId: PublicKey, destinationTokenAmount: BN, maximumPoolTokenAmount: BN): TransactionInstruction;
}
//# sourceMappingURL=index.d.ts.map