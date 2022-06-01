import { assert } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider, Provider } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import {
  Account,
  ConfirmOptions,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram
} from "@solana/web3.js"

import { MarginAccount, MarginClient } from "../../libraries/ts"
import { createAuthority, createUserWallet } from "./util"

describe("margin account", () => {
  const opts: ConfirmOptions = {
    preflightCommitment: "processed",
    commitment: "processed"
  }
  const connection = new Connection("http://localhost:8899", opts.preflightCommitment)

  const payer = Keypair.generate()
  const wallet = new NodeWallet(payer)

  const provider = new AnchorProvider(connection, wallet, opts)
  anchor.setProvider(provider)

  const programs = MarginClient.getPrograms(provider, "localnet")

  it("Fund payer", async () => {
    const airdropSignature = await connection.requestAirdrop(payer.publicKey, 300 * LAMPORTS_PER_SOL)
    await connection.confirmTransaction(airdropSignature)
  })

  let wallet_a: NodeWallet
  let wallet_b: NodeWallet

  let provider_a: AnchorProvider
  let provider_b: AnchorProvider

  it("Create two user wallets", async () => {
    // Create our two user wallets, with some SOL funding to get started
    wallet_a = await createUserWallet(connection, 10 * LAMPORTS_PER_SOL)
    wallet_b = await createUserWallet(connection, 10 * LAMPORTS_PER_SOL)
    provider_a = new AnchorProvider(connection, wallet_a, opts)
    provider_b = new AnchorProvider(connection, wallet_b, opts)
  })

  it("Create authority", async () => {
    await createAuthority(connection, payer)
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
