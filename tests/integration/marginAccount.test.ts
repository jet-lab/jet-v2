import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"
import { expect } from "chai"

import { MarginAccount, MarginClient, Pool, PoolManager } from "../../libraries/ts/src"
import { createAuthority, createUserWallet, DEFAULT_MARGIN_CONFIG } from "./util"

describe("margin account", async () => {
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }

  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)

  const payer: Keypair = (provider.wallet as NodeWallet).payer

  const programs = MarginClient.getPrograms(provider, DEFAULT_MARGIN_CONFIG)
  let poolManager: PoolManager
  let pools: Record<string, Pool>

  it("Fund payer", async () => {
    const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  let wallet_a: NodeWallet
  let wallet_b: NodeWallet

  let provider_a: AnchorProvider
  let provider_b: AnchorProvider

  let marginAccount_A: MarginAccount
  let marginAccount_B: MarginAccount

  it("Create two user wallets", async () => {
    // Create our two user wallets, with some SOL funding to get started
    wallet_a = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)
    wallet_b = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL)
    provider_a = new AnchorProvider(provider.connection, wallet_a, confirmOptions)
    provider_b = new AnchorProvider(provider.connection, wallet_b, confirmOptions)
  })

  it("Create authority", async () => {
    await createAuthority(programs, provider)
  })

  it("Fetch pools", async () => {
    poolManager = new PoolManager(programs, provider)
    pools = await poolManager.loadAll()
  })

  it("Create margin accounts", async () => {
    // Initialize the margin accounts for each user
    anchor.setProvider(provider_a)
    marginAccount_A = await MarginAccount.load({
      programs,
      provider: provider_a,
      pools,
      owner: provider_a.wallet.publicKey,
      seed: 0
    })
    await marginAccount_A.createAccount()

    anchor.setProvider(provider_b)
    marginAccount_B = await MarginAccount.load({
      programs,
      provider: provider_b,
      pools,
      owner: provider_b.wallet.publicKey,
      seed: 0
    })
    await marginAccount_B.createAccount()
  })

  it("Load margin account for user A", async () => {
    // SETUP
    const marginAccounts = await MarginAccount.loadAllByOwner({
      programs,
      provider: provider_a,
      owner: provider_a.wallet.publicKey
    })

    // TEST
    expect(marginAccounts.length).to.eq(1)
    expect(marginAccounts.find(acc => acc.seed === 0)?.seed).to.eq(marginAccount_A.seed)
  })

  it("Close margin accounts", async () => {
    await marginAccount_A.closeAccount()
    await marginAccount_B.closeAccount()
  })
})
