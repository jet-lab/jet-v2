import { useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentSwapOutput } from '@state/actions/actions';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { Typography } from 'antd';
import { useJetStore, getSwapLiquidity } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import { SwapChartComponent } from './SwapChartComponent';
import { ParentSizeModern } from '@visx/responsive';
import { Pool } from '@jet-lab/margin';
import { LoadingOutlined } from '@ant-design/icons';

// Graph for displaying pricing and slippage data for current swap pair
export function SwapChart(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentAccount = useRecoilValue(CurrentAccount);
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const outputToken = useRecoilValue(CurrentSwapOutput);

  const { Title } = Typography;

  return (
    <div className="swaps-graph view-element align-center column flex justify-end">
      <div className="swaps-graph-head column flex justify-center">
        <div className="swaps-graph-head-info flex-centered flex">
          <div className="flex-centered">
            <Title level={2}>{dictionary.actions.swap.swapsGraph.title}</Title>
          </div>
        </div>
      </div>
      <ConnectionFeedback />
      <div className="swaps-chart-root">
        {!currentPool || !outputToken ? (
          <LoadingOutlined />
        ) : (
          <DataWrapper currentPool={currentPool} outputToken={outputToken} />
        )}
      </div>
    </div>
  );
}

interface DataWrapperProps {
  currentPool: Pool;
  outputToken: Pool;
}

const DataWrapper = ({ currentPool, outputToken }: DataWrapperProps) => {
  const { cluster, prices } = useJetStore(state => ({ cluster: state.settings.cluster, prices: state.prices }));
  const swapEndpoint: string =
    cluster === 'mainnet-beta'
      ? ''
      : cluster === 'devnet'
      ? String(process.env.REACT_APP_DEV_SWAP_API)
      : String(process.env.REACT_APP_LOCAL_SWAP_API);

  const { data } = getSwapLiquidity(
    swapEndpoint,
    currentPool?.tokenMint.toString(),
    outputToken?.tokenMint.toString(),
    10 // TODO revert to correct amount
  );

  const oraclePrice = useMemo(() => {
    if (!data || !prices) return 0;
    const base = prices[data.base].price;
    const quote = prices[data.quote].price;
    return base / quote;
  }, [data, prices]);

  console.log(data, prices, oraclePrice);

  return (
    <ParentSizeModern>
      {({ height, width }) => (
        <SwapChartComponent
          oraclePrice={oraclePrice}
          bids={data?.bids}
          asks={data?.asks}
          height={height}
          width={width}
        />
      )}
    </ParentSizeModern>
  );
};
// todo - fix tooltip info
// todo - fix output token state for swap entry
// todo - fix entry state error, not match update, one state behind
