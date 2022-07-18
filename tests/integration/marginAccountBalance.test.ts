import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"
import { expect } from "chai"

import { bnToNumber, MarginAccount, MarginClient, MarginPools, Pool, PoolManager } from "../../libraries/ts/src"

describe("margin account", () => {
  const confirmOptions: ConfirmOptions = { skipPreflight: true, preflightCommitment: "recent", commitment: "recent" }

  const provider = AnchorProvider.local("https://mango.devnet.rpcpool.com", confirmOptions)
  anchor.setProvider(provider)

  const programs = MarginClient.getPrograms(provider, "devnet")
  let poolManager: PoolManager
  let pools: Record<MarginPools, Pool>
  let btcPool: Pool
  let ONE_BTC: number

  it("Fund payer", async () => {
    let airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 1 * LAMPORTS_PER_SOL)
    await provider.connection.confirmTransaction(airdropSignature)
  })

  let wallet_a: PublicKey
  let provider_a: AnchorProvider
  let marginAccount: MarginAccount

  it("Create user wallet", async () => {
    wallet_a = new PublicKey("DyR7iiyiVVVNEfJpNGSXDgrEW65w7sje6yEC4ybPZSNG")
  })

  it("Fetch pools", async () => {
    poolManager = new PoolManager(programs, provider)
    pools = await poolManager.loadAll()
    btcPool = pools.BTC
    ONE_BTC = 10 ** btcPool.decimals
  })

  it("Load margin account", async () => {
    anchor.setProvider(provider_a)
    marginAccount = await MarginAccount.load({
      programs,
      provider: provider_a,
      pools,
      owner: wallet_a,
      seed: 0
    })
  })

  it("Assert deposit balance", async () => {
    console.log(JSON.stringify(marginAccount.poolPositions.BTC.depositPosition, jsonReplacer, 2))
    console.log(pools.BTC.depositNoteExchangeRate().asNumber())
    expect(marginAccount.poolPositions.BTC.depositBalance.tokens).to.eq(100)
  })
})

const jsonReplacer = (key, value) => {
  if (value instanceof PublicKey) {
    return value.toBase58()
  }
  if (key === "_bn") {
    return parseInt(value, 16)
  }
  if (typeof key === "string" && (key.startsWith("_UNUSED_") || key.startsWith("_reserved"))) {
    return undefined
  }
  return value?.toJSON ? value.toJSON() : value
}
