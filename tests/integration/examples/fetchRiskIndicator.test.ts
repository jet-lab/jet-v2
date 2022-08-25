import { MarginAccount, MarginClient, PoolManager } from "../../../libraries/ts/src"
import { Connection } from "@solana/web3.js"
import { AnchorProvider, Wallet } from "@project-serum/anchor"
import { DEFAULT_CONFIRM_OPTS } from "../util"

// Examples of loading margin accounts and getting a margin account's risk indicator

describe("Typescript examples", async () => {
  it("Fetch risk indicator for local wallet", async () => {
    const config = await MarginClient.getConfig("devnet")
    const connection = new Connection("https://api.devnet.solana.com", DEFAULT_CONFIRM_OPTS.commitment)
    const wallet = Wallet.local()
    const localWalletPubkey = wallet.publicKey
    const provider = new AnchorProvider(connection, wallet, DEFAULT_CONFIRM_OPTS)

    const programs = MarginClient.getPrograms(provider, config)

    const poolManager = new PoolManager(programs, provider)
    //Load margin pools
    const pools = await poolManager.loadAll()

    //Load wallet tokens
    const walletTokens = await MarginAccount.loadTokens(poolManager.programs, localWalletPubkey)

    //Load all margin accounts - users can have multiple margin accounts eventually
    const marginAccounts = await MarginAccount.loadAllByOwner({
      programs: poolManager.programs,
      provider: poolManager.provider,
      pools,
      walletTokens,
      owner: localWalletPubkey
    })

    //Print risk level of a margin account
    if (marginAccounts) {
      console.log(`Public key ${localWalletPubkey} risk indicator is ${marginAccounts[0].riskIndicator}`)
    } else {
      console.log("We have trouble getting margin accounts")
    }
  })

  it("Fetch risk indicator for wallet `6XEn2q37nqsYQB5R79nueGi6n3uhgjiDwxoJeAVzWvaS`", async () => {
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
      console.log(`Public key ${walletPublicKey} risk indicator is ${marginAccounts[0].riskIndicator}`)
    } else {
      console.log("We have trouble getting margin accounts")
    }
  })
})
