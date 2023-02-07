import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { PoolsRowOrder } from '@state/views/views';
import { useCurrencyFormatting } from '@utils/currency';
import { formatRate } from '@utils/format';
import { PieChart } from './PieChart';
import { TokenLogo } from '@components/misc/TokenLogo';
import { AirdropButton } from './AirdropButton';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Info } from '@components/misc/Info';
import { Skeleton, Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';
import { PoolData } from '@jet-lab/store/dist/slices/pools';
import { useMemo } from 'react';
import { CopyableField } from '@components/misc/CopyableField';
const { Title, Paragraph, Text } = Typography;

interface WithPoolData {
  selectedPool?: PoolData;
}

// Renders the collateral weight for the current pool
function CollateralWeight({ selectedPool }: WithPoolData) {
  if (selectedPool) {
    return <Text>{formatRate(selectedPool.collateral_weight)}</Text>;
  }
  return <Skeleton paragraph={false} active style={{ width: 100 }} />;
}

// Renders the current price of the current pool
const CurrentPrice = ({ selectedPool, price }: WithPoolData & { price?: number }) => {
  const { currencyFormatter } = useCurrencyFormatting();
  if (selectedPool && price) {
    return <Text>{`1 ${selectedPool.symbol} = ${currencyFormatter(price, true)}`}</Text>;
  }

  return <Skeleton paragraph={false} active style={{ width: 100 }} />;
};

// Renders the utilization rate of the current pool
const UtilizationRate = ({ selectedPool }: WithPoolData) => {
  const dictionary = useRecoilValue(Dictionary);
  let rateString = '—%';
  if (selectedPool && selectedPool.deposit_tokens > 0) {
    rateString = formatRate(selectedPool.borrowed_tokens / selectedPool.deposit_tokens);
  }
  return <Text type="secondary" italic>{`${dictionary.poolsView.utilizationRate} ${rateString}`}</Text>;
};

// Renders the required collateral factor for the current pool
const CollateralFactor = ({ selectedPool }: WithPoolData) =>
  selectedPool ? (
    <Text>{selectedPool.collateral_factor}</Text>
  ) : (
    <Skeleton paragraph={false} active style={{ width: 100 }} />
  );

// Renders the total borrowed to accompany the pie chart
const TotalBorrowed = ({ selectedPool }: { selectedPool: PoolData }) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
  if (selectedPool) {
    const borrowedAbbrev = currencyAbbrev(selectedPool.borrowed_tokens, selectedPool.precision, false, undefined);
    render = (
      <div className="pie-chart-section-info-item">
        <Text type="danger">{borrowedAbbrev}</Text>
      </div>
    );
  }

  return render;
};

// Renders the pool size for the current pool
const PoolSize = ({ selectedPool }: { selectedPool: PoolData }) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  if (selectedPool) {
    const totalValueAbbrev = currencyAbbrev(selectedPool.deposit_tokens, selectedPool.precision, false, undefined);
    return <Title className="green-text">{`${totalValueAbbrev}`}</Title>;
  }

  return <Skeleton className="align-center" paragraph={false} active style={{ margin: '10px 0' }} />;
};

// Renders the available liquidity to accompany the pie chart
const AvailableLiquidity = ({ selectedPool }: { selectedPool: PoolData }) => {
  const { currencyAbbrev } = useCurrencyFormatting();
  let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
  if (selectedPool) {
    const vaultAbbrev = currencyAbbrev(
      selectedPool.deposit_tokens - selectedPool.borrowed_tokens,
      selectedPool.precision,
      false,
      undefined
    );
    render = (
      <div className="pie-chart-section-info-item">
        <Text type="success">{vaultAbbrev}</Text>
      </div>
    );
  }

  return render;
};

// Component that shows extra details on the selectedPool
export function PoolDetail(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [poolsRowOrder, setPoolsRowOrder] = useRecoilState(PoolsRowOrder);

  const { prices, selectedPoolKey, pools } = useJetStore(state => ({
    pools: state.pools,
    prices: state.prices,
    selectedPoolKey: state.selectedPoolKey
  }));

  const selectedPool = useMemo(() => pools[selectedPoolKey], [selectedPoolKey]);
  const selectedPoolPrice = useMemo(() => prices && prices[selectedPool.token_mint].price, [selectedPool]);

  return (
    <div className="pool-detail view-element flex align-center justify-start column">
      <div className="pool-detail-head flex align-center justify-start">
        <ReorderArrows component="poolDetail" order={poolsRowOrder} setOrder={setPoolsRowOrder} />
        <Paragraph strong>{dictionary.poolsView.poolDetail.title}</Paragraph>
      </div>
      <div className="pool-detail-body flex align-start justify-center">
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section pool-detail-title flex align-center justify-start">
            <TokenLogo height={30} symbol={selectedPool?.symbol} />
            <Title className="pool-detail-header">{selectedPool?.symbol ?? ''}</Title>
            <AirdropButton />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Text className="small-accent-text">{dictionary.common.currentPrice}</Text>
            <CurrentPrice price={selectedPoolPrice} selectedPool={selectedPool} />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="collateralWeight">
              <Text className="info-element small-accent-text">{dictionary.poolsView.collateralWeight}</Text>
            </Info>
            <CollateralWeight selectedPool={selectedPool} />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="requiredCollateralFactor">
              <Text className="info-element small-accent-text">{dictionary.poolsView.requiredCollateralFactor}</Text>
            </Info>
            <CollateralFactor selectedPool={selectedPool} />
          </div>
        </div>
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section flex-centered column">
            <Text className="small-accent-text">{dictionary.poolsView.poolDetail.poolSize}</Text>
            <PoolSize selectedPool={selectedPool} />
            <UtilizationRate selectedPool={selectedPool} />
          </div>
          <div className="pie-chart-section pool-detail-body-half-section flex-centered">
            <PieChart
              percentage={selectedPool ? selectedPool.borrowed_tokens / selectedPool.deposit_tokens : 0}
              text={dictionary.poolsView.utilizationRate.toUpperCase()}
              term="utilizationRate"
            />
            <div className="pie-chart-section-info flex align-start justify-center column">
              <div className="flex column">
                <Text className="small-accent-text">{dictionary.poolsView.availableLiquidity}</Text>
                <AvailableLiquidity selectedPool={selectedPool} />
              </div>
              <div className="pie-chart-section-info flex column">
                <Text className="small-accent-text">{dictionary.poolsView.totalBorrowed}</Text>
                <TotalBorrowed selectedPool={selectedPool} />
              </div>
            </div>
          </div>
        </div>
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section column">
            {selectedPool && (
              <>
                <Text className="info-element small-accent-text">Pool Address</Text>
                <div className={`pool-detail-body-half-section flex align-start justify-center column`}>
                  <CopyableField content={selectedPool.address} />
                </div>
              </>
            )}
            {selectedPool && (
              <>
                <Text className="info-element small-accent-text">Token Address</Text>
                <div className={`pool-detail-body-half-section flex align-start justify-start column`}>
                  <CopyableField content={selectedPool.token_mint} />
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
