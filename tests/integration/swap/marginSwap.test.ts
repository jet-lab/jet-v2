import assert from "assert"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider, BN } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import {
  AccountLayout,
  approve,
  createAccount,
  createMint,
  mintTo,
  RawAccount,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import {
  Account,
  ConfirmOptions,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction
} from "@solana/web3.js"

import MARGIN_CONFIG from "../../../libraries/ts/src/margin/config.json"

import { TokenSwap, CurveType, MarginSwap } from "../../../libraries/ts/src"
import { sleep } from "../../../libraries/ts/src/utils/util"

import { getTokenAccountInfo } from "../util"

describe("margin swap", () => {
  const controlProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.controlProgramId)
  const marginProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.marginProgramId)
  const marginSwapProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.marginSwapProgramId)
  const metadataProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.metadataProgramId)
  const splTokenSwapProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.splTokenSwapProgramId)

  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }

  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)

  const payer: Keypair = (provider.wallet as NodeWallet).payer

  const user = new Account()

  it("Fund payer", async () => {
    let airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)

    airdropSignature = await provider.connection.requestAirdrop(user.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  let tokenSwapAccount

  // authority of the token and accounts
  let authority: PublicKey

  let tokenPool: PublicKey

  let tokenAccountPool: PublicKey

  let feeTokenAccount: PublicKey

  it("Create token swap pool", async () => {
    tokenSwapAccount = new Account()
    ;[authority] = await PublicKey.findProgramAddress([tokenSwapAccount.publicKey.toBuffer()], splTokenSwapProgramId)

    tokenPool = await createMint(provider.connection, payer, authority, null, 2)

    tokenAccountPool = await MarginSwap.createAssociatedTokenAccount(
      provider.connection,
      payer,
      tokenPool,
      user.publicKey
    )

    feeTokenAccount = await createAccount(
      provider.connection,
      payer,
      tokenPool,
      user.publicKey, //orcaFeeOwner,
      Keypair.generate()
    )
  })

  let mintA: PublicKey
  let mintB: PublicKey
  let tokenAccountA: PublicKey
  let tokenAccountB: PublicKey

  it("create and mint tokens", async () => {
    mintA = await createMint(provider.connection, payer, user.publicKey, null, 2)

    tokenAccountA = await MarginSwap.createAssociatedTokenAccount(provider.connection, payer, mintA, authority)
    await mintTo(provider.connection, payer, mintA, tokenAccountA, user, 1000000)

    mintB = await createMint(provider.connection, payer, user.publicKey, null, 2)

    tokenAccountB = await MarginSwap.createAssociatedTokenAccount(provider.connection, payer, mintB, authority)
    await mintTo(provider.connection, payer, mintB, tokenAccountB, user, 1000000)
  })

  let marginSwap: MarginSwap

  it("createTokenSwap (constant product)", async () => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const tokenSwap: TokenSwap = await MarginSwap.create(
      provider.connection,
      new Account(payer.secretKey),
      tokenSwapAccount,
      authority,
      tokenAccountA,
      tokenAccountB,
      tokenPool,
      mintA,
      mintB,
      feeTokenAccount,
      tokenAccountPool,
      splTokenSwapProgramId,
      25,
      10000,
      5,
      10000,
      1,
      6,
      20,
      100,
      CurveType.ConstantProduct
    )

    marginSwap = await MarginSwap.load(
      provider.connection,
      tokenSwapAccount.publicKey,
      new Account(payer.secretKey),
      controlProgramId,
      marginProgramId,
      marginSwapProgramId,
      metadataProgramId,
      splTokenSwapProgramId
    )

    assert(marginSwap.tokenSwap.tokenProgramId.equals(TOKEN_PROGRAM_ID))
    assert(marginSwap.tokenSwap.tokenAccountA.equals(tokenAccountA))
    assert(marginSwap.tokenSwap.tokenAccountB.equals(tokenAccountB))
    assert(marginSwap.tokenSwap.mintA.equals(mintA))
    assert(marginSwap.tokenSwap.mintB.equals(mintB))
    assert(25 == marginSwap.tokenSwap.tradeFeeNumerator.toNumber())
    assert(10000 == marginSwap.tokenSwap.tradeFeeDenominator.toNumber())
    assert(5 == marginSwap.tokenSwap.ownerTradeFeeNumerator.toNumber())
    assert(10000 == marginSwap.tokenSwap.ownerTradeFeeDenominator.toNumber())
    assert(1 == marginSwap.tokenSwap.ownerWithdrawFeeNumerator.toNumber())
    assert(6 == marginSwap.tokenSwap.ownerWithdrawFeeDenominator.toNumber())
    assert(20 == marginSwap.tokenSwap.hostFeeNumerator.toNumber())
    assert(100 == marginSwap.tokenSwap.hostFeeDenominator.toNumber())
    assert(CurveType.ConstantProduct == marginSwap.tokenSwap.curveType)
  })

  let currentSwapTokenA = 1000000
  let currentSwapTokenB = 1000000
  let currentFeeAmount = 0

  it("deposit all token types", async () => {
    const poolMintInfo = await MarginSwap.getMintInfo(provider.connection, marginSwap.tokenSwap.poolToken)
    const supply = Number(poolMintInfo.supply)
    const swapTokenA = await getTokenAccountInfo(provider, tokenAccountA)
    const tokenA = Math.floor((Number(swapTokenA.amount) * 10000000) / supply)
    const swapTokenB = await getTokenAccountInfo(provider, tokenAccountB)
    const tokenB = Math.floor((Number(swapTokenB.amount) * 10000000) / supply)

    const userTransferAuthority = new Account()

    const userAccountA = await MarginSwap.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mintA,
      user.publicKey
    )
    await mintTo(provider.connection, payer, mintA, userAccountA, user, tokenA)
    await approve(provider.connection, payer, userAccountA, userTransferAuthority.publicKey, user, tokenA)

    const userAccountB = await MarginSwap.createAssociatedTokenAccount(
      provider.connection,
      payer,
      mintB,
      user.publicKey
    )
    await mintTo(provider.connection, payer, mintB, userAccountB, user, tokenB)
    await approve(provider.connection, payer, userAccountB, userTransferAuthority.publicKey, user, tokenB)

    const newAccountPool = await createAccount(
      provider.connection,
      payer,
      tokenPool,
      user.publicKey,
      Keypair.generate()
    )

    await marginSwap.tokenSwap.depositAllTokenTypes(
      userAccountA,
      userAccountB,
      newAccountPool,
      userTransferAuthority,
      new BN(10000000),
      new BN(tokenA),
      new BN(tokenB)
    )

    let info
    info = await getTokenAccountInfo(provider, userAccountA)
    assert(info.amount == 0)
    info = await getTokenAccountInfo(provider, userAccountB)
    assert(info.amount == 0)
    info = await getTokenAccountInfo(provider, tokenAccountA)
    assert(info.amount == currentSwapTokenA + tokenA)
    currentSwapTokenA += tokenA
    info = await getTokenAccountInfo(provider, tokenAccountB)
    assert(info.amount == currentSwapTokenB + tokenB)
    currentSwapTokenB += tokenB
    info = await getTokenAccountInfo(provider, newAccountPool)
    assert(info.amount == 10000000)
  })

  it("withdraw all token types", async () => {
    const poolMintInfo = await MarginSwap.getMintInfo(provider.connection, marginSwap.tokenSwap.poolToken)
    const supply = Number(poolMintInfo.supply)
    let swapTokenA = await getTokenAccountInfo(provider, tokenAccountA)
    let swapTokenB = await getTokenAccountInfo(provider, tokenAccountB)
    const feeAmount = Math.floor(10000000 / 6)
    const poolTokenAmount = 10000000 - feeAmount
    const tokenA = Math.floor((Number(swapTokenA.amount) * poolTokenAmount) / supply)
    const tokenB = Math.floor((Number(swapTokenB.amount) * poolTokenAmount) / supply)

    const userAccountA = await createAccount(provider.connection, payer, mintA, user.publicKey, Keypair.generate())
    const userAccountB = await createAccount(provider.connection, payer, mintB, user.publicKey, Keypair.generate())

    const userTransferAuthority = new Account()
    await marginSwap.approve(
      provider.connection,
      tokenAccountPool,
      userTransferAuthority.publicKey,
      user,
      new BN(10000000),
      new Account(payer.secretKey)
    )

    await marginSwap.tokenSwap.withdrawAllTokenTypes(
      userAccountA,
      userAccountB,
      tokenAccountPool,
      userTransferAuthority,
      new BN(10000000),
      new BN(tokenA),
      new BN(tokenB)
    )

    swapTokenA = await getTokenAccountInfo(provider, tokenAccountA)
    swapTokenB = await getTokenAccountInfo(provider, tokenAccountB)

    let info = await getTokenAccountInfo(provider, tokenAccountPool)
    assert(Number(info.amount) == 1000000000 - 10000000)
    assert(Number(swapTokenA.amount) == currentSwapTokenA - tokenA)
    currentSwapTokenA -= tokenA
    assert(Number(swapTokenB.amount) == currentSwapTokenB - tokenB)
    currentSwapTokenB -= tokenB
    info = await getTokenAccountInfo(provider, userAccountA)
    assert(Number(info.amount) == tokenA)
    info = await getTokenAccountInfo(provider, userAccountB)
    assert(Number(info.amount) == tokenB)
    info = await getTokenAccountInfo(provider, marginSwap.tokenSwap.feeAccount)
    assert(Number(info.amount) == feeAmount)
    currentFeeAmount = feeAmount
  })

  it("swap", async () => {
    const userAccountA = await createAccount(provider.connection, payer, mintA, user.publicKey, Keypair.generate())
    await mintTo(provider.connection, payer, mintA, userAccountA, user, 100000)
    const userTransferAuthority = new Account()
    await approve(provider.connection, payer, userAccountA, userTransferAuthority.publicKey, user, 100000)

    const userAccountB = await createAccount(provider.connection, payer, mintB, user.publicKey, Keypair.generate())
    const poolAccount = null

    await marginSwap.tokenSwap.swap(
      userAccountA,
      tokenAccountA,
      tokenAccountB,
      userAccountB,
      poolAccount,
      userTransferAuthority,
      new BN(100000),
      new BN(90674)
    )

    await sleep(500)

    let info
    info = await getTokenAccountInfo(provider, userAccountA)
    assert(Number(info.amount) == 0)

    info = await getTokenAccountInfo(provider, userAccountB)
    assert(Number(info.amount) == 90674)

    info = await getTokenAccountInfo(provider, tokenAccountA)
    assert(Number(info.amount) == currentSwapTokenA + 100000)
    currentSwapTokenA += 100000

    info = await getTokenAccountInfo(provider, tokenAccountB)
    assert(Number(info.amount) == currentSwapTokenB - 90674)
    currentSwapTokenB -= 90674

    info = await getTokenAccountInfo(provider, tokenAccountPool)
    assert(Number(info.amount) == 1000000000 - 10000000)

    info = await getTokenAccountInfo(provider, marginSwap.tokenSwap.feeAccount)
    assert(Number(info.amount) == currentFeeAmount + 22277)

    if (poolAccount != null) {
      info = await getTokenAccountInfo(provider, poolAccount)
      assert(Number(info.amount) == 0)
    }
  })

  it("create account, approve, swap all at once", async () => {
    const userAccountA = await createAccount(provider.connection, payer, mintA, user.publicKey, Keypair.generate())
    await mintTo(provider.connection, payer, mintA, userAccountA, user, 100000)

    const newAccount = new Account()
    const transaction = new Transaction()
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: user.publicKey,
        newAccountPubkey: newAccount.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(AccountLayout.span),
        space: AccountLayout.span,
        programId: TOKEN_PROGRAM_ID
      })
    )

    transaction.add(
      marginSwap.createInitAccountInstruction(TOKEN_PROGRAM_ID, mintB, newAccount.publicKey, user.publicKey)
    )

    const userTransferAuthority = new Account()
    transaction.add(
      marginSwap.createApproveInstruction(
        TOKEN_PROGRAM_ID,
        userAccountA,
        userTransferAuthority.publicKey,
        user.publicKey,
        new BN(100000)
      )
    )

    transaction.add(
      TokenSwap.swapInstruction(
        marginSwap.tokenSwap.tokenSwap,
        marginSwap.tokenSwap.authority,
        userTransferAuthority.publicKey,
        userAccountA,
        marginSwap.tokenSwap.tokenAccountA,
        marginSwap.tokenSwap.tokenAccountB,
        newAccount.publicKey,
        marginSwap.tokenSwap.poolToken,
        marginSwap.tokenSwap.feeAccount,
        null,
        marginSwap.tokenSwap.swapProgramId,
        marginSwap.tokenSwap.tokenProgramId,
        new BN(100000),
        new BN(0)
      )
    )

    await sendAndConfirmTransaction(provider.connection, transaction, [user, newAccount, userTransferAuthority])

    let info: RawAccount
    info = await getTokenAccountInfo(provider, tokenAccountA)
    currentSwapTokenA = Number(info.amount)
    info = await getTokenAccountInfo(provider, tokenAccountB)
    currentSwapTokenB = Number(info.amount)
  })

  function tradingTokensToPoolTokens(sourceAmount: number, swapSourceAmount: number, poolAmount: number): number {
    const tradingFee = (sourceAmount / 2) * (25 / 10000)
    const sourceAmountPostFee = sourceAmount - tradingFee
    const root = Math.sqrt(sourceAmountPostFee / swapSourceAmount + 1)
    return Math.floor(poolAmount * (root - 1))
  }

  it("deposit one exact amount in", async () => {
    // Pool token amount to deposit on one side
    const depositAmount = 10000

    const poolMintInfo = await MarginSwap.getMintInfo(provider.connection, marginSwap.tokenSwap.poolToken)
    const supply = Number(poolMintInfo.supply)
    const swapTokenA = await getTokenAccountInfo(provider, tokenAccountA)
    const poolTokenA = tradingTokensToPoolTokens(depositAmount, Number(swapTokenA.amount), supply)
    const swapTokenB = await getTokenAccountInfo(provider, tokenAccountB)
    const poolTokenB = tradingTokensToPoolTokens(depositAmount, Number(swapTokenB.amount), supply)

    const userTransferAuthority = new Account()
    const userAccountA = await createAccount(provider.connection, payer, mintA, user.publicKey, Keypair.generate())
    await mintTo(provider.connection, payer, mintA, userAccountA, user, depositAmount)
    await approve(provider.connection, payer, userAccountA, userTransferAuthority.publicKey, user, depositAmount)
    const userAccountB = await createAccount(provider.connection, payer, mintB, user.publicKey, Keypair.generate())
    await mintTo(provider.connection, payer, mintB, userAccountB, user, depositAmount)
    await approve(provider.connection, payer, userAccountB, userTransferAuthority.publicKey, user, depositAmount)
    const newAccountPool = await createAccount(
      provider.connection,
      payer,
      marginSwap.tokenSwap.poolToken,
      user.publicKey,
      Keypair.generate()
    )

    await marginSwap.tokenSwap.depositSingleTokenTypeExactAmountIn(
      userAccountA,
      newAccountPool,
      userTransferAuthority,
      new BN(depositAmount),
      new BN(poolTokenA)
    )

    let info: RawAccount
    info = await getTokenAccountInfo(provider, userAccountA)
    assert(Number(info.amount) == 0)
    info = await getTokenAccountInfo(provider, tokenAccountA)
    assert(Number(info.amount) == currentSwapTokenA + depositAmount)
    currentSwapTokenA += depositAmount

    await marginSwap.tokenSwap.depositSingleTokenTypeExactAmountIn(
      userAccountB,
      newAccountPool,
      userTransferAuthority,
      new BN(depositAmount),
      new BN(poolTokenB)
    )

    info = await getTokenAccountInfo(provider, userAccountB)
    assert(Number(info.amount) == 0)
    info = await getTokenAccountInfo(provider, tokenAccountB)
    assert(Number(info.amount) == currentSwapTokenB + depositAmount)
    currentSwapTokenB += depositAmount
    info = await getTokenAccountInfo(provider, newAccountPool)
    assert(Number(info.amount) >= poolTokenA + poolTokenB)
  })

  it("withrdaw one exact amount out", async () => {
    // Pool token amount to withdraw on one side
    const withdrawAmount = 50000
    const roundingAmount = 1.0001 // make math a little easier

    const poolMintInfo = await MarginSwap.getMintInfo(provider.connection, marginSwap.tokenSwap.poolToken)
    const supply = Number(poolMintInfo.supply)
    const swapTokenA = await getTokenAccountInfo(provider, tokenAccountA)
    const swapTokenAPost = Number(swapTokenA.amount) - withdrawAmount
    const poolTokenA = tradingTokensToPoolTokens(withdrawAmount, swapTokenAPost, supply)
    let adjustedPoolTokenA = poolTokenA * roundingAmount
    adjustedPoolTokenA *= 1 + 1 / 6

    const swapTokenB = await getTokenAccountInfo(provider, tokenAccountB)
    const swapTokenBPost = Number(swapTokenB.amount) - withdrawAmount
    const poolTokenB = tradingTokensToPoolTokens(withdrawAmount, swapTokenBPost, supply)
    let adjustedPoolTokenB = poolTokenB * roundingAmount
    adjustedPoolTokenB *= 1 + 1 / 6

    const userTransferAuthority = new Account()
    const userAccountA = await createAccount(provider.connection, payer, mintA, user.publicKey, Keypair.generate())
    const userAccountB = await createAccount(provider.connection, payer, mintB, user.publicKey, Keypair.generate())

    const poolAccount = await getTokenAccountInfo(provider, tokenAccountPool)
    const poolTokenAmount = Number(poolAccount.amount)
    await approve(
      provider.connection,
      payer,
      tokenAccountPool,
      userTransferAuthority.publicKey,
      user,
      BigInt(Math.floor(adjustedPoolTokenA + adjustedPoolTokenB))
    )

    await marginSwap.tokenSwap.withdrawSingleTokenTypeExactAmountOut(
      userAccountA,
      tokenAccountPool,
      userTransferAuthority,
      new BN(withdrawAmount),
      new BN(adjustedPoolTokenA)
    )

    let info: RawAccount
    info = await getTokenAccountInfo(provider, userAccountA)
    assert(Number(info.amount) == withdrawAmount)
    info = await getTokenAccountInfo(provider, tokenAccountA)
    assert(Number(info.amount) == currentSwapTokenA - withdrawAmount)
    currentSwapTokenA += withdrawAmount
    info = await getTokenAccountInfo(provider, tokenAccountPool)
    assert(Number(info.amount) >= poolTokenAmount - adjustedPoolTokenA)

    await marginSwap.tokenSwap.withdrawSingleTokenTypeExactAmountOut(
      userAccountB,
      tokenAccountPool,
      userTransferAuthority,
      new BN(withdrawAmount),
      new BN(adjustedPoolTokenB)
    )

    info = await getTokenAccountInfo(provider, userAccountB)
    assert(Number(info.amount) == withdrawAmount)
    info = await getTokenAccountInfo(provider, tokenAccountB)
    assert(Number(info.amount) == currentSwapTokenB - withdrawAmount)
    currentSwapTokenB += withdrawAmount
    info = await getTokenAccountInfo(provider, tokenAccountPool)
    assert(Number(info.amount) >= poolTokenAmount - adjustedPoolTokenA - adjustedPoolTokenB)
  })
})
