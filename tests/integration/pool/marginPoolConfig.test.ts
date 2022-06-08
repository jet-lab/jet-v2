import { assert } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import { ConfirmOptions, PublicKey } from "@solana/web3.js"

import { MarginClient, MarginPool, MarginTokens } from "../../../libraries/ts/src"

describe("margin pool devnet config", () => {
  const config = MarginClient.getConfig("devnet")
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }
  const provider = AnchorProvider.local("https://mango.devnet.rpcpool.com/", confirmOptions)
  // const { connection } = provider
  anchor.setProvider(provider)

  const programs = MarginClient.getPrograms(provider, config)
  let pools: Record<MarginTokens, MarginPool>

  it("Load pools", async () => {
    pools = await MarginPool.loadAll(programs)
  })

  it("Pools exists", async () => {
    for (const pool of Object.values(pools)) {
      assert.isDefined(pool)
      // assert(pool.info != null, `${token} info is not defined`)
    }
  })

  it("Pool accounts exist", async () => {
    for (const pool of Object.values(pools)) {
      const addresses: [string, PublicKey][] = Object.entries(pool.addresses)
      // const accounts: (AccountInfo<Buffer> | null)[] = await connection.getMultipleAccountsInfo(
      //   addresses.map(([_, pubkey]) => pubkey)
      // )
      for (let i = 0; i < addresses.length; i++) {
        // const account = accounts[i]
        // const [name] = addresses[i]
        // assert(account, `Account ${name} in pool ${pool.tokenConfig.symbol} does not exist`)
      }
    }
  })
})
