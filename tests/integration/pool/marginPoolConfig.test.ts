import { expect } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import { AccountInfo, ConfirmOptions, PublicKey } from "@solana/web3.js"

import { bnToNumber, MarginClient, MarginPools, Pool, PoolManager } from "../../../libraries/ts/src"
import { getMintSupply, getTokenBalance } from "../util"

describe("margin pool devnet config", () => {
  const config = MarginClient.getConfig("devnet")
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }
  const provider = AnchorProvider.local("https://mango.devnet.rpcpool.com/", confirmOptions)
  anchor.setProvider(provider)

  const programs = MarginClient.getPrograms(provider, config)
  const manager = new PoolManager(programs, provider)
  let pools: Record<MarginPools, Pool>

  it("Load pools", async () => {
    pools = await manager.loadAll()
  })

  it("Pools exists", async () => {
    for (const pool of Object.values(pools)) {
      pool.refresh()
      expect(pool).to.exist
      expect(pool.info).to.exist
    }
  })

  it("Pool accounts exist", async () => {
    for (const pool of Object.values(pools)) {
      const addresses: [string, PublicKey][] = Object.entries(pool.addresses)
      const accounts: (AccountInfo<Buffer> | null)[] = await provider.connection.getMultipleAccountsInfo(
        addresses.map(([_, pubkey]) => pubkey)
      )
      for (let i = 0; i < addresses.length; i++) {
        const account = accounts[i]
        expect(account).to.exist
      }
    }
  })

  it("should should have a name", async () => {
    expect(pools.USDC.name).to.eq("USD Coin")
  })

  it("should have a symbol", async () => {
    expect(pools.USDC.symbol).to.eq("USDC")
  })

  it("should have a market size", async () => {
    const USDC = pools.USDC
    const supply = await getTokenBalance(provider, undefined, USDC.addresses.vault)
    expect(bnToNumber(USDC.marketSize.tokens)).to.eq(supply)
  })

  it("should have a deposit APY", async () => {
    expect(pools.USDC.depositApy).to.eq(0)
  })

  it("should have a borrow APR", async () => {
    expect(pools.USDC.borrowApr).to.not.eq(0)
  })

  it("should have a token price", async () => {
    expect(pools.USDC.tokenPrice).to.not.eq(0)
  })
})
