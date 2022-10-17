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

function randomIntFromInterval(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1) + min);
}

export const FixedLendOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
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
    const loanOffer = await marketAndConfig.market.offerLoanIx(
      marginAccount,
      amount,
      basisPoints,
      wallet.publicKey,
      Uint8Array.from([
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127),
        randomIntFromInterval(0, 127)
      ])
    );
    let ixns: TransactionInstruction[] = [];
    await marginAccount.withAdapterInvoke({
      instructions: ixns,
      adapterProgram: config.bondsProgramId,
      adapterMetadata: marketAndConfig.market.addresses.marginAdapterMetadata,
      adapterInstruction: loanOffer
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
          <Paragraph className="order-entry-head-top-title">{dictionary.fixedView.lend.title}</Paragraph>
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
        <Button onClick={offerLoan}>Create Lend Order</Button>
      </div>
    </div>
  );
};
