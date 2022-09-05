import * as anchor from "@project-serum/anchor"
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddress } from "@solana/spl-token"
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, Transaction } from "@solana/web3.js"
import { BN } from "bn.js"
import { assert } from "chai"
import { BondMarket } from "../../../libraries/ts-bonds/src"
import { AssetKindTicket, AssetKindToken, BondsUser } from "../../libraries/ts/src/bondsUser"
import { JetBonds, JetBondsIdl } from "../../libraries/ts/src/types"
import { calculate_limit_price } from "../../libraries/ts/wasm-utils/pkg/wasm_utils"
import CONFIG from "./config.json"
import TEST_MINT_KEYPAIR from "../deps/keypairs/test_mint-keypair.json"
import { TestMint } from "./utils"

describe("jet-bonds", async () => {
  const confirmOptions: ConfirmOptions = {
    skipPreflight: true,
    commitment: "confirmed"
  }
  const connection = new Connection("http://localhost:8899", "confirmed")
  const payer = Keypair.generate()
  const wallet = new anchor.Wallet(payer)
  const provider = new anchor.AnchorProvider(connection, wallet, confirmOptions)
  anchor.setProvider(provider)

  let bondsProgram: anchor.Program<JetBonds>
  let testMintAuthority: Keypair

  interface TestUser {
    wallet?: Keypair
    tokenAcocunt?: PublicKey
    userAccount?: BondsUser
  }

  const TOKEN_DECIMALS = 6
  const ONE_TOKEN = 10 ** TOKEN_DECIMALS
  let testMint: TestMint

  let bob: TestUser = {}
  let alice: TestUser = {}

  before(async () => {
    bondsProgram = new anchor.Program(JetBondsIdl, CONFIG.addresses.jetBondsPid, provider)
    bob.wallet = Keypair.generate()
    alice.wallet = Keypair.generate()

    const airdropPayer = await provider.connection.requestAirdrop(payer.publicKey, 300 * LAMPORTS_PER_SOL)
    const airdropBob = await provider.connection.requestAirdrop(bob.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    const airdropAlice = await provider.connection.requestAirdrop(bob.wallet.publicKey, 300 * LAMPORTS_PER_SOL)
    const signatures = [airdropPayer, airdropBob, airdropAlice]
    for (let i in signatures) {
      await provider.connection.confirmTransaction(signatures[i])
    }

    testMintAuthority = Keypair.fromSecretKey(Uint8Array.of(...TEST_MINT_KEYPAIR))
    testMint = new TestMint(TOKEN_DECIMALS, testMintAuthority, provider)
    bob.tokenAcocunt = await testMint.createAndMintTo(10 ** 18, bob.wallet.publicKey, payer)
    alice.tokenAcocunt = await testMint.createAndMintTo(10 ** 18, alice.wallet!.publicKey, payer)
  })

  let bondMarket: BondMarket

  it("bondMarket is loaded", async () => {
    bondMarket = await BondMarket.load(bondsProgram, CONFIG.addresses.bondManager)
    assert(bondMarket.address.toBase58() === CONFIG.addresses.bondManager)
  })

  it("makes user accounts", async () => {
    bob.userAccount = await bondMarket.createBondsUser({
      user: bob.wallet!,
      payer,
      opts: confirmOptions
    })
    alice.userAccount = await bondMarket.createBondsUser({
      user: alice.wallet!,
      payer,
      opts: confirmOptions
    })
  })

  it("alice mints bond tickets", async () => {
    // create alice ticket account
    const ticketAddress = await getAssociatedTokenAddress(bondMarket.addresses.bondTicketMint, alice.wallet!.publicKey)
    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        ticketAddress,
        alice.wallet!.publicKey,
        bondMarket.addresses.bondTicketMint
      )
    )
    await provider.connection.confirmTransaction(await provider.sendAndConfirm(transaction, [payer]))

    // exchange for some tickets
    await bondMarket.exchangeTokensForTickets({
      amount: new BN(10 ** 12),
      user: alice.wallet!,
      payer,
      opts: confirmOptions
    })

    await alice.userAccount!.refresh()
  })

  it("users deposit", async () => {
    await bob.userAccount!.deposit({
      amount: new anchor.BN(10_000_000),
      payer,
      tokensOrTickets: AssetKindToken,
      opts: confirmOptions
    })

    await alice.userAccount!.deposit({
      amount: new anchor.BN(1_000_000),
      payer,
      tokensOrTickets: AssetKindTicket,
      opts: confirmOptions
    })

    await bob.userAccount!.refresh()
    await alice.userAccount!.refresh()

    assert(bob.userAccount!.storedInfo.underlyingTokenStored.toNumber() == 10_000_000, "Bob's balance is off")
    assert(alice.userAccount!.storedInfo.bondTicketsStored.toNumber() == 1_000_000, "Alice doesn't have her tickets")
  })

  it("bob makes a lend offer", async () => {
    await bob.userAccount!.lend({
      amount: new BN(1_000_000),
      interest: new BN(1_000),
      opts: confirmOptions
    })
  })

  it("alice makes a borrow offer", async () => {
    await alice.userAccount!.borrow({
      amount: new BN(100_000),
      interest: new BN(1_100),
      opts: confirmOptions
    })
  })

  it("fetches orderbook", async () => {
    const orderbook = await bondMarket.fetchOrderbook()

    console.log(orderbook.asks)
    console.log(orderbook.bids)
    // TODO assert
  })

  // TODO wait for crank to clear event queue

  // it("users withdraw", async () => {
  //   await bob.userAccount!.withdraw({
  //     amount: new BN(9_000_000),
  //     payer,
  //     tokensOrTickets: AssetKindToken,
  //     opts: confirmOptions,
  //   });

  //   await bob.userAccount!.refresh();

  //   assert(bob.userAccount!.storedInfo.underlyingTokenStored.toNumber() == 0);
  // });
})
