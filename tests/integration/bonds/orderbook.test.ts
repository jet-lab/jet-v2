import * as anchor from "@project-serum/anchor";
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  getAccount as getTokenAccount,
  mintTo,
} from "@solana/spl-token";
import {
  ConfirmOptions,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { BN } from "bn.js";
import { assert } from "chai";
import { BondMarket, calculateOrderAmount } from "../../libraries/ts/src";
import { BondsUser } from "../../libraries/ts/src/bondsUser";
import { JetBonds, JetBondsIdl } from "../../libraries/ts/src/types";
import CONFIG from "../../config.json";
import TEST_MINT_KEYPAIR from "../deps/keypairs/test_mint-keypair.json";
import ALICE_KEYPAIR from "../deps/keypairs/alice-keypair.json";
import BOB_KEYPAIR from "../deps/keypairs/bob-keypair.json";
import { TestMint, Transactor } from "./utils";

describe("jet-bonds", async () => {
  const confirmOptions: ConfirmOptions = {
    skipPreflight: true,
    commitment: "confirmed",
  };
  const connection = new Connection("http://localhost:8899", "confirmed");
  const payer = Keypair.generate();
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    confirmOptions
  );
  anchor.setProvider(provider);

  let bondsProgram: anchor.Program<JetBonds>;
  let transactor: Transactor;
  let testMintAuthority: Keypair;

  interface TestUser {
    wallet: Keypair;
    key: PublicKey;
    tokenAccount: PublicKey;
    userAccount: BondsUser;
  }

  const TOKEN_DECIMALS = 6;
  const SOL_AMOUNT = 300 * LAMPORTS_PER_SOL;
  const ONE_TOKEN = 10 ** TOKEN_DECIMALS;
  const STARTING_TOKENS = 10 ** 9 * ONE_TOKEN;
  let testMint: TestMint;

  let bob: TestUser;
  let alice: TestUser;

  const airdrop = async (key) => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(key, SOL_AMOUNT)
    );
  };
  const createFundedUser = async (wallet) => {
    const key = wallet.publicKey;
    await airdrop(key);

    const tokenAccount = await testMint.createAndMintTo(
      STARTING_TOKENS,
      key,
      payer
    );
    const userAccount = await BondsUser.load(bondMarket, key);

    return {
      wallet,
      key,
      tokenAccount,
      userAccount,
    } as TestUser;
  };

  before(async () => {
    bondsProgram = new anchor.Program(
      JetBondsIdl,
      CONFIG.jetBondsPid,
      provider
    );

    await airdrop(payer.publicKey);

    testMintAuthority = Keypair.fromSecretKey(
      Uint8Array.of(...TEST_MINT_KEYPAIR)
    );
    testMint = new TestMint(TOKEN_DECIMALS, testMintAuthority, provider);

    transactor = new Transactor([payer], provider);
  });

  let bondMarket: BondMarket;

  const getTicketAddress = async (testUser: TestUser) => {
    return await getAssociatedTokenAddress(
      bondMarket.addresses.bondTicketMint,
      testUser.key
    );
  };
  const createTicketAccount = async (testUser) => {
    const address = await getTicketAddress(testUser);
    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        address,
        testUser.key!,
        bondMarket.addresses.bondTicketMint
      )
    );
    await provider.connection.confirmTransaction(
      await provider.sendAndConfirm(transaction, [payer])
    );
  };

  it("bondMarket is loaded", async () => {
    bondMarket = await BondMarket.load(bondsProgram, CONFIG.bondManager);
    assert(bondMarket.address.toBase58() === CONFIG.bondManager);
  });

  const fetchTokenAccount = async (key, mint) => {
    const address = await getAssociatedTokenAddress(mint, key);

    const tokenAccount = await getTokenAccount(provider.connection, address);

    return tokenAccount;
  };
  const userTokens = async (testUser) => {
    const account = await fetchTokenAccount(testUser.key, testMint.address);
    return new BN(account.amount.toString());
  };
  const userTickets = async (testUser) => {
    const account = await fetchTokenAccount(
      testUser.key,
      bondMarket.addresses.bondTicketMint
    );
    return new BN(account.amount.toString());
  };

  it("bonds users are loaded", async () => {
    bob = await createFundedUser(
      Keypair.fromSecretKey(Uint8Array.of(...BOB_KEYPAIR))
    );
    alice = await createFundedUser(
      Keypair.fromSecretKey(Uint8Array.of(...ALICE_KEYPAIR))
    );

    transactor.addSigner(bob.wallet);
    transactor.addSigner(alice.wallet);
  });

  const TOKENS_EXCHANGED = new BN(10 ** 6 * ONE_TOKEN);
  it("alice mints bond tickets", async () => {
    // create alice ticket account
    await createTicketAccount(alice);

    // exchange for some tickets
    const exchange = await alice.userAccount.exchangeTokensForTicketsIx(
      TOKENS_EXCHANGED
    );
    await transactor.signSendInstructions([exchange], confirmOptions);

    const resultingTokens = await userTokens(alice);
    const resultingTickets = await userTickets(alice);

    assert(
      resultingTokens.toString() ===
        new BN(STARTING_TOKENS).sub(TOKENS_EXCHANGED).toString()
    );
    assert(
      resultingTickets.toString() ===
        new BN(TOKENS_EXCHANGED.toNumber()).toString()
    );
  });

  const TICKET_SEED = Uint8Array.from([0]);
  const STAKE_AMOUNT = new BN(1_000 * ONE_TOKEN);
  it("alice stakes some tickets", async () => {
    let stake = await bondMarket.stakeTicketsIx({
      amount: STAKE_AMOUNT,
      seed: TICKET_SEED,
      user: alice.key,
    });
    await transactor.signSendInstructions([stake], confirmOptions);

    const resultingTickets = await userTickets(alice);
    const claimTicket = await alice.userAccount.loadClaimTicket(TICKET_SEED);

    assert(
      resultingTickets.toString() ===
        TOKENS_EXCHANGED.sub(STAKE_AMOUNT).toString()
    );
    assert(claimTicket.redeemable.toString() === STAKE_AMOUNT.toString());
  });

  const BORROW_AMOUNT = new BN(1_000 * ONE_TOKEN);
  const BORROW_INTEREST = new BN(1_500);
  it("alice makes a borrow offer", async () => {
    const borrow = await bondMarket.sellTicketsOrderIx({
      amount: BORROW_AMOUNT,
      interest: BORROW_INTEREST, // 15% interest
      vaultAuthority: alice.key!,
    });
    await transactor.signSendInstructions([borrow], confirmOptions);
  });

  const BORROW_ORDER_AMOUNT = calculateOrderAmount(
    BORROW_AMOUNT,
    BORROW_INTEREST
  );
  it("loads orderbook and asserts borrow order", async () => {
    const orderbook = await bondMarket.fetchOrderbook();
    const order = orderbook.asks[0];

    assert(new PublicKey(order.owner).toString() === alice.key.toString());
    assert(
      new BN(order.base_size.toString()).toString() ===
        BORROW_ORDER_AMOUNT.base.toString()
    );
    assert(
      new BN(order.price.toString()).toString() ===
        BORROW_ORDER_AMOUNT.price.toString()
    );
    // posted quote cannot be directly compared with the quote value in the OrderAmount
  });

  it("bob makes a lend offer", async () => {
    await createTicketAccount(bob);

    const lend = await bondMarket.lendOrderIx({
      amount: new BN(10_000 * ONE_TOKEN),
      interest: new BN(1_000), // 10% interest
      seed: Uint8Array.of(0),
      vaultAuthority: bob.key,
      payer: payer.publicKey,
    });
    await transactor.signSendInstructions([lend], confirmOptions);

    // TODO assert order validity
  });
});
