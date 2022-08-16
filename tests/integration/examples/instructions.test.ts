import { MarginClient, MarginPrograms } from "../../../libraries/ts/src/margin/marginClient"
import { MarginAccount } from "../../../libraries/ts/src/margin/marginAccount"
import { Pool, PoolManager, PoolTokenChange } from "../../../libraries/ts/src/margin/pool"
import { Connection, Keypair, LAMPORTS_PER_SOL, TransactionInstruction } from "@solana/web3.js"
import { AnchorProvider, Wallet } from "@project-serum/anchor"
import { MarginConfig, sleep } from "../../../libraries/ts/src"
import { assert } from "chai"

//An example of loading margin accounts and getting a margin account's risk indicator

describe("Typescript examples", () => {
  const walletKepair = Keypair.generate()
  const walletPubkey = walletKepair.publicKey

  const options = AnchorProvider.defaultOptions()
  const connection = new Connection("https://api.devnet.solana.com", options.commitment)
  const wallet = new Wallet(walletKepair)
  const provider = new AnchorProvider(connection, wallet, options)

  let config: MarginConfig
  let programs: MarginPrograms
  let poolManager: PoolManager
  let pools: Record<string, Pool>

  let marginAccount: MarginAccount

  const instructions: TransactionInstruction[] = []

  describe("Margin account instructions", () => {
    it("Setup", async () => {
      // Airdrop
      await connection.requestAirdrop(walletPubkey, LAMPORTS_PER_SOL)

      // Load programs
      config = await MarginClient.getConfig("devnet")
      programs = MarginClient.getPrograms(provider, config)

      // Load margin pools
      poolManager = new PoolManager(programs, provider)
      pools = await poolManager.loadAll()

      marginAccount = await MarginAccount.createAccount({
        programs,
        provider,
        owner: walletPubkey,
        seed: 0,
        pools
      })

      // Create a position for use later
      await pools["SOL"].deposit({ marginAccount, change: PoolTokenChange.shiftBy(0.01) })

      await marginAccount.refresh()

      //Print the margin account pubkey
      console.log(`Created margin account ${marginAccount.address}`)
      // Created margin account 2BnWcRxGQBXcRuFCfgePdWPbtpR5FYc4gu5C3a8gNJDc
    })

    it("Create a new margin account for a user", async () => {
      await marginAccount.withCreateAccount(instructions)
    })

    it("Close a user's margin account", async () => {
      await marginAccount.withCloseAccount(instructions)
    })

    it("Register a position for some token that will be custodied by margin.", async () => {
      const depositNote = pools["SOL"].addresses.depositNoteMint
      await marginAccount.withRegisterPosition(instructions, depositNote)
    })

    it("Update the balance of a position stored in the margin account to match the actual balance stored by the SPL token account.", async () => {
      // Two ways to derive the position

      // Method 1, derive it
      const positionMint = pools["SOL"].addresses.depositNoteMint
      const position_A = marginAccount.findPositionTokenAddress(positionMint)

      // Method 2, fish it from an existing position
      const position_B = marginAccount.positions[0]?.address

      // Avoid RPC rate limiting
      await sleep(3000)

      await marginAccount.withUpdatePositionBalance({ instructions, position: position_A })
    })

    it("Close out a position, freeing up space in the account.", async () => {
      // Method 2, fish it from an existing position
      const position = marginAccount.positions[0]
      assert(position)
      await marginAccount.withClosePosition(instructions, position)
    })

    it("Update the metadata for a position stored in the margin account", async () => {
      // Method 2, fish it from an existing position
      const positionMint = marginAccount.positions[0]?.token
      assert(positionMint)
      await marginAccount.withRefreshPositionMetadata({ instructions, positionMint })
    })
  })
})
