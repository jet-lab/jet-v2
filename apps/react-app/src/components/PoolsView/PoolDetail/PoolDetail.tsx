import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { PieChart } from './PieChart';
import { TokenLogo } from '@components/misc/TokenLogo';
import { AirdropButton } from './AirdropButton';
import { Info } from '@components/misc/Info';
import { Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';
import { useMemo } from 'react';
import { CopyableField } from '@components/misc/CopyableField';
import { AvailableLiquidity } from './AvailableLiquidity';
import { PoolSize } from './PoolSize';
import { TotalBorrowed } from './TotalBorrowed';
import { CollateralWeight } from './CollateralWeight';
import { CurrentPrice } from './CurrentPrice';
import { CollateralFactor } from './CollateralFactor';
import { UtilizationRate } from './UtilizationRate';
const { Title, Paragraph, Text } = Typography;

export interface WithPoolData {
  pool?: PoolData;
}

// Component that shows extra details on the selectedPool
export function PoolDetail(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);

  const { prices, selectedPoolKey, pools } = useJetStore(state => ({
    pools: state.pools,
    prices: state.prices,
    selectedPoolKey: state.selectedPoolKey
  }));

  const selectedPool = useMemo(() => pools[selectedPoolKey], [selectedPoolKey, pools]);
  const selectedPoolPrice = useMemo(() => prices && prices[selectedPool?.token_mint]?.price, [selectedPool, prices]);
  if (!selectedPool) return <></>;

  return (
    <div className="pool-detail view-element flex align-center justify-start column">
      <div className="pool-detail-head flex align-center justify-start">
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
            <CurrentPrice price={selectedPoolPrice} pool={selectedPool} />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="collateralWeight">
              <Text className="info-element small-accent-text">{dictionary.poolsView.collateralWeight}</Text>
            </Info>
            <CollateralWeight pool={selectedPool} />
          </div>
          <div className="pool-detail-body-half-section flex align-start justify-center column">
            <Info term="requiredCollateralFactor">
              <Text className="info-element small-accent-text">{dictionary.poolsView.requiredCollateralFactor}</Text>
            </Info>
            <CollateralFactor pool={selectedPool} />
          </div>
        </div>
        <div className="pool-detail-body-half flex-align-start justify-center column">
          <div className="pool-detail-body-half-section flex-centered column">
            <Text className="small-accent-text">{dictionary.poolsView.poolDetail.poolSize}</Text>
            <PoolSize pool={selectedPool} />
            <UtilizationRate pool={selectedPool} />
          </div>
          <div className="pie-chart-section pool-detail-body-half-section flex-centered">
            <PieChart
              percentage={
                selectedPool.deposit_tokens
                  ? Math.round(
                      (selectedPool.borrowed_tokens / (selectedPool.deposit_tokens + selectedPool.borrowed_tokens)) *
                        100
                    )
                  : 0
              }
            />
            <div className="pie-chart-section-info flex align-start justify-center column">
              <div className="flex column">
                <Text className="small-accent-text">{dictionary.poolsView.availableLiquidity}</Text>
                <AvailableLiquidity pool={selectedPool} />
              </div>
              <div className="pie-chart-section-info flex column">
                <Text className="small-accent-text">{dictionary.poolsView.totalBorrowed}</Text>
                <TotalBorrowed pool={selectedPool} />
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
