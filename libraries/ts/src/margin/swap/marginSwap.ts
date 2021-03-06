import assert from "assert"
import * as BufferLayout from "@solana/buffer-layout"
import { BN } from "@project-serum/anchor"
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  MintLayout,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import {
  Account,
  Connection,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction
} from "@solana/web3.js"

import { TokenSwap } from "./index"

export class MarginSwap {
  constructor(public tokenSwap: TokenSwap) {
    assert(tokenSwap)
  }

  static async load(
    connection: Connection,
    tokenSwapAddress: PublicKey,
    payer: Account,
    splTokenSwapProgramId: PublicKey
  ) {
    const tokenSwap = await TokenSwap.loadTokenSwap(connection, tokenSwapAddress, splTokenSwapProgramId, payer)
    return new MarginSwap(tokenSwap)
  }

  static async create(
    connection: Connection,
    payer: Account,
    tokenSwapAccount: Account,
    authority: PublicKey,
    authorityNonce: number,
    tokenAccountA: PublicKey,
    tokenAccountB: PublicKey,
    tokenPool: PublicKey,
    mintA: PublicKey,
    mintB: PublicKey,
    feeAccount: PublicKey,
    tokenAccountPool: PublicKey,
    splTokenSwapProgramId: PublicKey,
    tradeFeeNumerator: number,
    tradeFeeDenominator: number,
    ownerTradeFeeNumerator: number,
    ownerTradeFeeDenominator: number,
    ownerWithdrawFeeNumerator: number,
    ownerWithdrawFeeDenominator: number,
    hostFeeNumerator: number,
    hostFeeDenominator: number,
    curveType: number
  ): Promise<TokenSwap> {
    const tokenSwap: TokenSwap = await TokenSwap.createTokenSwap(
      connection,
      payer,
      tokenSwapAccount,
      authority,
      authorityNonce,
      tokenAccountA,
      tokenAccountB,
      tokenPool,
      mintA,
      mintB,
      feeAccount,
      tokenAccountPool,
      splTokenSwapProgramId,
      TOKEN_PROGRAM_ID,
      tradeFeeNumerator,
      tradeFeeDenominator,
      ownerTradeFeeNumerator,
      ownerTradeFeeDenominator,
      ownerWithdrawFeeNumerator,
      ownerWithdrawFeeDenominator,
      hostFeeNumerator,
      hostFeeDenominator,
      curveType
    )

    return tokenSwap
  }

  async approve(
    connection: Connection,
    account: PublicKey,
    delegate: PublicKey,
    owner: Account,
    amount: BN,
    payer: Account
  ): Promise<void> {
    const tx: Transaction = new Transaction()
    tx.add(this.createApproveInstruction(TOKEN_PROGRAM_ID, account, delegate, owner.publicKey, amount))
    await sendAndConfirmTransaction(connection, tx, [payer, owner])
  }

  createApproveInstruction(
    programId: PublicKey,
    account: PublicKey,
    delegate: PublicKey,
    owner: PublicKey,
    amount: BN
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct<any>([BufferLayout.u8("instruction"), BufferLayout.blob(8, "amount")])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 4, // Approve instruction
        amount: new BN(amount).toArrayLike(Buffer, "le", 8)
      },
      data
    )

    const keys = [
      { pubkey: account, isSigner: false, isWritable: true },
      { pubkey: delegate, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: true, isWritable: false }
    ]
    return new TransactionInstruction({
      keys,
      programId: programId,
      data
    })
  }

  static async createAssociatedTokenAccount(
    connection: Connection,
    payer: Signer,
    mint: PublicKey,
    owner: PublicKey
  ): Promise<PublicKey> {
    const associatedToken = await getAssociatedTokenAddress(mint, owner, true)

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        associatedToken,
        owner,
        mint,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      )
    )

    await sendAndConfirmTransaction(connection, transaction, [payer], {
      skipPreflight: true
    })

    return associatedToken
  }

  createInitAccountInstruction(
    programId: PublicKey,
    mint: PublicKey,
    account: PublicKey,
    owner: PublicKey
  ): TransactionInstruction {
    const keys = [
      { pubkey: account, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ]
    const dataLayout = BufferLayout.struct<any>([BufferLayout.u8("instruction")])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1 // InitializeAccount instruction
      },
      data
    )

    return new TransactionInstruction({
      keys,
      programId,
      data
    })
  }

  static async getMintInfo(connection: Connection, mint: PublicKey) {
    const info = await connection.getAccountInfo(mint)
    if (info === null) {
      throw new Error("Failed to find mint account")
    }
    if (!info.owner.equals(TOKEN_PROGRAM_ID)) {
      throw new Error(`Invalid mint owner: ${JSON.stringify(info.owner)}`)
    }
    if (info.data.length != MintLayout.span) {
      throw new Error(`Invalid mint size`)
    }
    const data = Buffer.from(info.data)
    return MintLayout.decode(data)
  }
}
