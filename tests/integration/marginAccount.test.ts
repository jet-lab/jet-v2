import { assert } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"

import { MarginAccount, MarginClient } from "../../libraries/ts/src"
import { createAuthority, createUserWallet } from "./util"

describe("margin account", () => {
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }

  const provider = AnchorProvider.local(undefined, confirmOptions)
  anchor.setProvider(provider)

  const payer: Keypair = (provider.wallet as NodeWallet).payer

  const programs = MarginClient.getPrograms(provider, "localnet")

  it("Fund payer", async () => {
    const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  let wallet_a: NodeWallet
  let wallet_b: NodeWallet

  let provider_a: AnchorProvider
  let provider_b: AnchorProvider

  it("Create two user wallets", async () => {
    // Create our two user wallets, with some SOL funding to get started
    wallet_a = await createUserWallet(provider.connection, 10 * LAMPORTS_PER_SOL)
    wallet_b = await createUserWallet(provider.connection, 10 * LAMPORTS_PER_SOL)
    provider_a = new AnchorProvider(provider.connection, wallet_a, confirmOptions)
    provider_b = new AnchorProvider(provider.connection, wallet_b, confirmOptions)
  })

  it("Create authority", async () => {
    await createAuthority(provider.connection, payer)
  })

  it("Create margin accounts", async () => {
    // Initialize the margin accounts for each user
    anchor.setProvider(provider_a)
    const maginAccount_A: MarginAccount = await MarginAccount.load(programs, provider_a, provider_a.wallet.publicKey, 0)
    await maginAccount_A.createAccount()

    anchor.setProvider(provider_b)
    const maginAccount_B: MarginAccount = await MarginAccount.load(programs, provider_b, provider_b.wallet.publicKey, 0)
    await maginAccount_B.createAccount()
  })
})
