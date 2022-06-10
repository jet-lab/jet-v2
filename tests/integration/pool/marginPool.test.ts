import { expect } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider, BN } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"

import MARGIN_CONFIG from "../../../libraries/ts/src/margin/config.json"

import {
  MarginAccount,
  PoolAmount,
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
  getMintSupply,
  getTokenBalance,
  registerAdapter,
  sendToken
} from "../util"

describe("margin pool", () => {
  // SUITE SETUP
  const marginPoolProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.marginPoolProgramId)
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }
  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)
  const payer = (provider.wallet as NodeWallet).payer
  const ownerKeypair = payer
  const programs = MarginClient.getPrograms(provider, "localnet")
  const manager = new PoolManager(programs, provider)
  let USDC
  let SOL

  it("Fund payer", async () => {
    const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  it("Create tokens", async () => {
    // SETUP
    USDC = await createToken(provider, payer, 6, 10_000_000)
    SOL = await createToken(provider, payer, 9, 10_000)

    // ACT
    const usdc_supply = await getMintSupply(provider, USDC[0], 6)
    const usdc_balance = await getTokenBalance(provider, confirmOptions.commitment, USDC[1])
    const sol_supply = await getMintSupply(provider, SOL[0], 9)
    const sol_balance = await getTokenBalance(provider, confirmOptions.commitment, SOL[1])

    // TEST
    expect(usdc_supply).to.eq(10_000_000)
    expect(usdc_balance).to.eq(10_000_000)
    expect(sol_supply).to.eq(10_000)
    expect(sol_balance).to.eq(10_000)
  })

  const FEE_VAULT_USDC: PublicKey = new PublicKey("FEEVAULTUSDC1111111111111111111111111111111")
  const FEE_VAULT_SOL: PublicKey = new PublicKey("FEEVAULTTSoL1111111111111111111111111111111")

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
    await createAuthority(provider, payer)
  })

  it("Register adapter", async () => {
    await registerAdapter(provider, payer, marginPoolProgramId, payer)
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
      mintAndVault: SOL,
      weight: 9_500,
      config: DEFAULT_POOL_CONFIG
    }
  ]

  let marginPool_USDC: Pool
  let marginPool_SOL: Pool

  it("Load Pools", async () => {
    marginPool_SOL = await manager.load({ tokenMint: SOL[0] })
    marginPool_USDC = await manager.load({ tokenMint: USDC[0] })
  })

  it("Create margin pools", async () => {
    await manager.create({
      tokenMint: USDC[0],
      collateralWeight: 10_000,
      collateralMaxStaleness: new BN(0),
      feeDestination: FEE_VAULT_USDC,
      pythProduct: USDC_oracle[0].publicKey,
      pythPrice: USDC_oracle[1].publicKey,
      marginPoolConfig: POOLS[0].config
    })
    await manager.create({
      tokenMint: SOL[0],
      collateralWeight: 9_500,
      collateralMaxStaleness: new BN(0),
      feeDestination: FEE_VAULT_SOL,
      pythProduct: SOL_oracle[0].publicKey,
      pythPrice: SOL_oracle[1].publicKey,
      marginPoolConfig: POOLS[1].config
    })
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

  let marginAccount_A: MarginAccount
  let marginAccount_B: MarginAccount

  it("Initialize the margin accounts for each user", async () => {
    anchor.setProvider(provider_a)
    marginAccount_A = await MarginAccount.load(programs, provider_a, provider_a.wallet.publicKey, 0)
    await marginAccount_A.createAccount()

    anchor.setProvider(provider_b)
    marginAccount_B = await MarginAccount.load(programs, provider_b, provider_b.wallet.publicKey, 0)
    await marginAccount_B.createAccount()
  })

  let user_a_usdc_account
  let user_a_sol_account
  let user_b_sol_account
  let user_b_usdc_account

  it("Create some tokens for each user to deposit", async () => {
    // SETUP
    const payer_A: Keypair = Keypair.fromSecretKey((wallet_a as NodeWallet).payer.secretKey)
    user_a_usdc_account = await createTokenAccount(provider, USDC[0], wallet_a.publicKey, payer_A)
    user_a_sol_account = await createTokenAccount(provider, SOL[0], wallet_a.publicKey, payer_A)

    const payer_B: Keypair = Keypair.fromSecretKey((wallet_b as NodeWallet).payer.secretKey)
    user_b_sol_account = await createTokenAccount(provider, SOL[0], wallet_b.publicKey, payer_B)
    user_b_usdc_account = await createTokenAccount(provider, USDC[0], wallet_b.publicKey, payer_B)

    // ACT
    await sendToken(provider, USDC[0], 500_000, 6, ownerKeypair, new PublicKey(USDC[1]), user_a_usdc_account)
    await sendToken(provider, SOL[0], 50, 9, ownerKeypair, new PublicKey(SOL[1]), user_a_sol_account)
    await sendToken(provider, SOL[0], 500, 9, ownerKeypair, new PublicKey(SOL[1]), user_b_sol_account)
    await sendToken(provider, USDC[0], 50, 6, ownerKeypair, new PublicKey(USDC[1]), user_b_usdc_account)

    // TEST
    expect(await getTokenBalance(provider, "processed", user_a_usdc_account)).to.eq(500_000)
    expect(await getTokenBalance(provider, "processed", user_a_sol_account)).to.eq(50)
    expect(await getTokenBalance(provider, "processed", user_b_sol_account)).to.eq(500)
    expect(await getTokenBalance(provider, "processed", user_b_usdc_account)).to.eq(50)
  })

  it("Set the prices for each token", async () => {
    await pythClient.setPythPrice(ownerKeypair, USDC_oracle[1].publicKey, 1, 0.01, -8)
    await pythClient.setPythPrice(ownerKeypair, SOL_oracle[1].publicKey, 100, 1, -8)
  })

  it("Deposit user funds into their margin accounts", async () => {
    // ACT
    await marginAccount_A.deposit(marginPool_USDC, user_a_usdc_account, new BN(500_000 * ONE_USDC))
    await marginAccount_A.deposit(marginPool_SOL, user_a_sol_account, new BN(50 * ONE_SOL))
    await marginPool_USDC.refreshPosition(marginAccount_A)
    await marginPool_SOL.refreshPosition(marginAccount_A)

    await marginAccount_B.deposit(marginPool_SOL, user_b_sol_account, new BN(500 * ONE_SOL))
    await marginAccount_B.deposit(marginPool_USDC, user_b_usdc_account, new BN(50 * ONE_USDC))
    await marginPool_SOL.refreshPosition(marginAccount_B)
    await marginPool_USDC.refreshPosition(marginAccount_B)

    // TEST
    expect(await getTokenBalance(provider, "processed", user_b_sol_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_b_usdc_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_a_usdc_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", user_a_sol_account)).to.eq(0)
    expect(await getTokenBalance(provider, "processed", marginPool_USDC.addresses.vault)).to.eq(500_050)
    expect(await getTokenBalance(provider, "processed", marginPool_SOL.addresses.vault)).to.eq(550)
  })

  it("Have each user borrow the other's funds", async () => {
    // SETUP
    const borrowedSOL = new BN(10 * ONE_SOL)
    const borrowedUSDC = new BN(1_000 * ONE_USDC)

    // ACT
    await marginPool_SOL.marginBorrow({
      marginAccount: marginAccount_A,
      amount: borrowedSOL
    })
    await marginPool_USDC.marginBorrow({ marginAccount: marginAccount_B, amount: borrowedUSDC })
    await marginPool_SOL.refresh()
    await marginPool_USDC.refresh()

    const SOLLoanNotes = new BN(marginPool_SOL.info?.loanNoteMint.supply)
    const USDCLoanNotes = new BN(marginPool_USDC.info?.loanNoteMint.supply)

    // TEST
    expect(SOLLoanNotes.toNumber()).to.eq(borrowedSOL.toNumber())
    expect(USDCLoanNotes.toNumber()).to.eq(borrowedUSDC.toNumber())
  })

  it("User A repays his SOL loan", async () => {
    //SETUP
    await marginPool_SOL.refresh()
    const owedSOL = new BN(marginPool_SOL.info?.loanNoteMint.supply)

    // ACT
    await marginPool_SOL.marginRepay({ marginAccount: marginAccount_A, amount: PoolAmount.tokens(owedSOL) })
    await marginPool_SOL.refresh()

    // TEST
    const SOLLoanNotes = new BN(marginPool_SOL.info?.loanNoteMint.supply)
    expect(SOLLoanNotes.toNumber()).to.be.below(10)
  })

  it("User B repays his USDC loan", async () => {
    // SETUP
    await marginPool_USDC.refresh()
    const owedUSDC = new BN(marginPool_USDC.info?.loanNoteMint.supply)

    // ACT
    await marginPool_USDC.marginRepay({ marginAccount: marginAccount_B, amount: PoolAmount.tokens(owedUSDC) })
    await marginPool_USDC.refresh()

    // TEST
    const USDCLoanNotes = new BN(marginPool_USDC.info?.loanNoteMint.supply)
    expect(USDCLoanNotes.toNumber()).to.be.below(10)
  })

  it("Users withdraw their funds", async () => {
    // ACT
    await marginPool_USDC.marginWithdraw({
      marginAccount: marginAccount_A,
      destination: user_a_usdc_account,
      amount: PoolAmount.tokens(new BN(400_000 * ONE_USDC))
    })
    await marginPool_SOL.marginWithdraw({
      marginAccount: marginAccount_B,
      destination: user_b_sol_account,
      amount: PoolAmount.tokens(new BN(400 * ONE_SOL))
    })

    // TEST
    const tokenBalanceA = await getTokenBalance(provider, "processed", user_a_usdc_account)
    const tokenBalanceB = await getTokenBalance(provider, "processed", user_b_sol_account)
    expect(tokenBalanceA).to.eq(400_000)
    expect(tokenBalanceB).to.eq(400)
  })
})
