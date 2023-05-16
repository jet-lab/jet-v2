import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary, Geobanned } from '../state/settings/localization/localization';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { PoolsTable } from '@components/PoolsView/PoolsTable/PoolsTable';
import { PoolDetail } from '@components/PoolsView/PoolDetail/PoolDetail';
import { PoolsViewOrder } from '@state/views/views';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';
import { GeobannedComponent } from '@components/misc/GeoBanned';

// App view for using / viewing Jet pools
function PoolsView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const networkState = useRecoilValue(NetworkStateAtom);
  const viewOrder = useRecoilValue(PoolsViewOrder);
  const geoBanned = useRecoilValue(Geobanned);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.poolsView.title} | Jet Protocol`;
  }, [dictionary.poolsView.title]);

  // Pools view with ordered components
  const viewComponents: Record<string, JSX.Element> = {
    accountSnapshot: <AccountSnapshot key="accountSnapshot" />,
    poolsRow: <PoolDetail key="pools-details" />,
    poolsTable: <PoolsTable key="poolsTable" />
  };
  const PoolsView = (): JSX.Element => {
    const PoolsViewComponents: JSX.Element[] = [];
    for (const component of viewOrder) {
      PoolsViewComponents.push(viewComponents[component]);
    }
    return <div className="pools-view view">{PoolsViewComponents}</div>;
  };

  if (networkState !== 'connected' || geoBanned === undefined)
    return <WaitingForNetworkView networkState={networkState} />;

  if (geoBanned) {
    return <GeobannedComponent />;
  }

  return PoolsView();
}

export default PoolsView;
