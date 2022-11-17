import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../state/settings/localization/localization';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { PoolsTable } from '@components/PoolsView/PoolsTable/PoolsTable';
import { PoolDetail } from '@components/PoolsView/PoolDetail/PoolDetail';
import { Radar } from '@components/PoolsView/Radar';
import { PoolsRowOrder, PoolsViewOrder } from '@state/views/views';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';

// App view for using / viewing Jet pools
export function PoolsView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const networkState = useRecoilValue(NetworkStateAtom);
  const rowOrder = useRecoilValue(PoolsRowOrder);
  const viewOrder = useRecoilValue(PoolsViewOrder);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.poolsView.title} | Jet Protocol`;
  }, [dictionary.poolsView.title]);

  // Row of Pool Detail and Radar components
  const rowComponents: Record<string, JSX.Element> = {
    poolDetail: <PoolDetail key="poolDetail" />,
    radar: <Radar key="radar" />
  };
  const poolsRow = (): JSX.Element => {
    const poolsRowComponents: JSX.Element[] = [];
    for (const component of rowOrder) {
      poolsRowComponents.push(rowComponents[component]);
    }
    return (
      <div key="viewRow" className="view-row pools-row">
        {poolsRowComponents}
      </div>
    );
  };

  // Pools view with ordered components
  const viewComponents: Record<string, JSX.Element> = {
    accountSnapshot: <AccountSnapshot key="accountSnapshot" />,
    poolsRow: poolsRow(),
    poolsTable: <PoolsTable key="poolsTable" />
  };
  const PoolsView = (): JSX.Element => {
    const PoolsViewComponents: JSX.Element[] = [];
    for (const component of viewOrder) {
      PoolsViewComponents.push(viewComponents[component]);
    }
    return <div className="pools-view view">{PoolsViewComponents}</div>;
  };

  if (networkState !== 'connected') return <WaitingForNetworkView networkState={networkState} />;

  return PoolsView();
}
