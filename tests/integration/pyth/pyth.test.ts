import assert from 'assert'
import * as anchor from '@project-serum/anchor'
import { AnchorProvider } from '@project-serum/anchor'
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';

import { PythClient } from './pythClient'

describe('pyth-oracle', () => {

  const opts: ConfirmOptions = {
    preflightCommitment: "processed",
    commitment: "processed",
  };

  const connection = new Connection("http://localhost:8899", opts.preflightCommitment);

  const payer = Keypair.generate();
  const wallet = new NodeWallet(payer);

  const provider = new AnchorProvider(connection, wallet, opts);
	anchor.setProvider(provider);

  it("Fund payer", async () => {
    const airdropSignature = await connection.requestAirdrop(payer.publicKey, 300 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdropSignature);
  });

  const pythClient = new PythClient({ pythProgramId: "ASfdvRMCan2aoWtbDi5HLXhz2CFfgEkuDoxc57bJLKLX", url: "http://127.0.0.1:8899/" });

  it('initialize', async () => {
    const price = 50000
    const expo = -6
    const priceAccount = Keypair.generate();
    const confidence = price / 10;
    await pythClient.createPriceAccount(wallet.payer, priceAccount, price, confidence, expo)
    const feedData = await pythClient.getPythPrice(priceAccount.publicKey)
    assert.ok(feedData.price === price)
  })

  it('change feed price', async () => {
    const price = 50000
    const expo = -7
    const priceAccount = Keypair.generate();
    const confidence = price / 10;
    await pythClient.createPriceAccount(wallet.payer, priceAccount, price, confidence, expo)
    const feedDataBefore = await pythClient.getPythPrice(priceAccount.publicKey)
    assert.ok(feedDataBefore.price === price)
    assert.ok(feedDataBefore.exponent === expo)

    const newPrice = 55000
    await pythClient.setPythPrice(wallet.payer, priceAccount.publicKey, newPrice, confidence, expo)
    const feedDataAfter = await pythClient.getPythPrice(priceAccount.publicKey)
    assert.ok(feedDataAfter.price === newPrice)
    assert.ok(feedDataAfter.exponent === expo)
  })

})
