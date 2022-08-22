import { expect } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider, BN } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"

import {
  MarginAccount,
  PoolTokenChange,
  MarginClient,
  Pool,
  MarginPoolConfigData,
  PoolManager
} from "../../../libraries/ts/src"

import { PythClient } from "../pyth/pythClient"
import {
  createAuthority,
  createToken,
  createTokenAccount,
  createUserWallet,
  DEFAULT_MARGIN_CONFIG,
  getMintSupply,
  getTokenBalance,
  MARGIN_POOL_PROGRAM_ID,
  registerAdapter,
  sendToken,
  TestToken
} from "../util"

describe("margin pool deposit", async () => {
  // SUITE SETUP
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }
  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)
  const payer = (provider.wallet as NodeWallet).payer
  const ownerKeypair = payer
  const programs = MarginClient.getPrograms(provider, DEFAULT_MARGIN_CONFIG)
  const manager = new PoolManager(programs, provider)
  let USDC: TestToken = null as any
  let SOL: TestToken = null as any

  it("Fund payer", async () => {
    const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  it("Create tokens", async () => {
    // SETUP
    USDC = await createToken(provider, payer, 6, 10_000_000, "USDC")
    SOL = await createToken(provider, payer, 9, 10_000, "SOL")

    // ACT
    const usdc_supply = await getMintSupply(provider, USDC.mint, 6)
    const usdc_balance = await getTokenBalance(provider, confirmOptions.commitment, USDC.vault)
    const sol_supply = await getMintSupply(provider, SOL.mint, 9)
    const sol_balance = await getTokenBalance(provider, confirmOptions.commitment, SOL.vault)

    // TEST
    expect(usdc_supply).to.eq(10_000_000)
    expect(usdc_balance).to.eq(10_000_000)
    expect(sol_supply).to.eq(10_000)
    expect(sol_balance).to.eq(10_000)
  })

  let USDC_oracle: Keypair[]
  let SOL_oracle: Keypair[]

  const pythClient = new PythClient({
    pythProgramId: "FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL",
    url: "http://127.0.0.1:8899/"
  })

  it("Create oracles", async () => {
    USDC_oracle = [Keypair.generate(), Keypair.generate()]
    await pythClient.createPriceAccount(payer, USDC_oracle[0], "USD", USDC_oracle[1], 1, 0.01, -8)
    SOL_oracle = [Keypair.generate(), Keypair.generate()]
    await pythClient.createPriceAccount(payer, SOL_oracle[0], "USD", SOL_oracle[1], 100, 1, -8)
  })

  it("Create authority", async () => {
    await createAuthority(programs, provider)
  })

  it("Register adapter", async () => {
    await registerAdapter(programs, provider, payer, MARGIN_POOL_PROGRAM_ID, payer)
  })

  const ONE_USDC = 1_000_000
  const ONE_SOL: number = LAMPORTS_PER_SOL

  const DEFAULT_POOL_CONFIG: MarginPoolConfigData = {
    borrowRate0: 10,
    borrowRate1: 20,
    borrowRate2: 30,
    borrowRate3: 40,
    utilizationRate1: 10,
    utilizationRate2: 20,
    managementFeeRate: 10,
    managementFeeCollectThreshold: new BN(2),
    flags: new BN(2) // ALLOW_LENDING
  }

  let marginPool_USDC: Pool
  let marginPool_SOL: Pool
  let pools: Pool[]

  it("Load Pools", async () => {
    marginPool_SOL = await manager.load({ tokenMint: SOL.mint, tokenConfig: SOL.tokenConfig })
    marginPool_USDC = await manager.load({ tokenMint: USDC.mint, tokenConfig: USDC.tokenConfig })
    pools = [marginPool_SOL, marginPool_USDC]
  })

  it("Create margin pools", async () => {
    await manager.create({
      tokenMint: USDC.mint,
      collateralWeight: 1_00,
      maxLeverage: 4_00,
      pythProduct: USDC_oracle[0].publicKey,
      pythPrice: USDC_oracle[1].publicKey,
      marginPoolConfig: DEFAULT_POOL_CONFIG
    })
    await manager.create({
      tokenMint: SOL.mint,
      collateralWeight: 95,
      maxLeverage: 4_00,
      pythProduct: SOL_oracle[0].publicKey,
      pythPrice: SOL_oracle[1].publicKey,
      marginPoolConfig: DEFAULT_POOL_CONFIG
    })
  })

  let wallet_a: NodeWallet
  let wallet_b: NodeWallet
  let wallet_c: NodeWallet

  let provider_a: AnchorProvider
  let provider_b: AnchorProvider
  let provider_c: AnchorProvider

  it("Create our two user wallets, with some SOL funding to get started", async () => {
    wallet_a = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)
    wallet_b = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)
    wallet_c = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)

    provider_a = new AnchorProvider(provider.connection, wallet_a, confirmOptions)
    provider_b = new AnchorProvider(provider.connection, wallet_b, confirmOptions)
    provider_c = new AnchorProvider(provider.connection, wallet_c, confirmOptions)
  })

  let marginAccount_A: MarginAccount
  let marginAccount_B: MarginAccount
  let marginAccount_C: MarginAccount

  it("Initialize the margin accounts for each user", async () => {
    anchor.setProvider(provider_a)
    marginAccount_A = await MarginAccount.load({
      programs,
      provider: provider_a,
      owner: provider_a.wallet.publicKey,
      seed: 0
    })
    await marginAccount_A.createAccount()

    anchor.setProvider(provider_b)
    marginAccount_B = await MarginAccount.load({
      programs,
      provider: provider_b,
      owner: provider_b.wallet.publicKey,
      seed: 0
    })
    await marginAccount_B.createAccount()

    anchor.setProvider(provider_c)
    marginAccount_C = await MarginAccount.load({
      programs,
      provider: provider_c,
      owner: provider_c.wallet.publicKey,
      seed: 0
    })
    await marginAccount_C.createAccount()
  })

  let user_a_usdc_account: PublicKey
  let user_a_sol_account: PublicKey
  let user_b_sol_account: PublicKey
  let user_b_usdc_account: PublicKey
  let user_c_sol_account: PublicKey
  let user_c_usdc_account: PublicKey

  it("Create some tokens for each user to deposit", async () => {
    // SETUP
    const payer_A: Keypair = Keypair.fromSecretKey((wallet_a as NodeWallet).payer.secretKey)
    user_a_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_a.publicKey, payer_A)
    user_a_sol_account = await createTokenAccount(provider, SOL.mint, wallet_a.publicKey, payer_A)

    const payer_B: Keypair = Keypair.fromSecretKey((wallet_b as NodeWallet).payer.secretKey)
    user_b_sol_account = await createTokenAccount(provider, SOL.mint, wallet_b.publicKey, payer_B)
    user_b_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_b.publicKey, payer_B)

    const payer_C: Keypair = Keypair.fromSecretKey((wallet_c as NodeWallet).payer.secretKey)
    user_c_sol_account = await createTokenAccount(provider, SOL.mint, wallet_c.publicKey, payer_C)
    user_c_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_c.publicKey, payer_C)

    // ACT
    await sendToken(provider, USDC.mint, 500_000, 6, ownerKeypair, USDC.vault, user_a_usdc_account)
    await sendToken(provider, SOL.mint, 50, 9, ownerKeypair, SOL.vault, user_a_sol_account)
    await sendToken(provider, SOL.mint, 500, 9, ownerKeypair, SOL.vault, user_b_sol_account)
    await sendToken(provider, USDC.mint, 50, 6, ownerKeypair, USDC.vault, user_b_usdc_account)
    await sendToken(provider, SOL.mint, 1, 9, ownerKeypair, SOL.vault, user_c_sol_account)
    await sendToken(provider, USDC.mint, 1, 6, ownerKeypair, USDC.vault, user_c_usdc_account)

    // TEST
    expect(await getTokenBalance(provider, "processed", user_a_usdc_account)).to.eq(500_000)
    expect(await getTokenBalance(provider, "processed", user_a_sol_account)).to.eq(50)
    expect(await getTokenBalance(provider, "processed", user_b_sol_account)).to.eq(500)
    expect(await getTokenBalance(provider, "processed", user_b_usdc_account)).to.eq(50)
  })

  it("Refresh pools", async () => {
    await marginPool_USDC.refresh()
    await marginPool_SOL.refresh()
  })

  it("Deposit user funds into their margin accounts", async () => {
    // ACT
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_A,
      source: user_a_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(500_000 * ONE_USDC))
    })
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_B,
      source: user_b_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(50 * ONE_USDC))
    })
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_C,
      source: user_c_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(ONE_USDC))
    })
    await pythClient.setPythPrice(ownerKeypair, USDC_oracle[1].publicKey, 1, 0.01, -8)
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_A)
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_B)
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_C)

    await marginPool_SOL.deposit({
      marginAccount: marginAccount_A,
      source: user_a_sol_account,
      change: PoolTokenChange.shiftBy(new BN(50 * ONE_SOL))
    })
    await marginPool_SOL.deposit({
      marginAccount: marginAccount_B,
      source: user_b_sol_account,
      change: PoolTokenChange.shiftBy(new BN(500 * ONE_SOL))
    })
    await marginPool_SOL.deposit({
      marginAccount: marginAccount_C,
      source: user_c_sol_account,
      change: PoolTokenChange.shiftBy(new BN(ONE_SOL))
    })
    await pythClient.setPythPrice(ownerKeypair, SOL_oracle[1].publicKey, 100, 1, -8)
    await marginPool_SOL.marginRefreshPositionPrice(marginAccount_A)
    await marginPool_SOL.marginRefreshPositionPrice(marginAccount_B)
    await marginPool_SOL.marginRefreshPositionPrice(marginAccount_C)
    await marginAccount_A.refresh()
    await marginAccount_B.refresh()
    await marginAccount_C.refresh()

    // TEST
    expect(await getTokenBalance(provider, "processed", user_b_sol_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_b_usdc_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_a_usdc_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_a_sol_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", marginPool_USDC.addresses.vault)).to.eq(500_050 + 1)
    expect(await getTokenBalance(provider, "processed", marginPool_SOL.addresses.vault)).to.eq(550 + 1)
  })

  it("Users withdraw their funds", async () => {
    // ACT
    await pythClient.setPythPrice(ownerKeypair, USDC_oracle[1].publicKey, 1, 0.01, -8)
    await pythClient.setPythPrice(ownerKeypair, SOL_oracle[1].publicKey, 100, 1, -8)
    await marginPool_USDC.withdraw({
      marginAccount: marginAccount_A,
      pools,
      destination: user_a_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(400_000 * ONE_USDC))
    })
    await marginPool_SOL.withdraw({
      marginAccount: marginAccount_B,
      pools,
      destination: user_b_sol_account,
      change: PoolTokenChange.shiftBy(new BN(400 * ONE_SOL))
    })

    // TEST
    const tokenBalanceA = await getTokenBalance(provider, "processed", user_a_usdc_account)
    const tokenBalanceB = await getTokenBalance(provider, "processed", user_b_sol_account)
    expect(tokenBalanceA).to.eq(400_000)
    expect(tokenBalanceB).to.eq(400)

    expect(await getTokenBalance(provider, "processed", marginPool_USDC.addresses.vault)).to.eq(100_050 + 1)
    expect(await getTokenBalance(provider, "processed", marginPool_SOL.addresses.vault)).to.eq(150 + 1)
  })

  provider.opts.skipPreflight = true

  it("Close margin accounts", async () => {
    await marginPool_USDC.withdrawAndCloseDepositPosition({
      marginAccount: marginAccount_A,
      destination: user_a_usdc_account
    })
    await marginPool_SOL.withdrawAndCloseDepositPosition({
      marginAccount: marginAccount_A,
      destination: user_a_sol_account
    })
    await marginAccount_A.closeAccount()

    await marginPool_USDC.withdrawAndCloseDepositPosition({
      marginAccount: marginAccount_B,
      destination: user_b_usdc_account
    })
    await marginPool_SOL.withdrawAndCloseDepositPosition({
      marginAccount: marginAccount_B,
      destination: user_b_sol_account
    })
    await marginAccount_B.closeAccount()
  })
})
