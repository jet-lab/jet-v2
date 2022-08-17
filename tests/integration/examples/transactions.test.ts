/*

import { MarginClient, MarginPrograms } from "../../../libraries/ts/src/margin/marginClient"
import { MarginAccount } from "../../../libraries/ts/src/margin/marginAccount"
import { Pool, PoolManager, PoolTokenChange } from "../../../libraries/ts/src/margin/pool"
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"
import { AnchorProvider, BN, Wallet } from "@project-serum/anchor"
import { MarginConfig, PositionKind, TokenFaucet, TokenFormat } from "../../../libraries/ts/src"
import { assert } from "chai"

//An example of loading margin accounts and getting a margin account's risk indicator


describe("Typescript examples", () => {
  const walletKepair = Keypair.generate()
  const walletPubkey = walletKepair.publicKey

  const options: ConfirmOptions = { commitment: "recent", skipPreflight: true }
  const connection = new Connection("https://api.devnet.solana.com", options.commitment)
  const wallet = new Wallet(walletKepair)
  const provider = new AnchorProvider(connection, wallet, options)

  let config: MarginConfig
  let programs: MarginPrograms
  let poolManager: PoolManager
  let pools: Record<string, Pool>

  let marginAccount: MarginAccount

  describe("Margin account transactions", () => {
    it("Initialize the margin account", async () => {
      // Load programs
      config = await MarginClient.getConfig("devnet")
      programs = MarginClient.getPrograms(provider, config)

      // Load margin pools
      poolManager = new PoolManager(programs, provider)
      pools = await poolManager.loadAll()

      // Airdrop
      const solPool = pools["SOL"].tokenConfig
      const usdcPool = pools["USDC"].tokenConfig
      assert(solPool)
      assert(usdcPool)
      await TokenFaucet.airdrop(programs, provider, new BN(LAMPORTS_PER_SOL), solPool)
      await TokenFaucet.airdrop(programs, provider, new BN(1_000_000_000), usdcPool)

      marginAccount = await MarginAccount.createAccount({
        programs,
        provider,
        owner: walletPubkey,
        seed: 0,
        pools
      })

      //Print the margin account pubkey
      console.log(`Created margin account ${marginAccount.address}`)
      // Created margin account 2BnWcRxGQBXcRuFCfgePdWPbtpR5FYc4gu5C3a8gNJDc
    })

    it("Deposit user funds into their margin accounts", async () => {
      const txid = await pools["SOL"].deposit({
        marginAccount,
        change: PoolTokenChange.shiftBy(0.1 * LAMPORTS_PER_SOL)
      })

      await marginAccount.refresh()
      const position = marginAccount.positions[0]

      console.log(`Deposited SOL.. ${txid}`)
      console.log(`Balance ${position.balance.toNumber() / LAMPORTS_PER_SOL}`)
      console.log(`Price $${position.price.toNumber()}`)
      console.log(`Value $${position.value.toFixed(2)}`)
      console.log(`Is Deposit? ${position.kind === PositionKind.Deposit}`)
      console.log(``)
    })

    it("Borrow tokens in a margin account", async () => {
      const txid = await pools["USDC"].marginBorrow({
        marginAccount,
        pools,
        change: PoolTokenChange.shiftBy(1_000_000)
      })

      await marginAccount.refresh()
      const position = marginAccount.positions[1]

      console.log(`Borrowed USDC.. ${txid}`)
      console.log(`Loan Balance ${position.balance.toNumber() / 1_000_000}`)
      console.log(`Loan Price $${position.price.toNumber()}`)
      console.log(`Loan Value $${position.value.toFixed(2)}`)
      console.log(`Is Claim? ${position.kind === PositionKind.Claim}`)
      console.log(``)
    })

    it("Repay loans from margin account", async () => {
      const depositTxid = await pools["USDC"].deposit({ marginAccount, change: PoolTokenChange.shiftBy(1_000_000) })
      console.log(`Depositing USDC for repayment.. ${depositTxid}`)

      const repayTxid = await pools["USDC"].marginRepay({
        marginAccount,
        pools,
        change: PoolTokenChange.setTo(500_000)
      })

      await marginAccount.refresh()
      const position = marginAccount.positions[1]

      console.log(`Repayed USDC.. ${repayTxid}`)
      console.log(`Loan Balance ${position.balance.toNumber() / 1_000_000}`)
      console.log(`Loan Price $${position.price.toNumber()}`)
      console.log(`Loan Value $${position.value.toFixed(2)}`)
      console.log(`Is Claim? ${position.kind === PositionKind.Claim}`)
      console.log(``)
    })

    it("Repay with user wallet", async () => {
      const txid = await pools["USDC"].marginRepay({
        marginAccount,
        pools,
        change: PoolTokenChange.setTo(0),
        // Providing a source will repay from the wallet
        source: TokenFormat.unwrappedSol
      })

      await marginAccount.refresh()
      const position = marginAccount.positions[1]

      console.log(`Repayed USDC.. ${txid}`)
      console.log(`Loan Balance ${position.balance.toNumber() / 1_000_000}`)
      console.log(`Loan Price $${position.price.toNumber()}`)
      console.log(`Loan Value $${position.value.toFixed(2)}`)
      console.log(`Is Claim? ${position.kind === PositionKind.Claim}`)
      console.log(``)
    })

    it("Withdraw funds from margin account", async () => {
      const solTxid = await pools["SOL"].withdraw({ marginAccount, pools, change: PoolTokenChange.setTo(0) })
      const usdcTxid = await pools["USDC"].withdraw({ marginAccount, pools, change: PoolTokenChange.setTo(0) })

      await marginAccount.refresh()

      const solPosition = marginAccount.positions[0]
      console.log(`Withdrawing SOL.. ${solTxid}`)
      console.log(`SOL Balance ${solPosition.balance.toNumber() / LAMPORTS_PER_SOL}`)
      console.log(`SOL Price $${solPosition.price.toNumber()}`)
      console.log(`SOL Value $${solPosition.value.toFixed(2)}`)
      console.log(`Is Deposit? ${solPosition.kind === PositionKind.Deposit}`)
      console.log(``)

      const usdcPosition = marginAccount.positions[2]
      console.log(`Withdrawing USDC.. ${usdcTxid}`)
      console.log(`USDC Balance ${usdcPosition.balance.toNumber() / 1_000_000}`)
      console.log(`USDC Price $${usdcPosition.price.toNumber()}`)
      console.log(`USDC Value $${usdcPosition.value.toFixed(2)}`)
      console.log(`Is Deposit? ${usdcPosition.kind === PositionKind.Deposit}`)
    })

    it("Close margin account positions", async () => {
      for (const position of marginAccount.positions) {
        await marginAccount.closePosition(position)
      }

      await marginAccount.refresh()

      console.log(`Margin account positions closed..`)
      console.log(`Position count == ${marginAccount.positions.length}`)
    })

    it("Close margin account", async () => {
      await marginAccount.closeAccount()

      const exists = await MarginAccount.exists(programs, walletPubkey, 0)
      console.log(`Margin account closed..`)
      console.log(`Exists? ${exists}`)
    })
  })
})
*/
