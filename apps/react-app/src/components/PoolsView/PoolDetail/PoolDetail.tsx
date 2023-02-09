import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { PoolsRowOrder } from '@state/views/views';
import { Pools, CurrentPool } from '@state/pools/pools';
import { useCurrencyFormatting } from '@utils/currency';
import { formatRate } from '@utils/format';
import { PieChart } from './PieChart';
import { TokenLogo } from '@components/misc/TokenLogo';
import { AirdropButton } from './AirdropButton';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Info } from '@components/misc/Info';
import { Skeleton, Typography } from 'antd';
import { CopyableField } from '@components/misc/CopyableField';

// Component that shows extra details on the currentPool
export function PoolDetail(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const [poolsRowOrder, setPoolsRowOrder] = useRecoilState(PoolsRowOrder);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const init = pools && currentPool;
  const { Title, Paragraph, Text } = Typography;

  // Renders the current price of the current pool
  function renderCurrentPrice() {
    let render = <Skeleton paragraph={false} active style={{ width: 100 }} />;
    if (init) {
      render = <Text>{`1 ${currentPool.symbol} = ${currencyFormatter(currentPool.tokenPrice, true)}`}</Text>;
    }

    return render;
  }

  // Renders the collateral weight for the current pool
  function renderCollateralWeight() {
    let render = <Skeleton paragraph={false} active style={{ width: 100 }} />;
    if (init) {
      render = <Text>{formatRate(currentPool.depositNoteMetadata.valueModifier.toNumber())}</Text>;
    }

    return render;
  }

  // Renders the required collateral factor for the current pool
  function renderRequiredCollateralFactor() {
    let render = <Skeleton paragraph={false} active style={{ width: 100 }} />;
    if (init) {
      render = <Text>{currentPool.loanNoteMetadata.valueModifier.toNumber()}</Text>;
    }

    return render;
  }

  // Renders the pool size for the current pool
  function renderPoolSize() {
    let render = <Skeleton className="align-center" paragraph={false} active style={{ margin: '10px 0' }} />;
    if (init) {
      const totalValueAbbrev = currencyAbbrev(currentPool.totalValue.tokens, currentPool.precision, false, undefined);
      render = <Title className="green-text">{`${totalValueAbbrev}`}</Title>;
    }

    return render;
  }

  // Renders the available liquidity to accompany the pie chart
  function renderAvailableLiquidity() {
    let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
    if (init) {
      const vaultAbbrev = currencyAbbrev(currentPool.vault.tokens, currentPool.precision, false, undefined);
      render = (
        <div className="pie-chart-section-info-item">
          <Text type="success">{vaultAbbrev}</Text>
        </div>
      );
    }

    return render;
  }

  // Renders the total borrowed to accompany the pie chart
  function renderTotalBorrowed() {
    let render = <Skeleton paragraph={false} active style={{ marginTop: 5 }} />;
    if (init) {
      const borrowedAbbrev = currencyAbbrev(currentPool.borrowedTokens.tokens, currentPool.precision, false, undefined);
      render = (
        <div className="pie-chart-section-info-item">
          <Text type="danger">{borrowedAbbrev}</Text>
        </div>
      );
    }

    return render;
  }

  // Renders the utilization rate of the current pool
  function renderUtilizationRate() {
    let rateString = '—%';
    if (init) {
      rateString = formatRate(currentPool.utilizationRate);
    }

    const render = <Text type="secondary" italic>{`${dictionary.poolsView.utilizationRate} ${rateString}`}</Text>;
    return render;
  }

  return (
    <div className="pool-detail view-element flex align-center justify-start column">
      <div className="pool-detail-head flex align-center justify-start">
        <ReorderArrows component="poolDetail" order={poolsRowOrder} setOrder={setPoolsRowOrder} />
        <Paragraph strong>{dictionary.poolsView.poolDetail.title}</Paragraph>
      </div>
      <div className="pool-detail-body flex align-start justify-center">
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section pool-detail-title flex align-center justify-start">
            <TokenLogo height={30} symbol={currentPool?.symbol} />
            <Title className="pool-detail-header">{currentPool?.name ?? ''}</Title>
            <AirdropButton />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Text className="small-accent-text">{dictionary.common.currentPrice}</Text>
            {renderCurrentPrice()}
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="collateralWeight">
              <Text className="info-element small-accent-text">{dictionary.poolsView.collateralWeight}</Text>
            </Info>
            {renderCollateralWeight()}
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="requiredCollateralFactor">
              <Text className="info-element small-accent-text">{dictionary.poolsView.requiredCollateralFactor}</Text>
            </Info>
            {renderRequiredCollateralFactor()}
          </div>
        </div>
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section flex-centered column">
            <Text className="small-accent-text">{dictionary.poolsView.poolDetail.poolSize}</Text>
            {renderPoolSize()}
            {renderUtilizationRate()}
          </div>
          <div className="pie-chart-section pool-detail-body-half-section flex-centered">
            <PieChart
              percentage={init ? currentPool.utilizationRate : 0}
              text={dictionary.poolsView.utilizationRate.toUpperCase()}
              term="utilizationRate"
            />
            <div className="pie-chart-section-info flex align-start justify-center column">
              <div className="flex column">
                <Text className="small-accent-text">{dictionary.poolsView.availableLiquidity}</Text>
                {renderAvailableLiquidity()}
              </div>
              <div className="pie-chart-section-info flex column">
                <Text className="small-accent-text">{dictionary.poolsView.totalBorrowed}</Text>
                {renderTotalBorrowed()}
              </div>
            </div>
          </div>
        </div>
        <div className='pool-detail-body-half flex-align-start justify-center column'>
          <div className="pool-detail-body-half-section column">
            {currentPool && (
              <>
                <Text className="info-element small-accent-text">Pool Address</Text>
                <div className={`pool-detail-body-half-section flex align-start justify-center column`}>
                  <CopyableField content={currentPool.address.toBase58()} />
                </div>
              </>
            )}
            {currentPool && (
              <>
                <Text className="info-element small-accent-text">Token Address</Text>
                <div className={`pool-detail-body-half-section flex align-start justify-start column`}>
                  <CopyableField content={currentPool.addresses.tokenMint.toBase58()} />
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
