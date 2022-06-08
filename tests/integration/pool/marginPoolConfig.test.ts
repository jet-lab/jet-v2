import { assert } from "chai"
import * as anchor from "@project-serum/anchor"
import { AnchorProvider } from "@project-serum/anchor"
import { AccountInfo, ConfirmOptions, PublicKey } from "@solana/web3.js"

import { MarginClient, MarginPool, MarginTokens } from "../../../libraries/ts/src"

describe("margin pool devnet config", () => {
  const config = MarginClient.getConfig("devnet")
  const confirmOptions: ConfirmOptions = { preflightCommitment: "processed", commitment: "processed" }
  const provider = AnchorProvider.local("https://mango.devnet.rpcpool.com/", confirmOptions)
  const { connection } = provider
  anchor.setProvider(provider)

  const programs = MarginClient.getPrograms(provider, config)
  let pools: Record<MarginTokens, MarginPool>

  const TEST_TOKENS = ["USDC", "SOL"]

  it("Load pools", async () => {
    pools = await MarginPool.loadAll(programs)
  })

  it("Pools exists", async () => {
    TEST_TOKENS.map(token => {
      const pool = pools[token]
      assert.isDefined(pool)
      console.log("POOL INFO: ", pool.info)
      // assert(pool.info != null, `${token} info is not defined`)
    })
  })

  it("Pool accounts exist", async () => {
    TEST_TOKENS.map(async token => {
      const pool = pools[token]
      const addresses: [string, PublicKey][] = Object.entries(pool.addresses)
      const accounts: (AccountInfo<Buffer> | null)[] = await connection.getMultipleAccountsInfo(
        addresses.map(([_, pubkey]) => pubkey)
      )
      for (let i = 0; i < addresses.length; i++) {
        const account = accounts[i]
        const [name] = addresses[i]
        assert(account, `Account ${name} in pool ${pool.tokenConfig.symbol} does not exist`)
      }
    })
  })
})
