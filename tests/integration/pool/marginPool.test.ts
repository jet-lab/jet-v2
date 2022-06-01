import { assert } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider, BN } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"

import MARGIN_CONFIG from "../../../libraries/ts/src/margin/config.json"

import { MarginAccount, PoolAmount, MarginClient, MarginPool, MarginPoolConfig } from "../../../libraries/ts/src"

import { PythClient } from "../pyth/pythClient"
import {
  createAuthority,
  createToken,
  createTokenAccount,
  createUserWallet,
  getMintSupply,
  getTokenBalance,
  registerAdapter,
  sendToken
} from "../util"

describe("margin pool", () => {
  const controlProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.controlProgramId)
  const marginProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.marginProgramId)
  const marginPoolProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.marginPoolProgramId)
  const metadataProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.metadataProgramId)

  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }

  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)

  const payer = (provider.wallet as NodeWallet).payer
  const ownerKeypair = payer

  const programs = MarginClient.getPrograms(provider, "localnet")

  it("Fund payer", async () => {
    const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  let USDC: [PublicKey, PublicKey]
  let TSOL: [PublicKey, PublicKey]

  it("Create tokens", async () => {
    USDC = await createToken(provider, payer, 6, 10_000_000)
    const usdc_supply = await getMintSupply(provider, USDC[0], 6)
    assert(usdc_supply > 0)
    const usdc_balance = await getTokenBalance(provider, confirmOptions.commitment, USDC[1])
    assert(usdc_balance > 0)

    TSOL = await createToken(provider, payer, 9, 10_000)
    const tsol_supply = await getMintSupply(provider, TSOL[0], 9)
    assert(tsol_supply > 0)
    const tsol_balance = await getTokenBalance(provider, confirmOptions.commitment, TSOL[1])
    assert(tsol_balance > 0)
  })

  const FEE_VAULT_USDC: PublicKey = new PublicKey("FEEVAULTUSDC1111111111111111111111111111111")
  const FEE_VAULT_TSOL: PublicKey = new PublicKey("FEEVAULTTSoL1111111111111111111111111111111")

  let USDC_oracle: Keypair
  let TSOL_oracle: Keypair

  const pythClient = new PythClient({
    pythProgramId: "ASfdvRMCan2aoWtbDi5HLXhz2CFfgEkuDoxc57bJLKLX",
    url: "http://127.0.0.1:8899/"
  })

  it("Create oracles", async () => {
    USDC_oracle = Keypair.generate()
    await pythClient.createPriceAccount(payer, USDC_oracle, 1, 0.01, -8)
    TSOL_oracle = Keypair.generate()
    await pythClient.createPriceAccount(payer, TSOL_oracle, 100, 1, -8)
  })

  it("Create authority", async () => {
    await createAuthority(provider, payer)
  })

  
  it("Register adapter", async () => {
    await registerAdapter(provider, payer, marginPoolProgramId, payer)
  })

  const ONE_USDC: number = 1_000_000
  const ONE_TSOL: number = LAMPORTS_PER_SOL

  const DEFAULT_POOL_CONFIG: MarginPoolConfig = {
    borrowRate0: 10,
    borrowRate1: 20,
    borrowRate2: 30,
    borrowRate3: 40,
    utilizationRate1: 10,
    utilizationRate2: 20,
    managementFeeRate: 10,
    managementFeeCollectThreshold: new BN(100),
    flags: new BN(2) // ALLOW_LENDING
  }

  const POOLS = [
    {
      mintAndVault: USDC,
      weight: 10_000,
      config: DEFAULT_POOL_CONFIG
    },
    {
      mintAndVault: TSOL,
      weight: 9_500,
      config: DEFAULT_POOL_CONFIG
    }
  ]

  let maginPool_USDC: MarginPool
  let maginPool_TSOL: MarginPool

  it("Create margin pools", async () => {
    maginPool_USDC = await MarginPool.load(programs, USDC[0])
    await maginPool_USDC.create(
      provider,
      ownerKeypair.publicKey,
      10_000,
      new BN(0),
      FEE_VAULT_USDC,
      Keypair.generate().publicKey,
      USDC_oracle.publicKey,
      POOLS[0].config
    )

    maginPool_TSOL = await MarginPool.load(programs, TSOL[0])
    await maginPool_TSOL.create(
      provider,
      ownerKeypair.publicKey,
      9_500,
      new BN(0),
      FEE_VAULT_TSOL,
      Keypair.generate().publicKey,
      TSOL_oracle.publicKey,
      POOLS[1].config
    )
  })

  let wallet_a: NodeWallet
  let wallet_b: NodeWallet

  let provider_a: AnchorProvider
  let provider_b: AnchorProvider

  it("Create our two user wallets, with some SOL funding to get started", async () => {
    wallet_a = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)
    wallet_b = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)

    provider_a = new AnchorProvider(provider.connection, wallet_a, confirmOptions)
    provider_b = new AnchorProvider(provider.connection, wallet_b, confirmOptions)
  })

  let maginAccount_A: MarginAccount
  let maginAccount_B: MarginAccount

  it("Initialize the margin accounts for each user", async () => {
    anchor.setProvider(provider_a)
    maginAccount_A = await MarginAccount.load(programs, provider_a, provider_a.wallet.publicKey, 0)
    await maginAccount_A.createAccount()

    anchor.setProvider(provider_b)
    maginAccount_B = await MarginAccount.load(programs, provider_b, provider_b.wallet.publicKey, 0)
    await maginAccount_B.createAccount()
  })

  let user_a_usdc_account: PublicKey
  let user_b_tsol_account: PublicKey

  it("Create some tokens for each user to deposit", async () => {
    const payer_A: Keypair = Keypair.fromSecretKey((wallet_a as NodeWallet).payer.secretKey)
    user_a_usdc_account = await createTokenAccount(provider_a, USDC[0], wallet_a.publicKey)
    await sendToken(provider_a, USDC[0], 1_000_000, 6, new PublicKey(USDC[1]), user_a_usdc_account)

    const payer_B: Keypair = Keypair.fromSecretKey((wallet_b as NodeWallet).payer.secretKey)
    user_b_tsol_account = await createTokenAccount(provider_b, TSOL[0], wallet_b.publicKey)
    await sendToken(provider_b, TSOL[0], 1_000, 9, new PublicKey(TSOL[1]), user_b_tsol_account)
  })

  it("Set the prices for each token", async () => {
    await pythClient.setPythPrice(ownerKeypair, USDC_oracle.publicKey, 1, 0.01, -8)
    await pythClient.setPythPrice(ownerKeypair, TSOL_oracle.publicKey, 100, 1, -8)
  })

  it("Deposit user funds into their margin accounts", async () => {
    await maginAccount_A.deposit(maginPool_USDC, user_a_usdc_account, new BN(1_000_000 * ONE_USDC))
    assert((await getTokenBalance(provider_a, "processed", user_a_usdc_account)) == 0)
    await maginPool_USDC.refreshPosition(maginAccount_A)

    await maginAccount_B.deposit(maginPool_TSOL, user_b_tsol_account, new BN(1_000 * ONE_TSOL))
    assert((await getTokenBalance(provider_b, "processed", user_b_tsol_account)) == 0)
    await maginPool_TSOL.refreshPosition(maginAccount_B)
  })

  it("Set the prices for each token", async () => {
    await pythClient.setPythPrice(ownerKeypair, USDC_oracle.publicKey, 1, 0.01, -8)
    await pythClient.setPythPrice(ownerKeypair, TSOL_oracle.publicKey, 100, 1, -8)
  })

  it("Have each user borrow the other's funds", async () => {
    await maginPool_TSOL.marginBorrow(maginAccount_A, new BN(10 * ONE_TSOL))
    await maginPool_USDC.marginBorrow(maginAccount_B, new BN(1_000 * ONE_USDC))
  })

  it("Users repay their loans", async () => {
    await maginPool_TSOL.marginRepay(maginAccount_A, PoolAmount.tokens(new BN(10 * ONE_TSOL)))
    await maginPool_USDC.marginRepay(maginAccount_B, PoolAmount.tokens(new BN(1_000 * ONE_USDC)))
  })

  it("Users withdraw their funds", async () => {
    await maginPool_USDC.marginWithdraw(
      maginAccount_A,
      user_a_usdc_account,
      PoolAmount.tokens(new BN(1_000_000 * ONE_USDC))
    )
    await maginPool_TSOL.marginWithdraw(
      maginAccount_B,
      user_b_tsol_account,
      PoolAmount.tokens(new BN(1_000 * ONE_TSOL))
    )
  })

  it("Now verify that the users got all their tokens back", async () => {
    assert((await getTokenBalance(provider, "processed", user_a_usdc_account)) == 1_000_000)
    assert((await getTokenBalance(provider, "processed", user_b_tsol_account)) == 1_000)
  })
  
})
