import * as BufferLayout from "@solana/buffer-layout";
import BN from "bn.js";
import { PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import * as Layout from "../../utils";
import { u64 } from "../../utils";
export * from "./marginSwap";
export const TokenSwapLayout = BufferLayout.struct([
    BufferLayout.u8("version"),
    BufferLayout.u8("isInitialized"),
    BufferLayout.u8("bumpSeed"),
    Layout.pubkey("tokenProgramId"),
    Layout.pubkey("tokenAccountA"),
    Layout.pubkey("tokenAccountB"),
    Layout.pubkey("tokenPool"),
    Layout.pubkey("mintA"),
    Layout.pubkey("mintB"),
    Layout.pubkey("feeAccount"),
    Layout.u64("tradeFeeNumerator"),
    Layout.u64("tradeFeeDenominator"),
    Layout.u64("ownerTradeFeeNumerator"),
    Layout.u64("ownerTradeFeeDenominator"),
    Layout.u64("ownerWithdrawFeeNumerator"),
    Layout.u64("ownerWithdrawFeeDenominator"),
    Layout.u64("hostFeeNumerator"),
    Layout.u64("hostFeeDenominator"),
    BufferLayout.u8("curveType"),
    BufferLayout.blob(32, "curveParameters")
]);
export const CurveType = Object.freeze({
    ConstantProduct: 0,
    ConstantPrice: 1,
    Offset: 3 // Offset curve, like Uniswap, but with an additional offset on the token B side
});
/**
 * A program to exchange tokens against a pool of liquidity
 */
export class TokenSwap {
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
    constructor(connection, tokenSwap, swapProgramId, tokenProgramId, poolToken, feeAccount, authority, nonce, tokenAccountA, tokenAccountB, mintA, mintB, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType, payer) {
        this.connection = connection;
        this.tokenSwap = tokenSwap;
        this.swapProgramId = swapProgramId;
        this.tokenProgramId = tokenProgramId;
        this.poolToken = poolToken;
        this.feeAccount = feeAccount;
        this.authority = authority;
        this.nonce = nonce;
        this.tokenAccountA = tokenAccountA;
        this.tokenAccountB = tokenAccountB;
        this.mintA = mintA;
        this.mintB = mintB;
        this.tradeFeeNumerator = tradeFeeNumerator;
        this.tradeFeeDenominator = tradeFeeDenominator;
        this.ownerTradeFeeNumerator = ownerTradeFeeNumerator;
        this.ownerTradeFeeDenominator = ownerTradeFeeDenominator;
        this.ownerWithdrawFeeNumerator = ownerWithdrawFeeNumerator;
        this.ownerWithdrawFeeDenominator = ownerWithdrawFeeDenominator;
        this.hostFeeNumerator = hostFeeNumerator;
        this.hostFeeDenominator = hostFeeDenominator;
        this.curveType = curveType;
        this.payer = payer;
        this.connection = connection;
        this.tokenSwap = tokenSwap;
        this.swapProgramId = swapProgramId;
        this.tokenProgramId = tokenProgramId;
        this.poolToken = poolToken;
        this.feeAccount = feeAccount;
        this.authority = authority;
        this.nonce = nonce;
        this.tokenAccountA = tokenAccountA;
        this.tokenAccountB = tokenAccountB;
        this.mintA = mintA;
        this.mintB = mintB;
        this.tradeFeeNumerator = tradeFeeNumerator;
        this.tradeFeeDenominator = tradeFeeDenominator;
        this.ownerTradeFeeNumerator = ownerTradeFeeNumerator;
        this.ownerTradeFeeDenominator = ownerTradeFeeDenominator;
        this.ownerWithdrawFeeNumerator = ownerWithdrawFeeNumerator;
        this.ownerWithdrawFeeDenominator = ownerWithdrawFeeDenominator;
        this.hostFeeNumerator = hostFeeNumerator;
        this.hostFeeDenominator = hostFeeDenominator;
        this.curveType = curveType;
        this.payer = payer;
    }
    /**
     * Get the minimum balance for the token swap account to be rent exempt
     *
     * @return Number of lamports required
     */
    static async getMinBalanceRentForExemptTokenSwap(connection) {
        return await connection.getMinimumBalanceForRentExemption(TokenSwapLayout.span);
    }
    static createInitSwapInstruction(tokenSwapAccount, authority, nonce, tokenAccountA, tokenAccountB, tokenPool, feeAccount, tokenAccountPool, tokenProgramId, swapProgramId, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType, curveParameters = new BN(0)) {
        const keys = [
            { pubkey: tokenSwapAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: tokenAccountA, isSigner: false, isWritable: false },
            { pubkey: tokenAccountB, isSigner: false, isWritable: false },
            { pubkey: tokenPool, isSigner: false, isWritable: true },
            { pubkey: feeAccount, isSigner: false, isWritable: false },
            { pubkey: tokenAccountPool, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        const commandDataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            BufferLayout.u8("nonce"),
            BufferLayout.nu64("tradeFeeNumerator"),
            BufferLayout.nu64("tradeFeeDenominator"),
            BufferLayout.nu64("ownerTradeFeeNumerator"),
            BufferLayout.nu64("ownerTradeFeeDenominator"),
            BufferLayout.nu64("ownerWithdrawFeeNumerator"),
            BufferLayout.nu64("ownerWithdrawFeeDenominator"),
            BufferLayout.nu64("hostFeeNumerator"),
            BufferLayout.nu64("hostFeeDenominator"),
            BufferLayout.u8("curveType"),
            BufferLayout.blob(32, "curveParameters")
        ]);
        let data = Buffer.alloc(1024);
        // package curve parameters
        // NOTE: currently assume all curves take a single parameter, u64 int
        //       the remaining 24 of the 32 bytes available are filled with 0s
        const curveParamsBuffer = Buffer.alloc(32);
        curveParameters.toBuffer().copy(curveParamsBuffer);
        {
            const encodeLength = commandDataLayout.encode({
                instruction: 0,
                nonce,
                tradeFeeNumerator,
                tradeFeeDenominator,
                ownerTradeFeeNumerator,
                ownerTradeFeeDenominator,
                ownerWithdrawFeeNumerator,
                ownerWithdrawFeeDenominator,
                hostFeeNumerator,
                hostFeeDenominator,
                curveType,
                curveParameters: curveParamsBuffer
            }, data);
            data = data.slice(0, encodeLength);
        }
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
    static async loadAccount(connection, address, programId) {
        const accountInfo = await connection.getAccountInfo(address);
        if (accountInfo === null) {
            throw new Error("Failed to find account");
        }
        if (!accountInfo.owner.equals(programId)) {
            throw new Error(`Invalid owner: ${JSON.stringify(accountInfo.owner)}`);
        }
        return Buffer.from(accountInfo.data);
    }
    static async loadTokenSwap(connection, address, programId, payer) {
        const data = await this.loadAccount(connection, address, programId);
        const tokenSwapData = TokenSwapLayout.decode(data);
        if (!tokenSwapData.isInitialized) {
            throw new Error(`Invalid token swap state`);
        }
        const [authority, nonce] = await PublicKey.findProgramAddress([address.toBuffer()], programId);
        const poolToken = new PublicKey(tokenSwapData.tokenPool);
        const feeAccount = new PublicKey(tokenSwapData.feeAccount);
        const tokenAccountA = new PublicKey(tokenSwapData.tokenAccountA);
        const tokenAccountB = new PublicKey(tokenSwapData.tokenAccountB);
        const mintA = new PublicKey(tokenSwapData.mintA);
        const mintB = new PublicKey(tokenSwapData.mintB);
        const tokenProgramId = new PublicKey(tokenSwapData.tokenProgramId);
        const tradeFeeNumerator = new BN(tokenSwapData.tradeFeeNumerator);
        const tradeFeeDenominator = new BN(tokenSwapData.tradeFeeDenominator);
        const ownerTradeFeeNumerator = new BN(tokenSwapData.ownerTradeFeeNumerator);
        const ownerTradeFeeDenominator = new BN(tokenSwapData.ownerTradeFeeDenominator);
        const ownerWithdrawFeeNumerator = new BN(tokenSwapData.ownerWithdrawFeeNumerator);
        const ownerWithdrawFeeDenominator = new BN(tokenSwapData.ownerWithdrawFeeDenominator);
        const hostFeeNumerator = new BN(tokenSwapData.hostFeeNumerator);
        const hostFeeDenominator = new BN(tokenSwapData.hostFeeDenominator);
        const curveType = tokenSwapData.curveType;
        return new TokenSwap(connection, address, programId, tokenProgramId, poolToken, feeAccount, authority, nonce, tokenAccountA, tokenAccountB, mintA, mintB, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType, payer);
    }
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
    static async createTokenSwap(connection, payer, tokenSwapAccount, authority, nonce, tokenAccountA, tokenAccountB, poolToken, mintA, mintB, feeAccount, tokenAccountPool, swapProgramId, tokenProgramId, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType, curveParameters, confirmOptions) {
        const tokenSwap = new TokenSwap(connection, tokenSwapAccount.publicKey, swapProgramId, tokenProgramId, poolToken, feeAccount, authority, nonce, tokenAccountA, tokenAccountB, mintA, mintB, new BN(tradeFeeNumerator), new BN(tradeFeeDenominator), new BN(ownerTradeFeeNumerator), new BN(ownerTradeFeeDenominator), new BN(ownerWithdrawFeeNumerator), new BN(ownerWithdrawFeeDenominator), new BN(hostFeeNumerator), new BN(hostFeeDenominator), curveType, payer);
        // Allocate memory for the account
        const balanceNeeded = await TokenSwap.getMinBalanceRentForExemptTokenSwap(connection);
        const transaction = new Transaction();
        transaction.add(SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: tokenSwapAccount.publicKey,
            lamports: balanceNeeded,
            space: TokenSwapLayout.span,
            programId: swapProgramId
        }));
        const instruction = TokenSwap.createInitSwapInstruction(tokenSwapAccount, authority, nonce, tokenAccountA, tokenAccountB, poolToken, feeAccount, tokenAccountPool, tokenProgramId, swapProgramId, tradeFeeNumerator, tradeFeeDenominator, ownerTradeFeeNumerator, ownerTradeFeeDenominator, ownerWithdrawFeeNumerator, ownerWithdrawFeeDenominator, hostFeeNumerator, hostFeeDenominator, curveType, curveParameters);
        transaction.add(instruction);
        await sendAndConfirmTransaction(connection, transaction, [payer, tokenSwapAccount], confirmOptions);
        return tokenSwap;
    }
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
    async swap(userSource, poolSource, poolDestination, userDestination, hostFeeAccount, userTransferAuthority, amountIn, minimumAmountOut, confirmOptions) {
        return await sendAndConfirmTransaction(this.connection, new Transaction().add(TokenSwap.swapInstruction(this.tokenSwap, this.authority, userTransferAuthority.publicKey, userSource, poolSource, poolDestination, userDestination, this.poolToken, this.feeAccount, hostFeeAccount, this.swapProgramId, this.tokenProgramId, amountIn, minimumAmountOut)), [this.payer, userTransferAuthority], confirmOptions);
    }
    static swapInstruction(tokenSwap, authority, userTransferAuthority, userSource, poolSource, poolDestination, userDestination, poolMint, feeAccount, hostFeeAccount, swapProgramId, tokenProgramId, amountIn, minimumAmountOut) {
        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            Layout.u64("amountIn"),
            Layout.u64("minimumAmountOut")
        ]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 1,
            amountIn: amountIn,
            minimumAmountOut: minimumAmountOut
        }, data);
        const keys = [
            { pubkey: tokenSwap, isSigner: false, isWritable: false },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
            { pubkey: userSource, isSigner: false, isWritable: true },
            { pubkey: poolSource, isSigner: false, isWritable: true },
            { pubkey: poolDestination, isSigner: false, isWritable: true },
            { pubkey: userDestination, isSigner: false, isWritable: true },
            { pubkey: poolMint, isSigner: false, isWritable: true },
            { pubkey: feeAccount, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        if (hostFeeAccount !== null) {
            keys.push({ pubkey: hostFeeAccount, isSigner: false, isWritable: true });
        }
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
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
    async depositAllTokenTypes(userAccountA, userAccountB, poolAccount, userTransferAuthority, poolTokenAmount, maximumTokenA, maximumTokenB, confirmOptions) {
        return await sendAndConfirmTransaction(this.connection, new Transaction().add(TokenSwap.depositAllTokenTypesInstruction(this.tokenSwap, this.authority, userTransferAuthority.publicKey, userAccountA, userAccountB, this.tokenAccountA, this.tokenAccountB, this.poolToken, poolAccount, this.swapProgramId, this.tokenProgramId, poolTokenAmount, maximumTokenA, maximumTokenB)), [this.payer, userTransferAuthority], confirmOptions);
    }
    static depositAllTokenTypesInstruction(tokenSwap, authority, userTransferAuthority, sourceA, sourceB, intoA, intoB, poolToken, poolAccount, swapProgramId, tokenProgramId, poolTokenAmount, maximumTokenA, maximumTokenB) {
        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            Layout.u64("poolTokenAmount"),
            Layout.u64("maximumTokenA"),
            Layout.u64("maximumTokenB")
        ]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 2,
            poolTokenAmount: poolTokenAmount,
            maximumTokenA: maximumTokenA,
            maximumTokenB: maximumTokenB
        }, data);
        const keys = [
            { pubkey: tokenSwap, isSigner: false, isWritable: false },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
            { pubkey: sourceA, isSigner: false, isWritable: true },
            { pubkey: sourceB, isSigner: false, isWritable: true },
            { pubkey: intoA, isSigner: false, isWritable: true },
            { pubkey: intoB, isSigner: false, isWritable: true },
            { pubkey: poolToken, isSigner: false, isWritable: true },
            { pubkey: poolAccount, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
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
    async withdrawAllTokenTypes(userAccountA, userAccountB, poolAccount, userTransferAuthority, poolTokenAmount, minimumTokenA, minimumTokenB, confirmOptions) {
        return await sendAndConfirmTransaction(this.connection, new Transaction().add(TokenSwap.withdrawAllTokenTypesInstruction(this.tokenSwap, this.authority, userTransferAuthority.publicKey, this.poolToken, this.feeAccount, poolAccount, this.tokenAccountA, this.tokenAccountB, userAccountA, userAccountB, this.swapProgramId, this.tokenProgramId, poolTokenAmount, minimumTokenA, minimumTokenB)), [this.payer, userTransferAuthority], confirmOptions);
    }
    static withdrawAllTokenTypesInstruction(tokenSwap, authority, userTransferAuthority, poolMint, feeAccount, sourcePoolAccount, fromA, fromB, userAccountA, userAccountB, swapProgramId, tokenProgramId, poolTokenAmount, minimumTokenA, minimumTokenB) {
        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            Layout.u64("poolTokenAmount"),
            Layout.u64("minimumTokenA"),
            Layout.u64("minimumTokenB")
        ]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 3,
            poolTokenAmount: poolTokenAmount,
            minimumTokenA: minimumTokenA,
            minimumTokenB: minimumTokenB
        }, data);
        const keys = [
            { pubkey: tokenSwap, isSigner: false, isWritable: false },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
            { pubkey: poolMint, isSigner: false, isWritable: true },
            { pubkey: sourcePoolAccount, isSigner: false, isWritable: true },
            { pubkey: fromA, isSigner: false, isWritable: true },
            { pubkey: fromB, isSigner: false, isWritable: true },
            { pubkey: userAccountA, isSigner: false, isWritable: true },
            { pubkey: userAccountB, isSigner: false, isWritable: true },
            { pubkey: feeAccount, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
    /**
     * Deposit one side of tokens into the pool
     * @param userAccount User account to deposit token A or B
     * @param poolAccount User account to receive pool tokens
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param sourceTokenAmount The amount of token A or B to deposit
     * @param minimumPoolTokenAmount Minimum amount of pool tokens to mint
     */
    async depositSingleTokenTypeExactAmountIn(userAccount, poolAccount, userTransferAuthority, sourceTokenAmount, minimumPoolTokenAmount, confirmOptions) {
        return await sendAndConfirmTransaction(this.connection, new Transaction().add(TokenSwap.depositSingleTokenTypeExactAmountInInstruction(this.tokenSwap, this.authority, userTransferAuthority.publicKey, userAccount, this.tokenAccountA, this.tokenAccountB, this.poolToken, poolAccount, this.swapProgramId, this.tokenProgramId, sourceTokenAmount, minimumPoolTokenAmount)), [this.payer, userTransferAuthority], confirmOptions);
    }
    static depositSingleTokenTypeExactAmountInInstruction(tokenSwap, authority, userTransferAuthority, source, intoA, intoB, poolToken, poolAccount, swapProgramId, tokenProgramId, sourceTokenAmount, minimumPoolTokenAmount) {
        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            Layout.u64("sourceTokenAmount"),
            Layout.u64("minimumPoolTokenAmount")
        ]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 4,
            sourceTokenAmount: sourceTokenAmount,
            minimumPoolTokenAmount: minimumPoolTokenAmount
        }, data);
        const keys = [
            { pubkey: tokenSwap, isSigner: false, isWritable: false },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
            { pubkey: source, isSigner: false, isWritable: true },
            { pubkey: intoA, isSigner: false, isWritable: true },
            { pubkey: intoB, isSigner: false, isWritable: true },
            { pubkey: poolToken, isSigner: false, isWritable: true },
            { pubkey: poolAccount, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
    /**
     * Withdraw tokens from the pool
     *
     * @param userAccount User account to receive token A or B
     * @param poolAccount User account to burn pool token
     * @param userTransferAuthority Account delegated to transfer user's tokens
     * @param destinationTokenAmount The amount of token A or B to withdraw
     * @param maximumPoolTokenAmount Maximum amount of pool tokens to burn
     */
    async withdrawSingleTokenTypeExactAmountOut(userAccount, poolAccount, userTransferAuthority, destinationTokenAmount, maximumPoolTokenAmount, confirmOptions) {
        return await sendAndConfirmTransaction(this.connection, new Transaction().add(TokenSwap.withdrawSingleTokenTypeExactAmountOutInstruction(this.tokenSwap, this.authority, userTransferAuthority.publicKey, this.poolToken, this.feeAccount, poolAccount, this.tokenAccountA, this.tokenAccountB, userAccount, this.swapProgramId, this.tokenProgramId, destinationTokenAmount, maximumPoolTokenAmount)), [this.payer, userTransferAuthority], confirmOptions);
    }
    static withdrawSingleTokenTypeExactAmountOutInstruction(tokenSwap, authority, userTransferAuthority, poolMint, feeAccount, sourcePoolAccount, fromA, fromB, userAccount, swapProgramId, tokenProgramId, destinationTokenAmount, maximumPoolTokenAmount) {
        const dataLayout = BufferLayout.struct([
            BufferLayout.u8("instruction"),
            u64("destinationTokenAmount"),
            u64("maximumPoolTokenAmount")
        ]);
        const data = Buffer.alloc(dataLayout.span);
        dataLayout.encode({
            instruction: 5,
            destinationTokenAmount: destinationTokenAmount,
            maximumPoolTokenAmount: maximumPoolTokenAmount
        }, data);
        const keys = [
            { pubkey: tokenSwap, isSigner: false, isWritable: false },
            { pubkey: authority, isSigner: false, isWritable: false },
            { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
            { pubkey: poolMint, isSigner: false, isWritable: true },
            { pubkey: sourcePoolAccount, isSigner: false, isWritable: true },
            { pubkey: fromA, isSigner: false, isWritable: true },
            { pubkey: fromB, isSigner: false, isWritable: true },
            { pubkey: userAccount, isSigner: false, isWritable: true },
            { pubkey: feeAccount, isSigner: false, isWritable: true },
            { pubkey: tokenProgramId, isSigner: false, isWritable: false }
        ];
        return new TransactionInstruction({
            keys,
            programId: swapProgramId,
            data
        });
    }
}
//# sourceMappingURL=index.js.map