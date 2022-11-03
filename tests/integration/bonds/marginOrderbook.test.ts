import { assert } from 'chai';
import * as anchor from '@project-serum/anchor';
import { AnchorProvider, BN } from '@project-serum/anchor';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { Keypair, LAMPORTS_PER_SOL, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';

import {
  MarginAccount,
  PoolTokenChange,
  MarginClient,
  Pool,
  MarginPoolConfigData,
  PoolManager,
  bnToBigInt
} from '@jet-lab/margin';

import { PythClient } from '../pyth/pythClient';
import {
  airdropToken,
  createAuthority,
  createTokenAccount,
  createUserWallet,
  DEFAULT_CONFIRM_OPTS,
  DEFAULT_MARGIN_CONFIG,
  MARGIN_POOL_PROGRAM_ID,
  registerAdapter,
  sendToken,
  TestToken
} from '../util';

import CONFIG from '../../../app/public/localnet.config.json';

import { BondMarket, JetBonds, JetBondsIdl, MarginUserInfo, rate_to_price } from '@jet-lab/jet-bonds-client';
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddress } from '@solana/spl-token';

describe('margin bonds borrowing', async () => {
  // SUITE SETUP
  const provider = AnchorProvider.local(undefined, DEFAULT_CONFIRM_OPTS);
  anchor.setProvider(provider);
  const payer = (provider.wallet as NodeWallet).payer;
  const ownerKeypair = payer;
  const programs = MarginClient.getPrograms(provider, DEFAULT_MARGIN_CONFIG);
  const manager = new PoolManager(programs, provider);
  let USDC: TestToken = null as never;
  let BTC: TestToken = null as never;

  let USDC_oracle: Keypair[];
  let ticket_oracle: Keypair[];
  let SOL_oracle: Keypair[];

  const pythClient = new PythClient({
    pythProgramId: 'FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL',
    url: 'http://127.0.0.1:8899/'
  });

  const ONE_USDC = 1_000_000;
  const ONE_BTC = 10 ** CONFIG.tokens.Bitcoin.decimals;
  const ONE_SOL: number = LAMPORTS_PER_SOL;

  const DEFAULT_POOL_CONFIG: MarginPoolConfigData = {
    borrowRate0: 10,
    borrowRate1: 20,
    borrowRate2: 30,
    borrowRate3: 40,
    utilizationRate1: 10,
    utilizationRate2: 20,
    managementFeeRate: 10,
    managementFeeCollectThreshold: new BN(2),
    flags: new BN(2) // ALLOW_LENDING
  };

  let marginPool_USDC: Pool;
  let marginPool_BTC: Pool;
  let pools: Pool[];

  let wallet_a: NodeWallet;
  let wallet_b: NodeWallet;
  let wallet_c: NodeWallet;

  let provider_a: AnchorProvider;
  let provider_b: AnchorProvider;
  let provider_c: AnchorProvider;

  let marginAccount_A: MarginAccount;
  let marginAccount_B: MarginAccount;
  let marginAccount_C: MarginAccount;

  let user_a_usdc_account: PublicKey;
  let user_a_BTC_account: PublicKey;
  let user_b_BTC_account: PublicKey;
  let user_b_usdc_account: PublicKey;
  let user_c_BTC_account: PublicKey;
  let user_c_usdc_account: PublicKey;

  const bondsProgram: anchor.Program<JetBonds> = new anchor.Program(JetBondsIdl, CONFIG.bondsProgramId, provider);
  let bondMarket: BondMarket;

  it('setup', async () => {
    // Fund payer
    const airdropSignature = await provider.connection.requestAirdrop(
      provider.wallet.publicKey,
      300 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);

    USDC = await airdropToken(provider, payer.publicKey, 'USDC');
    BTC = await airdropToken(provider, payer.publicKey, 'Bitcoin');

    // create authority
    await createAuthority(programs, provider);

    // register adapter
    await registerAdapter(programs, provider, payer, MARGIN_POOL_PROGRAM_ID, payer);

    // load pools
    marginPool_BTC = await manager.load({ tokenMint: BTC.mint, tokenConfig: BTC.tokenConfig });
    marginPool_USDC = await manager.load({ tokenMint: USDC.mint, tokenConfig: USDC.tokenConfig });
    pools = [marginPool_BTC, marginPool_USDC];

    // create user wallets
    wallet_a = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL);
    wallet_b = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL);
    wallet_c = await createUserWallet(provider, 10 * LAMPORTS_PER_SOL);

    provider_a = new AnchorProvider(provider.connection, wallet_a, DEFAULT_CONFIRM_OPTS);
    provider_b = new AnchorProvider(provider.connection, wallet_b, DEFAULT_CONFIRM_OPTS);
    provider_c = new AnchorProvider(provider.connection, wallet_c, DEFAULT_CONFIRM_OPTS);

    // create margin accounts
    anchor.setProvider(provider_a);
    marginAccount_A = await MarginAccount.load({
      programs,
      provider: provider_a,
      owner: provider_a.wallet.publicKey,
      seed: 0
    });
    await marginAccount_A.createAccount();

    anchor.setProvider(provider_b);
    marginAccount_B = await MarginAccount.load({
      programs,
      provider: provider_b,
      owner: provider_b.wallet.publicKey,
      seed: 0
    });
    await marginAccount_B.createAccount();

    anchor.setProvider(provider_c);
    marginAccount_C = await MarginAccount.load({
      programs,
      provider: provider_c,
      owner: provider_c.wallet.publicKey,
      seed: 0
    });
    await marginAccount_C.createAccount();

    // give users tokens

    // SETUP
    const payer_A: Keypair = Keypair.fromSecretKey((wallet_a as NodeWallet).payer.secretKey);
    user_a_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_a.publicKey, payer_A);
    user_a_BTC_account = await createTokenAccount(provider, BTC.mint, wallet_a.publicKey, payer_A);

    const payer_B: Keypair = Keypair.fromSecretKey((wallet_b as NodeWallet).payer.secretKey);
    user_b_BTC_account = await createTokenAccount(provider, BTC.mint, wallet_b.publicKey, payer_B);
    user_b_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_b.publicKey, payer_B);

    const payer_C: Keypair = Keypair.fromSecretKey((wallet_c as NodeWallet).payer.secretKey);
    user_c_BTC_account = await createTokenAccount(provider, BTC.mint, wallet_c.publicKey, payer_C);
    user_c_usdc_account = await createTokenAccount(provider, USDC.mint, wallet_c.publicKey, payer_C);

    // ACT
    await sendToken(
      provider,
      USDC.mint,
      500_000,
      USDC.tokenConfig.decimals,
      ownerKeypair,
      USDC.vault,
      user_a_usdc_account
    );
    await sendToken(provider, BTC.mint, 50, BTC.tokenConfig.decimals, ownerKeypair, BTC.vault, user_a_BTC_account);
    await sendToken(provider, BTC.mint, 500, BTC.tokenConfig.decimals, ownerKeypair, BTC.vault, user_b_BTC_account);
    await sendToken(provider, USDC.mint, 50, USDC.tokenConfig.decimals, ownerKeypair, USDC.vault, user_b_usdc_account);
    await sendToken(provider, BTC.mint, 1, BTC.tokenConfig.decimals, ownerKeypair, BTC.vault, user_c_BTC_account);
    await sendToken(provider, USDC.mint, 1, USDC.tokenConfig.decimals, ownerKeypair, USDC.vault, user_c_usdc_account);

    // refresh pools
    await marginPool_USDC.refresh();
    await marginPool_BTC.refresh();

    // deposit into margin accounts
    // ACT
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_A,
      source: user_a_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(500_000 * ONE_USDC))
    });
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_B,
      source: user_b_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(50 * ONE_USDC))
    });
    await marginPool_USDC.deposit({
      marginAccount: marginAccount_C,
      source: user_c_usdc_account,
      change: PoolTokenChange.shiftBy(new BN(ONE_USDC))
    });
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_A);
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_B);
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_C);

    await marginPool_BTC.deposit({
      marginAccount: marginAccount_A,
      source: user_a_BTC_account,
      change: PoolTokenChange.shiftBy(new BN(50 * ONE_BTC))
    });
    await marginPool_BTC.deposit({
      marginAccount: marginAccount_B,
      source: user_b_BTC_account,
      change: PoolTokenChange.shiftBy(new BN(500 * ONE_BTC))
    });
    await marginPool_BTC.deposit({
      marginAccount: marginAccount_C,
      source: user_c_BTC_account,
      change: PoolTokenChange.shiftBy(new BN(ONE_BTC))
    });
    await marginPool_BTC.marginRefreshPositionPrice(marginAccount_A);
    await marginPool_BTC.marginRefreshPositionPrice(marginAccount_B);
    await marginPool_BTC.marginRefreshPositionPrice(marginAccount_C);
    await marginAccount_A.refresh();
    await marginAccount_B.refresh();
    await marginAccount_C.refresh();

    // load the bond market
    bondMarket = await BondMarket.load(
      bondsProgram,
      CONFIG.airspaces[0].bondMarkets.USDC_86400.bondManager,
      CONFIG.marginProgramId
    );
  });

  const registerNewMarginUser = async (
    marginAccount: MarginAccount,
    bondMarket: BondMarket,
    payer: Keypair,
    provider: AnchorProvider
  ) => {
    const tokenAcc = await getAssociatedTokenAddress(
      bondMarket.addresses.underlyingTokenMint,
      marginAccount.address,
      true
    );
    const ticketAcc = await getAssociatedTokenAddress(bondMarket.addresses.bondTicketMint, marginAccount.address, true);
    await provider.sendAndConfirm(
      new Transaction()
        .add(
          createAssociatedTokenAccountInstruction(
            payer.publicKey,
            tokenAcc,
            marginAccount.address,
            bondMarket.addresses.underlyingTokenMint
          )
        )
        .add(
          createAssociatedTokenAccountInstruction(
            payer.publicKey,
            ticketAcc,
            marginAccount.address,
            bondMarket.addresses.bondTicketMint
          )
        ),
      [payer]
    );

    let ixs: TransactionInstruction[] = [
      await viaMargin(marginAccount, await bondMarket.registerAccountWithMarket(marginAccount, payer.publicKey)),
      await viaMargin(marginAccount, await bondMarket.refreshPosition(marginAccount, false))
    ];
    await provider.sendAndConfirm(new Transaction().add(...ixs), [payer]);
  };

  it('margin users create bond market accounts', async () => {
    assert(bondMarket);

    // register token wallets with margin accounts
    await registerNewMarginUser(marginAccount_A, bondMarket, wallet_a.payer, provider_a);
    await registerNewMarginUser(marginAccount_B, bondMarket, wallet_b.payer, provider_b);

    let borrower_a: MarginUserInfo = await bondMarket.fetchMarginUser(marginAccount_A);
    let borrower_b: MarginUserInfo = await bondMarket.fetchMarginUser(marginAccount_B);

    assert(borrower_a.bondManager.toBase58() === bondMarket.addresses.bondManager.toBase58());
    assert(borrower_b.marginAccount.toBase58() === marginAccount_B.address.toBase58());
  });

  const airdropMarginWallet = async (margin: MarginAccount, token: TestToken, amount: number) => {
    const tokenAcc = await getAssociatedTokenAddress(token.mint, margin.address, true);
    await sendToken(provider, token.mint, amount, token.tokenConfig.decimals, ownerKeypair, token.vault, tokenAcc);
  };

  const viaMargin = async (margin: MarginAccount, ix: TransactionInstruction): Promise<TransactionInstruction> => {
    let ixns = [];
    await margin.withAdapterInvoke({
      instructions: ixns,
      adapterInstruction: ix
    });
    return ixns[0];
  };

  const makeTx = (ix: TransactionInstruction[]) => {
    return new Transaction().add(...ix);
  };

  const loanOfferParams = {
    amount: new BN(1_500),
    rate: new BN(1_000)
  };

  it('margin users place lend orders', async () => {
    // give margin spl wallets some liquidity
    await airdropMarginWallet(marginAccount_A, USDC, 100_000);
    await airdropMarginWallet(marginAccount_B, USDC, 100_000);

    const lendNowA = await bondMarket.lendNowIx(
      marginAccount_A,
      new BN(1_000),
      wallet_a.payer.publicKey,
      Uint8Array.from([0, 0, 0, 0])
    );
    const offerLoanB = await bondMarket.offerLoanIx(
      marginAccount_B,
      loanOfferParams.amount,
      loanOfferParams.rate,
      wallet_b.payer.publicKey,
      Uint8Array.from([0, 0, 0, 0])
    );
    const invokeA = await viaMargin(marginAccount_A, lendNowA);
    const invokeB = await viaMargin(marginAccount_B, offerLoanB);

    await provider_a.sendAndConfirm(makeTx([invokeA]), [wallet_a.payer]);
    await provider_b.sendAndConfirm(makeTx([invokeB]), [wallet_b.payer]);
  });

  const borrowRequestParams = {
    amount: new BN(1000),
    rate: new BN(1000)
  };

  it('margin users place borrow orders', async () => {
    const borrowNowA = await bondMarket.borrowNowIx(
      marginAccount_A,
      wallet_a.payer.publicKey,
      new BN(1000),
      Uint8Array.from([0, 0, 0, 0])
    );
    const requestBorrowB = await bondMarket.requestBorrowIx(
      marginAccount_B,
      wallet_b.payer.publicKey,
      borrowRequestParams.amount,
      borrowRequestParams.rate,
      Uint8Array.from([0, 0, 0, 0])
    );

    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_A);
    await marginPool_USDC.marginRefreshPositionPrice(marginAccount_B);
    await marginPool_BTC.marginRefreshPositionPrice(marginAccount_A);
    await marginPool_BTC.marginRefreshPositionPrice(marginAccount_B);
    await marginAccount_A.refresh();
    await marginAccount_B.refresh();

    const refreshA = await viaMargin(marginAccount_A, await bondMarket.refreshPosition(marginAccount_A, false));
    const refreshB = await viaMargin(marginAccount_B, await bondMarket.refreshPosition(marginAccount_B, false));
    const invokeA = await viaMargin(marginAccount_A, borrowNowA);
    const invokeB = await viaMargin(marginAccount_B, requestBorrowB);

    // TODO: refresh position is not able to get a properly formatted price from the oracle
    await provider_a.sendAndConfirm(makeTx([refreshA, invokeA]), [wallet_a.payer]);
    await provider_b.sendAndConfirm(makeTx([refreshB, invokeB]), [wallet_b.payer]);
  });

  let loanId: Uint8Array;

  it('loads orderbook and has correct orders', async () => {
    const orderbook = await bondMarket.fetchOrderbook();
    const offeredLoan = orderbook.bids[0];
    // const requestedBorrow = orderbook.asks[0]

    loanId = offeredLoan.order_id;

    assert(
      offeredLoan.limit_price ===
        rate_to_price(bnToBigInt(loanOfferParams.rate), bnToBigInt(bondMarket.info.lendDuration))
    );

    // TODO this test requires utilization of the wasm math utils to properly derive order values to be found on the orderbook
    // assert(
    //   offeredLoan.quote_size === loanQuote,
    //   'Quote amount does not match given params, Given: [' + loanQuote + ']; On book: [' + offeredLoan.quote_size + ']'
    // );

    // TODO this requires borrow orders to properly post
    // assert(requestedBorrow.quote_size === bnToBigInt(borrowRequestParams.amount))
    // assert(
    //   requestedBorrow.limit_price ===
    //     rate_to_price(bnToBigInt(borrowRequestParams.rate), bnToBigInt(bondMarket.info.duration))
    // )
  });

  it('margin users cancel orders', async () => {
    const cancelLoan = await bondMarket.cancelOrderIx(marginAccount_B, loanId);
    const invokeCancelLoan = await viaMargin(marginAccount_B, cancelLoan);

    await provider_b.sendAndConfirm(makeTx([invokeCancelLoan]));
  });
});
