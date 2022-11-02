import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, InputNumber, Typography } from 'antd';
import { Suspense, useMemo, useState } from 'react';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { FixedMarketAtom } from '@state/fixed-market/fixed-term-market-sync';
import { CurrentAccount } from '@state/user/accounts';
import BN from 'bn.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MainConfig } from '@state/config/marginConfig';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { createFixedBorrowOrder } from '@jet-lab/jet-bonds-client';
import { OrderList } from './OrderList';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { marketToString } from '@utils/jet/fixed-term-utils';

export const FixedBorrowOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const marginConfig = useRecoilValue(MainConfig);

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

  if (!decimals) return null;

  const createBorrowOrder = async () => {
    let signature: string;
    try {
      signature = await createFixedBorrowOrder({
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
        'Borrow Order Created',
        `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} was created successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
    } catch (e) {
      notify(
        'Borrow Order Failed',
        `Your borrow order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
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
          <Paragraph className="order-entry-head-top-title">{marketToString(marketAndConfig.config)}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body">
        <div className='fixed-order-entry-fields'>
        <label>
          Loan amount
          <InputNumber
            onChange={e => setAmount(new BN(e * 10 ** decimals))}
            min={0}
            formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
        <label>
          Interest Rate
          <InputNumber
            onChange={e => {
              setBasisPoints(new BN(e * 100));
            }}
            type="number"
            step={0.01}
            min={0}
            controls={false}
            addonAfter="%"
          />
        </label>
        </div>
        <Button disabled={!marketAndConfig?.market} onClick={createBorrowOrder}>
          Request {marketToString(marketAndConfig.config)} loan
        </Button>
      </div>
    </div>
  );
};
