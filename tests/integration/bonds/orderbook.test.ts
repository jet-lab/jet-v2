import * as anchor from "@project-serum/anchor"
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  getAccount as getTokenAccount
} from "@solana/spl-token"
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, Transaction } from "@solana/web3.js"
import { BN } from "bn.js"
import { assert } from "chai"
import { BondMarket, calculateOrderAmount } from "../../../libraries/ts-bonds/src"
import { BondsUser } from "../../../libraries/ts-bonds/src"
import { JetBonds, JetBondsIdl } from "../../../libraries/ts-bonds/src"
import CONFIG from "./config.json"
import TEST_MINT_KEYPAIR from "../../deps/keypairs/test_mint-keypair.json"
import { TestMint, Transactor } from "./utils"

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
  let transactor: Transactor
  let testMintAuthority: Keypair

  interface TestUser {
    wallet: Keypair
    key: PublicKey
    tokenAccount: PublicKey
    userAccount: BondsUser
  }

  const TOKEN_DECIMALS = 6
  const SOL_AMOUNT = 300 * LAMPORTS_PER_SOL
  const ONE_TOKEN = 10 ** TOKEN_DECIMALS
  const STARTING_TOKENS = 10 ** 9 * ONE_TOKEN
  let testMint: TestMint

  let bob: TestUser
  let alice: TestUser

  const airdrop = async key => {
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(key, SOL_AMOUNT))
  }
  const createFundedUser = async () => {
    const wallet = Keypair.generate()
    const key = wallet.publicKey
    await airdrop(key)

    const tokenAccount = await testMint.createAndMintTo(STARTING_TOKENS, key, payer)
    const userAccount = await BondsUser.load(bondMarket, key)

    return {
      wallet,
      key,
      tokenAccount,
      userAccount
    } as TestUser
  }

  before(async () => {
    bondsProgram = new anchor.Program(JetBondsIdl, CONFIG.addresses.jetBondsPid, provider)

    await airdrop(payer.publicKey)

    testMintAuthority = Keypair.fromSecretKey(Uint8Array.of(...TEST_MINT_KEYPAIR))
    testMint = new TestMint(TOKEN_DECIMALS, testMintAuthority, provider)

    transactor = new Transactor([payer], provider)
  })

  let bondMarket: BondMarket

  const getTicketAddress = async testUser => {
    return await getAssociatedTokenAddress(testUser.key!, bondMarket.addresses.bondTicketMint)
  }
  const createTicketAccount = async testUser => {
    const address = await getTicketAddress(testUser)
    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        address,
        testUser.key!,
        bondMarket.addresses.bondTicketMint
      )
    )
    await provider.connection.confirmTransaction(await provider.sendAndConfirm(transaction, [payer]))
  }

  it("bondMarket is loaded", async () => {
    bondMarket = await BondMarket.load(bondsProgram, CONFIG.addresses.bondManager)
    assert(bondMarket.address.toBase58() === CONFIG.addresses.bondManager)
  })

  const START_TICKETS = new BN(10 ** 6 * ONE_TOKEN)
  const fetchTokenAccount = async (key, mint) => {
    const address = await getAssociatedTokenAddress(mint, key)

    const tokenAccount = await getTokenAccount(provider.connection, address)

    return tokenAccount
  }
  const userTokens = async testUser => {
    const account = await fetchTokenAccount(testUser.key, testMint.address)
    return new BN(account.amount.toString())
  }
  const userTickets = async testUser => {
    const account = await fetchTokenAccount(testUser.key, bondMarket.addresses.bondTicketMint)
    return new BN(account.amount.toString())
  }

  it("bonds users are loaded", async () => {
    bob = await createFundedUser()
    alice = await createFundedUser()

    transactor.addSigner(bob.wallet)
    transactor.addSigner(alice.wallet)
  })

  it("alice mints bond tickets", async () => {
    // create alice ticket account
    await createTicketAccount(alice)

    // exchange for some tickets
    let exchange = await alice.userAccount.exchangeTokensForTicketsIx(START_TICKETS)

    await transactor.signSendInstructions([exchange], confirmOptions)

    const resultingTokens = await userTokens(alice)
    const resultingTickets = await userTickets(alice)

    assert(resultingTickets === START_TICKETS)
    assert(resultingTokens === new BN(STARTING_TOKENS).sub(START_TICKETS))
  })

  const TICKET_SEED = new BN(1337)
  const STAKE_AMOUNT = new BN(1_000 * ONE_TOKEN)
  it("alice stakes some tickets", async () => {
    let stake = await bondMarket.stakeTicketsIx({
      amount: STAKE_AMOUNT,
      seed: TICKET_SEED,
      user: alice.key
    })
    await transactor.signSendInstructions([stake], confirmOptions)

    const resultingTickets = await userTickets(alice)
    const claimTicket = await alice.userAccount.loadClaimTicket(TICKET_SEED)

    assert(resultingTickets === START_TICKETS.sub(STAKE_AMOUNT))
    assert(claimTicket.redeemable === STAKE_AMOUNT)
  })

  const BORROW_AMOUNT = new BN(1_000 * ONE_TOKEN)
  const BORROW_INTEREST = new BN(1_500)
  it("alice makes a borrow offer", async () => {
    const borrow = await bondMarket.borrowOrderIx({
      amount: BORROW_AMOUNT,
      interest: BORROW_INTEREST, // 15% interest
      vaultAuthority: alice.key!
    })
    await transactor.signSendInstructions([borrow], confirmOptions)
  })

  const BORROW_ORDER_AMOUNT = calculateOrderAmount(BORROW_AMOUNT, BORROW_INTEREST)
  it("loads orderbook and asserts borrow order", async () => {
    const orderbook = await bondMarket.fetchOrderbook()
    const order = orderbook.asks[0]

    assert(new PublicKey(order.account_key) === alice.key)
    assert(new BN(order.base_size.toString()) === BORROW_ORDER_AMOUNT.base)
    assert(new BN(order.price.toString()) === BORROW_ORDER_AMOUNT.price)
    // posted quote cannot be directly compared with the quote value in the OrderAmount
  })

  it("bob makes a lend offer", async () => {
    const lend = await bondMarket.lendOrderIx({
      amount: new BN(10_000 * ONE_TOKEN),
      interest: new BN(1_000), // 10% interest
      seed: new BN(0),
      payer: payer.publicKey
    })
    await transactor.signSendInstructions([lend], confirmOptions)

    // TODO assert order validity
  })
})
