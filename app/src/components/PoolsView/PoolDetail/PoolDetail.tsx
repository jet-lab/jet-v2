import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import axios from 'axios';
import { useWallet } from '@solana/wallet-adapter-react';
import { BlockExplorer, Cluster } from '../../../state/settings/settings';
import { Dictionary } from '../../../state/settings/localization/localization';
import { PoolsRowOrder } from '../../../state/views/views';
import { WalletModal } from '../../../state/modals/modals';
import { PoolOptions, PoolsInit, CurrentPool } from '../../../state/borrow/pools';
import { useCurrencyFormatting } from '../../../utils/currency';
import { formatRate } from '../../../utils/format';
import { ActionResponse } from '../../../utils/jet/marginActions';
import { useMarginActions } from '../../../utils/jet/marginActions';
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { PieChart } from './PieChart';
import { Button, Skeleton, Typography } from 'antd';
import { CloudFilled } from '@ant-design/icons';
import { TokenLogo } from '../../misc/TokenLogo';
import { ReorderArrows } from '../../misc/ReorderArrows';
import { PriceHistorySparkline } from '../../misc/PriceHistorySparkline';
import { useMarginConfig } from '../../../utils/jet/marginConfig';

export function PoolDetail(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const { connected } = useWallet();
  const { airdrop } = useMarginActions();
  const [poolsRowOrder, setPoolsRowOrder] = useRecoilState(PoolsRowOrder);
  const poolOptions = useRecoilValue(PoolOptions);
  const poolsInit = useRecoilValue(PoolsInit);
  const currentPool = useRecoilValue(CurrentPool);
  const [marketCaps, setMarketCaps] = useState<Record<string, number>>({});
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Paragraph, Text } = Typography;
  const config = useMarginConfig();

  // Airdrop token
  async function doAirdrop() {
    if (!currentPool) {
      return;
    }

    setSendingTransaction(true);
    const [txId, resp] = await airdrop(currentPool);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.successDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', currentPool?.symbol === 'SOL' ? '1' : '100')
          .replace('{{ASSET}}', currentPool?.symbol ?? ''),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.actions.cancelledTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.cancelledDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', currentPool?.symbol === 'SOL' ? '1' : '100')
          .replace('{{ASSET}}', currentPool?.symbol ?? ''),
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.actions.failedTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.failedDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', currentPool?.symbol === 'SOL' ? '1' : '100')
          .replace('{{ASSET}}', currentPool?.symbol ?? ''),
        'error'
      );
    }
    setSendingTransaction(false);
  }

  // Fetch marketCaps from coingecko API on init
  useEffect(() => {
    if (poolOptions) {
      const marketCaps: Record<string, number> = {};
      for (const pool of poolOptions) {
        const nameFormatted = pool.name.toLowerCase().replaceAll(' ', '-');
        axios
          .get(`https://api.coingecko.com/api/v3/coins/${nameFormatted}`)
          .then(({ data }) => {
            const pool = data.data;
            if (pool && pool.market_cap) {
              marketCaps[pool.symbol] = pool.market_cap.usd;
              setMarketCaps(marketCaps);
            }
          })
          .catch(err => err);
      }
    }
  }, [poolOptions, poolsInit]);

  return (
    <div className="pool-detail view-element view-element-hidden flex align-center justify-start column">
      <div className="pool-detail-head flex align-center justify-start view-element-item view-element-item-hidden">
        <ReorderArrows component="poolDetail" order={poolsRowOrder} setOrder={setPoolsRowOrder} />
        <Paragraph strong>{dictionary.poolsView.poolDetail.title}</Paragraph>
      </div>
      <div className="pool-detail-body flex align-start justify-center view-element-item view-element-item-hidden">
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section pool-detail-title flex align-center justify-start">
            <TokenLogo height={30} symbol={currentPool?.symbol} />
            <Title className="pool-detail-header">
              <b>{currentPool?.name ?? ''}</b>
            </Title>
            {cluster === 'devnet' && config && (
              <Button
                type="dashed"
                style={{ marginLeft: 20 }}
                onClick={() => (connected ? doAirdrop() : setWalletModalOpen(true))}
                disabled={!currentPool || sendingTransaction}
                loading={sendingTransaction}
                icon={<CloudFilled />}>
                {dictionary.poolsView.poolDetail.airdrop}
              </Button>
            )}
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Text className="small-accent-text">{dictionary.common.currentPrice}</Text>
            {poolsInit && currentPool?.symbol ? (
              <Text>{`1 ${currentPool.symbol} = ${currencyFormatter(currentPool.tokenPrice, true)}`}</Text>
            ) : (
              <Skeleton paragraph={false} active style={{ width: 100 }} />
            )}
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Text className="small-accent-text">{dictionary.poolsView.poolDetail.fullyDilutedMarketCap}</Text>
            {poolsInit && currentPool?.symbol ? (
              <Text>{currencyFormatter(marketCaps[currentPool.symbol] ?? 0, true)}</Text>
            ) : (
              <Skeleton paragraph={false} active style={{ width: 100 }} />
            )}
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <PriceHistorySparkline />
          </div>
        </div>
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section flex-centered column">
            <Text className="small-accent-text">{dictionary.poolsView.poolDetail.poolSize}</Text>
            {poolsInit && currentPool?.symbol ? (
              <Title className="green-text">{`${currencyAbbrev(
                currentPool.totalValue.tokens,
                false,
                undefined,
                currentPool.decimals
              )} ${currentPool?.symbol}`}</Title>
            ) : (
              <Skeleton className="align-center" paragraph={false} active style={{ margin: '10px 0' }} />
            )}
            <Text type="secondary" italic>{`${dictionary.poolsView.utilizationRate} ${
              poolsInit && currentPool ? formatRate(currentPool.utilizationRate) : '—%'
            }`}</Text>
          </div>
          <div className="pie-chart-section pool-detail-body-half-section flex-centered">
            <PieChart
              percentage={poolsInit && currentPool ? currentPool.utilizationRate : 0}
              text={dictionary.poolsView.utilizationRate.toUpperCase()}
              term="utilizationRate"
            />
            <div className="pie-chart-section-info flex align-start justify-center column">
              <div className="flex column">
                <Text className="small-accent-text">{dictionary.poolsView.availableLiquidity}</Text>
                {poolsInit && currentPool?.symbol ? (
                  <div className="pie-chart-section-info-item">
                    <Text type="success">
                      {currencyAbbrev(currentPool.vault.tokens, false, undefined, currentPool.decimals)}
                    </Text>
                    <Text>{currentPool?.symbol}</Text>
                  </div>
                ) : (
                  <Skeleton paragraph={false} active style={{ marginTop: 5 }} />
                )}
              </div>
              <div className="pie-chart-section-info flex column">
                <Text className="small-accent-text">{dictionary.poolsView.totalBorrowed}</Text>
                {poolsInit && currentPool?.symbol ? (
                  <div className="pie-chart-section-info-item">
                    <Text type="danger">
                      {currencyAbbrev(currentPool.borrowedTokens.tokens, false, undefined, currentPool.decimals)}
                    </Text>
                    <Text>{currentPool?.symbol}</Text>
                  </div>
                ) : (
                  <Skeleton paragraph={false} active style={{ marginTop: 5 }} />
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
