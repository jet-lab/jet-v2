import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, Input, Typography } from 'antd';
import { Suspense, useMemo, useState } from 'react';
import BN from 'bn.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MainConfig } from '@state/config/marginConfig';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { createFixedLendOrder } from '@jet-lab/jet-bonds-client';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { OrderList } from './OrderList';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { FixedMarketAtom } from '@state/fixed-market/fixed-term-market-sync';
import { CurrentAccount } from '@state/user/accounts';

export const FixedLendOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const marginConfig = useRecoilValue(MainConfig);
  const cluster = useRecoilValue(Cluster);
  const blockExplorer = useRecoilValue(BlockExplorer);

  const token = useMemo(() => {
    if (!marginConfig || !marketAndConfig) return null;
    return Object.values(marginConfig?.tokens).find(token => {
      return marketAndConfig.config.underlyingTokenMint === token.mint.toString();
    });
  }, [marginConfig, marketAndConfig?.config]);

  const decimals = useMemo(() => {
    if (!token) return null;
    if (!marginConfig || !marketAndConfig?.config) return 6;
    return token.decimals;
  }, [token]);

  const [amount, setAmount] = useState(new BN(0));
  const [basisPoints, setBasisPoints] = useState(new BN(0));

  const { Paragraph } = Typography;

  const offerLoan = async () => {
    let signature: string;
    try {
      signature = await createFixedLendOrder({
        market: marketAndConfig.market,
        marginAccount,
        marginConfig,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        currentPool,
        amount,
        basisPoints
      });
      notify(
        'Lend Order Created',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} was created successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
    } catch (e) {
      notify(
        'Lend Order Failed',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
    }
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
        <Button disabled={!marketAndConfig?.market} onClick={offerLoan}>
          Create Lend Order
        </Button>
      </div>
    </div>
  );
};
