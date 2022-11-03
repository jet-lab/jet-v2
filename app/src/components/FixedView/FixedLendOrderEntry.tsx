import { useRecoilState, useRecoilValue } from 'recoil';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, InputNumber, Switch, Tabs, Typography } from 'antd';
import { useMemo, useState } from 'react';
import BN from 'bn.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MainConfig } from '@state/config/marginConfig';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { createFixedLendOrder } from '@jet-lab/jet-bonds-client';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { FixedLendRowOrder } from '@state/views/fixed-term';
import { FixedMarketAtom } from '@state/fixed-market/fixed-term-market-sync';
import { CurrentAccount } from '@state/user/accounts';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { formatDuration, intervalToDuration } from 'date-fns';

export const FixedLendOrderEntry = () => {
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const marginConfig = useRecoilValue(MainConfig);
  const blockExplorer = useRecoilValue(BlockExplorer);

  const [orderType, setOrderType]= useState('limit')
  
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
          <Paragraph className="order-entry-head-top-title">{marketToString(marketAndConfig.config)}</Paragraph>
        </div>
      </div>
      <Tabs
        defaultActiveKey="limit"
        activeKey={orderType}
        onChange={type => setOrderType(type)}
        items={[
          {
            label: 'offer loan',
            key: 'limit'
          },
          {
            label: 'lend now',
            key: 'market'
          }
        ]}></Tabs>
      {orderType === 'limit' && (
        <div className="order-entry-body">
          <div className="fixed-order-entry-fields">
            <label>
              Loan amount
              <InputNumber
                onChange={e => setAmount(new BN(e * 10 ** decimals))}
                placeholder={'10,000'}
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
                placeholder={'1.5'}
                type="number"
                step={0.01}
                min={0}
                controls={false}
                addonAfter="%"
              />
            </label>
          </div>
          <div className="auto-roll-controls">
            <Switch disabled={true} />
            Auto-roll Off
          </div>
          <div className="stats">
            <div className="stat-line">
              <span>Repayment Date</span>
              <span>
                {formatDuration(
                  intervalToDuration({
                    start: new Date(0),
                    end: new Date(marketAndConfig.config.borrowDuration * 1000)
                  })
                )}{' '}
                from fill
              </span>
            </div>
            <div className="stat-line">
              <span>Repayment Amount</span>
              <span>
                {(amount.toNumber() / 10 ** decimals) * (1 + basisPoints.toNumber() / 10000)} {token.symbol}
              </span>
            </div>
            <div className="stat-line">
              <span>Total Interest</span>
              <span>
                {(amount.toNumber() / 10 ** decimals) * (basisPoints.toNumber() / 10000)} {token.symbol}
              </span>
            </div>
            <div className="stat-line">
              <span>Interest Rate</span>
              <span>{basisPoints.toNumber() / 100}%</span>
            </div>
            <div className="stat-line">Risk Level</div>
            <div className="stat-line">
              <span>Auto Roll</span>
              <span>Off</span>
            </div>
          </div>
          <Button disabled={!marketAndConfig?.market || basisPoints.lte(new BN(0)) || amount.lte(new BN(0))} onClick={offerLoan}>
            Offer {marketToString(marketAndConfig.config)} loan
          </Button>
        </div>
      )}
      {orderType === 'market' && <div className="coming-soon">Coming soon</div>}
    </div>
  );
};
