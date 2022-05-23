import * as BufferLayout from "@solana/buffer-layout";
import BN from "bn.js";
import {
  Account,
  ConfirmOptions,
  Connection,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
} from "@solana/web3.js";

import * as Layout from "../../utils/layout";
import { u64 } from "../../utils/layout";

export const TokenSwapLayout = BufferLayout.struct<any>([
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
  BufferLayout.blob(32, "curveParameters"),
]);

export const CurveType = Object.freeze({
  ConstantProduct: 0, // Constant product curve, Uniswap-style
  ConstantPrice: 1, // Constant price curve, always X amount of A token for 1 B token, where X is defined at init
  Offset: 3, // Offset curve, like Uniswap, but with an additional offset on the token B side
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
  constructor(
    private connection: Connection,
    public tokenSwap: PublicKey,
    public swapProgramId: PublicKey,
    public tokenProgramId: PublicKey,
    public poolToken: PublicKey,
    public feeAccount: PublicKey,
    public authority: PublicKey,
    public tokenAccountA: PublicKey,
    public tokenAccountB: PublicKey,
    public mintA: PublicKey,
    public mintB: PublicKey,
    public tradeFeeNumerator: BN,
    public tradeFeeDenominator: BN,
    public ownerTradeFeeNumerator: BN,
    public ownerTradeFeeDenominator: BN,
    public ownerWithdrawFeeNumerator: BN,
    public ownerWithdrawFeeDenominator: BN,
    public hostFeeNumerator: BN,
    public hostFeeDenominator: BN,
    public curveType: number,
    public payer: Account
  ) {
    this.connection = connection;
    this.tokenSwap = tokenSwap;
    this.swapProgramId = swapProgramId;
    this.tokenProgramId = tokenProgramId;
    this.poolToken = poolToken;
    this.feeAccount = feeAccount;
    this.authority = authority;
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
  static async getMinBalanceRentForExemptTokenSwap(
    connection: Connection
  ): Promise<number> {
    return await connection.getMinimumBalanceForRentExemption(
      TokenSwapLayout.span
    );
  }

  static createInitSwapInstruction(
    tokenSwapAccount: Account,
    authority: PublicKey,
    tokenAccountA: PublicKey,
    tokenAccountB: PublicKey,
    tokenPool: PublicKey,
    feeAccount: PublicKey,
    tokenAccountPool: PublicKey,
    tokenProgramId: PublicKey,
    swapProgramId: PublicKey,
    tradeFeeNumerator: number,
    tradeFeeDenominator: number,
    ownerTradeFeeNumerator: number,
    ownerTradeFeeDenominator: number,
    ownerWithdrawFeeNumerator: number,
    ownerWithdrawFeeDenominator: number,
    hostFeeNumerator: number,
    hostFeeDenominator: number,
    curveType: number,
    curveParameters: BN = new BN(0)
  ): TransactionInstruction {
    const keys = [
      { pubkey: tokenSwapAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: false, isWritable: false },
      { pubkey: tokenAccountA, isSigner: false, isWritable: false },
      { pubkey: tokenAccountB, isSigner: false, isWritable: false },
      { pubkey: tokenPool, isSigner: false, isWritable: true },
      { pubkey: feeAccount, isSigner: false, isWritable: false },
      { pubkey: tokenAccountPool, isSigner: false, isWritable: true },
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    const commandDataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      BufferLayout.nu64("tradeFeeNumerator"),
      BufferLayout.nu64("tradeFeeDenominator"),
      BufferLayout.nu64("ownerTradeFeeNumerator"),
      BufferLayout.nu64("ownerTradeFeeDenominator"),
      BufferLayout.nu64("ownerWithdrawFeeNumerator"),
      BufferLayout.nu64("ownerWithdrawFeeDenominator"),
      BufferLayout.nu64("hostFeeNumerator"),
      BufferLayout.nu64("hostFeeDenominator"),
      BufferLayout.u8("curveType"),
      BufferLayout.blob(32, "curveParameters"),
    ]);
    let data = Buffer.alloc(1024);

    // package curve parameters
    // NOTE: currently assume all curves take a single parameter, u64 int
    //       the remaining 24 of the 32 bytes available are filled with 0s
    let curveParamsBuffer = Buffer.alloc(32);
    curveParameters.toBuffer().copy(curveParamsBuffer);

    {
      const encodeLength = commandDataLayout.encode(
        {
          instruction: 0, // InitializeSwap instruction
          tradeFeeNumerator,
          tradeFeeDenominator,
          ownerTradeFeeNumerator,
          ownerTradeFeeDenominator,
          ownerWithdrawFeeNumerator,
          ownerWithdrawFeeDenominator,
          hostFeeNumerator,
          hostFeeDenominator,
          curveType,
          curveParameters: curveParamsBuffer,
        },
        data
      );
      data = data.slice(0, encodeLength);
    }
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
    });
  }

  static async loadAccount(
    connection: Connection,
    address: PublicKey,
    programId: PublicKey
  ): Promise<Buffer> {
    const accountInfo = await connection.getAccountInfo(address);
    if (accountInfo === null) {
      throw new Error("Failed to find account");
    }

    if (!accountInfo.owner.equals(programId)) {
      throw new Error(`Invalid owner: ${JSON.stringify(accountInfo.owner)}`);
    }

    return Buffer.from(accountInfo.data);
  }

  static async loadTokenSwap(
    connection: Connection,
    address: PublicKey,
    programId: PublicKey,
    payer: Account
  ): Promise<TokenSwap> {
    const data = await this.loadAccount(connection, address, programId);
    const tokenSwapData = TokenSwapLayout.decode(data);
    if (!tokenSwapData.isInitialized) {
      throw new Error(`Invalid token swap state`);
    }

    const [authority] = await PublicKey.findProgramAddress(
      [address.toBuffer()],
      programId
    );

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
    const ownerTradeFeeDenominator = new BN(
      tokenSwapData.ownerTradeFeeDenominator
    );
    const ownerWithdrawFeeNumerator = new BN(
      tokenSwapData.ownerWithdrawFeeNumerator
    );
    const ownerWithdrawFeeDenominator = new BN(
      tokenSwapData.ownerWithdrawFeeDenominator
    );
    const hostFeeNumerator = new BN(tokenSwapData.hostFeeNumerator);
    const hostFeeDenominator = new BN(tokenSwapData.hostFeeDenominator);
    const curveType = tokenSwapData.curveType;

    return new TokenSwap(
      connection,
      address,
      programId,
      tokenProgramId,
      poolToken,
      feeAccount,
      authority,
      tokenAccountA,
      tokenAccountB,
      mintA,
      mintB,
      tradeFeeNumerator,
      tradeFeeDenominator,
      ownerTradeFeeNumerator,
      ownerTradeFeeDenominator,
      ownerWithdrawFeeNumerator,
      ownerWithdrawFeeDenominator,
      hostFeeNumerator,
      hostFeeDenominator,
      curveType,
      payer
    );
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
  static async createTokenSwap(
    connection: Connection,
    payer: Account,
    tokenSwapAccount: Account,
    authority: PublicKey,
    tokenAccountA: PublicKey,
    tokenAccountB: PublicKey,
    poolToken: PublicKey,
    mintA: PublicKey,
    mintB: PublicKey,
    feeAccount: PublicKey,
    tokenAccountPool: PublicKey,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    tradeFeeNumerator: number,
    tradeFeeDenominator: number,
    ownerTradeFeeNumerator: number,
    ownerTradeFeeDenominator: number,
    ownerWithdrawFeeNumerator: number,
    ownerWithdrawFeeDenominator: number,
    hostFeeNumerator: number,
    hostFeeDenominator: number,
    curveType: number,
    curveParameters?: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TokenSwap> {
    let transaction;
    const tokenSwap = new TokenSwap(
      connection,
      tokenSwapAccount.publicKey,
      swapProgramId,
      tokenProgramId,
      poolToken,
      feeAccount,
      authority,
      tokenAccountA,
      tokenAccountB,
      mintA,
      mintB,
      new BN(tradeFeeNumerator),
      new BN(tradeFeeDenominator),
      new BN(ownerTradeFeeNumerator),
      new BN(ownerTradeFeeDenominator),
      new BN(ownerWithdrawFeeNumerator),
      new BN(ownerWithdrawFeeDenominator),
      new BN(hostFeeNumerator),
      new BN(hostFeeDenominator),
      curveType,
      payer
    );

    // Allocate memory for the account
    const balanceNeeded = await TokenSwap.getMinBalanceRentForExemptTokenSwap(
      connection
    );
    transaction = new Transaction();
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: tokenSwapAccount.publicKey,
        lamports: balanceNeeded,
        space: TokenSwapLayout.span,
        programId: swapProgramId,
      })
    );

    const instruction = TokenSwap.createInitSwapInstruction(
      tokenSwapAccount,
      authority,
      tokenAccountA,
      tokenAccountB,
      poolToken,
      feeAccount,
      tokenAccountPool,
      tokenProgramId,
      swapProgramId,
      tradeFeeNumerator,
      tradeFeeDenominator,
      ownerTradeFeeNumerator,
      ownerTradeFeeDenominator,
      ownerWithdrawFeeNumerator,
      ownerWithdrawFeeDenominator,
      hostFeeNumerator,
      hostFeeDenominator,
      curveType,
      curveParameters
    );

    transaction.add(instruction);
    await sendAndConfirmTransaction(
      connection,
      transaction,
      [payer, tokenSwapAccount],
      confirmOptions
    );

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
  async swap(
    userSource: PublicKey,
    poolSource: PublicKey,
    poolDestination: PublicKey,
    userDestination: PublicKey,
    hostFeeAccount: PublicKey | null,
    userTransferAuthority: Account,
    amountIn: BN,
    minimumAmountOut: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionSignature> {
    return await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(
        TokenSwap.swapInstruction(
          this.tokenSwap,
          this.authority,
          userTransferAuthority.publicKey,
          userSource,
          poolSource,
          poolDestination,
          userDestination,
          this.poolToken,
          this.feeAccount,
          hostFeeAccount,
          this.swapProgramId,
          this.tokenProgramId,
          amountIn,
          minimumAmountOut
        )
      ),
      [this.payer, userTransferAuthority],
      confirmOptions
    );
  }

  static swapInstruction(
    tokenSwap: PublicKey,
    authority: PublicKey,
    userTransferAuthority: PublicKey,
    userSource: PublicKey,
    poolSource: PublicKey,
    poolDestination: PublicKey,
    userDestination: PublicKey,
    poolMint: PublicKey,
    feeAccount: PublicKey,
    hostFeeAccount: PublicKey | null,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    amountIn: BN,
    minimumAmountOut: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      Layout.u64("amountIn"),
      Layout.u64("minimumAmountOut"),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 1, // Swap instruction
        amountIn: amountIn,
        minimumAmountOut: minimumAmountOut,
      },
      data
    );

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
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    if (hostFeeAccount !== null) {
      keys.push({ pubkey: hostFeeAccount, isSigner: false, isWritable: true });
    }
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
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
  async depositAllTokenTypes(
    userAccountA: PublicKey,
    userAccountB: PublicKey,
    poolAccount: PublicKey,
    userTransferAuthority: Account,
    poolTokenAmount: BN,
    maximumTokenA: BN,
    maximumTokenB: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionSignature> {
    return await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(
        TokenSwap.depositAllTokenTypesInstruction(
          this.tokenSwap,
          this.authority,
          userTransferAuthority.publicKey,
          userAccountA,
          userAccountB,
          this.tokenAccountA,
          this.tokenAccountB,
          this.poolToken,
          poolAccount,
          this.swapProgramId,
          this.tokenProgramId,
          poolTokenAmount,
          maximumTokenA,
          maximumTokenB
        )
      ),
      [this.payer, userTransferAuthority],
      confirmOptions
    );
  }

  static depositAllTokenTypesInstruction(
    tokenSwap: PublicKey,
    authority: PublicKey,
    userTransferAuthority: PublicKey,
    sourceA: PublicKey,
    sourceB: PublicKey,
    intoA: PublicKey,
    intoB: PublicKey,
    poolToken: PublicKey,
    poolAccount: PublicKey,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    poolTokenAmount: BN,
    maximumTokenA: BN,
    maximumTokenB: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      Layout.u64("poolTokenAmount"),
      Layout.u64("maximumTokenA"),
      Layout.u64("maximumTokenB"),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 2, // Deposit instruction
        poolTokenAmount: poolTokenAmount,
        maximumTokenA: maximumTokenA,
        maximumTokenB: maximumTokenB,
      },
      data
    );

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
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
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
  async withdrawAllTokenTypes(
    userAccountA: PublicKey,
    userAccountB: PublicKey,
    poolAccount: PublicKey,
    userTransferAuthority: Account,
    poolTokenAmount: BN,
    minimumTokenA: BN,
    minimumTokenB: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionSignature> {
    return await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(
        TokenSwap.withdrawAllTokenTypesInstruction(
          this.tokenSwap,
          this.authority,
          userTransferAuthority.publicKey,
          this.poolToken,
          this.feeAccount,
          poolAccount,
          this.tokenAccountA,
          this.tokenAccountB,
          userAccountA,
          userAccountB,
          this.swapProgramId,
          this.tokenProgramId,
          poolTokenAmount,
          minimumTokenA,
          minimumTokenB
        )
      ),
      [this.payer, userTransferAuthority],
      confirmOptions
    );
  }

  static withdrawAllTokenTypesInstruction(
    tokenSwap: PublicKey,
    authority: PublicKey,
    userTransferAuthority: PublicKey,
    poolMint: PublicKey,
    feeAccount: PublicKey,
    sourcePoolAccount: PublicKey,
    fromA: PublicKey,
    fromB: PublicKey,
    userAccountA: PublicKey,
    userAccountB: PublicKey,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    poolTokenAmount: BN,
    minimumTokenA: BN,
    minimumTokenB: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      Layout.u64("poolTokenAmount"),
      Layout.u64("minimumTokenA"),
      Layout.u64("minimumTokenB"),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 3, // Withdraw instruction
        poolTokenAmount: poolTokenAmount,
        minimumTokenA: minimumTokenA,
        minimumTokenB: minimumTokenB,
      },
      data
    );

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
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
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
  async depositSingleTokenTypeExactAmountIn(
    userAccount: PublicKey,
    poolAccount: PublicKey,
    userTransferAuthority: Account,
    sourceTokenAmount: BN,
    minimumPoolTokenAmount: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionSignature> {
    return await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(
        TokenSwap.depositSingleTokenTypeExactAmountInInstruction(
          this.tokenSwap,
          this.authority,
          userTransferAuthority.publicKey,
          userAccount,
          this.tokenAccountA,
          this.tokenAccountB,
          this.poolToken,
          poolAccount,
          this.swapProgramId,
          this.tokenProgramId,
          sourceTokenAmount,
          minimumPoolTokenAmount
        )
      ),
      [this.payer, userTransferAuthority],
      confirmOptions
    );
  }

  static depositSingleTokenTypeExactAmountInInstruction(
    tokenSwap: PublicKey,
    authority: PublicKey,
    userTransferAuthority: PublicKey,
    source: PublicKey,
    intoA: PublicKey,
    intoB: PublicKey,
    poolToken: PublicKey,
    poolAccount: PublicKey,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    sourceTokenAmount: BN,
    minimumPoolTokenAmount: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      Layout.u64("sourceTokenAmount"),
      Layout.u64("minimumPoolTokenAmount"),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 4, // depositSingleTokenTypeExactAmountIn instruction
        sourceTokenAmount: sourceTokenAmount,
        minimumPoolTokenAmount: minimumPoolTokenAmount,
      },
      data
    );

    const keys = [
      { pubkey: tokenSwap, isSigner: false, isWritable: false },
      { pubkey: authority, isSigner: false, isWritable: false },
      { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: intoA, isSigner: false, isWritable: true },
      { pubkey: intoB, isSigner: false, isWritable: true },
      { pubkey: poolToken, isSigner: false, isWritable: true },
      { pubkey: poolAccount, isSigner: false, isWritable: true },
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
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
  async withdrawSingleTokenTypeExactAmountOut(
    userAccount: PublicKey,
    poolAccount: PublicKey,
    userTransferAuthority: Account,
    destinationTokenAmount: BN,
    maximumPoolTokenAmount: BN,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionSignature> {
    return await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(
        TokenSwap.withdrawSingleTokenTypeExactAmountOutInstruction(
          this.tokenSwap,
          this.authority,
          userTransferAuthority.publicKey,
          this.poolToken,
          this.feeAccount,
          poolAccount,
          this.tokenAccountA,
          this.tokenAccountB,
          userAccount,
          this.swapProgramId,
          this.tokenProgramId,
          destinationTokenAmount,
          maximumPoolTokenAmount
        )
      ),
      [this.payer, userTransferAuthority],
      confirmOptions
    );
  }

  static withdrawSingleTokenTypeExactAmountOutInstruction(
    tokenSwap: PublicKey,
    authority: PublicKey,
    userTransferAuthority: PublicKey,
    poolMint: PublicKey,
    feeAccount: PublicKey,
    sourcePoolAccount: PublicKey,
    fromA: PublicKey,
    fromB: PublicKey,
    userAccount: PublicKey,
    swapProgramId: PublicKey,
    tokenProgramId: PublicKey,
    destinationTokenAmount: BN,
    maximumPoolTokenAmount: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([
      BufferLayout.u8("instruction"),
      u64("destinationTokenAmount"),
      u64("maximumPoolTokenAmount"),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 5, // withdrawSingleTokenTypeExactAmountOut instruction
        destinationTokenAmount: destinationTokenAmount,
        maximumPoolTokenAmount: maximumPoolTokenAmount,
      },
      data
    );

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
      { pubkey: tokenProgramId, isSigner: false, isWritable: false },
    ];
    return new TransactionInstruction({
      keys,
      programId: swapProgramId,
      data,
    });
  }
}
