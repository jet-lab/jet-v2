import { MarginClient } from "../../../libraries/ts/src/margin/marginClient"
import { MarginAccount } from "../../../libraries/ts/src/margin/marginAccount"
import { PoolManager } from "../../../libraries/ts/src/margin/pool"
import { Connection } from "@solana/web3.js"
import { AnchorProvider, Wallet } from "@project-serum/anchor"

//An example of loading margin accounts and getting a margin account's risk indicator

describe("Typescript examples", async () => {
  it("Fetch risk indicator", async () => {
    const walletPublicKey = "6XEn2q37nqsYQB5R79nueGi6n3uhgjiDwxoJeAVzWvaS"
    const config = await MarginClient.getConfig("devnet")
    const connection = new Connection("https://api.devnet.solana.com", "recent")
    const options = AnchorProvider.defaultOptions()
    const wallet = Wallet.local()
    const provider = new AnchorProvider(connection, wallet, options)

    const programs = MarginClient.getPrograms(provider, config)

    const poolManager = new PoolManager(programs, provider)
    //Load margin pools
    const pools = await poolManager.loadAll()

    //Load wallet tokens
    const walletTokens = await MarginAccount.loadTokens(poolManager.programs, walletPublicKey)

    //Load all margin accounts - users can have multiple margin accounts eventually
    const marginAccounts = await MarginAccount.loadAllByOwner({
      programs: poolManager.programs,
      provider: poolManager.provider,
      pools,
      walletTokens,
      owner: walletPublicKey
    })

    //Print risk level of a margin account
    if (marginAccounts) {
      console.log(
        `Public key 6XEn2q37nqsYQB5R79nueGi6n3uhgjiDwxoJeAVzWvaS risk indicator is ${marginAccounts[0].riskIndicator}`
      )
    } else {
      console.log("We have trouble getting margin accounts")
    }
  })
})
