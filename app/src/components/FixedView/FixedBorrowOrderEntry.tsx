import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Button, Input, Typography } from 'antd';
import { FixedLendRowOrder } from '../../state/views/fixed-term';
import { FixedMarketAtom } from '../../state/fixed/fixed-term-market-sync';
import { CurrentAccount } from '../../state/user/accounts';
import { useMemo, useState } from 'react';
import BN from 'bn.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { Transaction, TransactionInstruction } from '@solana/web3.js';
import { MainConfig } from '../../state/config/marginConfig';
import { useProvider } from '../../utils/jet/provider';
import { Pools } from '../../state/pools/pools';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
// import { AssociatedToken } from '@jet-lab/margin';

function randomIntFromInterval(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1) + min);
}

export const FixedBorrowOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const pools = useRecoilValue(Pools);
  const wallet = useWallet();
  const config = useRecoilValue(MainConfig);
  const decimals = useMemo(() => {
    if (!config || !marketAndConfig) return 6;
    const token = Object.values(config.tokens).find(token => {
      return marketAndConfig.config.underlyingTokenMint === token.mint.toString();
    });
    return token.decimals;
  }, [config, marketAndConfig]);

  const [amount, setAmount] = useState(new BN(0));
  const [basisPoints, setBasisPoints] = useState(new BN(0));

  const { Paragraph } = Typography;

  const offerLoan = async () => {
    let ixns: TransactionInstruction[] = [];

    // const tokenMint = marketAndConfig.market.addresses.underlyingTokenMint;
    // const ticketMint = marketAndConfig.market.addresses.bondTicketMint;

    // await AssociatedToken.withCreate(ixns, provider, marginAccount.address, tokenMint);
    // await AssociatedToken.withCreate(ixns, provider, marginAccount.address, ticketMint);

    // AssociatedToken.withTransfer(ixns, tokenMint, wallet.publicKey, marginAccount.address, amount);

    const createAccountIx = await marketAndConfig.market.registerAccountWithMarket(marginAccount, wallet.publicKey);

    await marginAccount.withAdapterInvoke({
      instructions: ixns,
      adapterInstruction: createAccountIx
    });

    await provider
      .sendAndConfirm(new Transaction().add(...ixns))
      .then(result => {
        console.log('SUCCESS: ', result);
      })
      .catch(e => {
        console.log('ERROR: ', e);
      });

    await marginAccount.withUpdateAllPositionBalances({
      instructions: ixns
    });

    const borrowOffer = await marketAndConfig.market.requestBorrowIx(
      marginAccount,
      wallet.publicKey,
      amount,
      basisPoints,
      Uint8Array.from([
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127)
      ])
    );

    const borrowerAccount = await marketAndConfig.market.deriveMarginUserAddress(marginAccount);
    const refreshIx = await marketAndConfig.market.program.methods
      .refreshPosition(true)
      .accounts({
        borrowerAccount,
        marginAccount: marginAccount.address,
        claimsMint: marketAndConfig.market.addresses.claimsMint,
        bondManager: marketAndConfig.market.addresses.bondManager,
        underlyingOracle: marketAndConfig.market.addresses.underlyingOracle,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction();

    await marginAccount.withAdapterInvoke({
      instructions: ixns,
      adapterInstruction: refreshIx
    });

    await marginAccount.withAdapterInvoke({
      instructions: ixns,
      adapterInstruction: borrowOffer
    });

    await provider
      .sendAndConfirm(new Transaction().add(...ixns))
      .then(result => {
        console.log('SUCCESS: ', result);
      })
      .catch(e => {
        console.log('ERROR: ', e);
      });
  };

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={rowOrder} setOrder={setRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.fixedView.borrow.title}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body">
        <Input
          onChange={e => setAmount(new BN(parseFloat(e.target.value) * 10 ** decimals))}
          placeholder="enter order value"
          type="number"
        />
        <Input
          onChange={e => {
            setBasisPoints(new BN(parseFloat(e.target.value) * 100));
          }}
          placeholder="enter interest"
          type="number"
          step=".01"
          min="0"
        />
        <Button onClick={offerLoan}>Create Borrow Order</Button>
      </div>
    </div>
  );
};
