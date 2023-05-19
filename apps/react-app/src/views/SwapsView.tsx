import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { SwapEntry } from '@components/SwapsView/SwapEntry';
import { SwapChart } from '@components/SwapsView/SwapChart';
import { FullAccountBalance } from '@components/tables/FullAccountBalance';
import { Dictionary, Geobanned } from '@state/settings/localization/localization';
import { SwapsViewOrder, SwapsRowOrder } from '@state/views/views';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';
import { GeobannedComponent } from '@components/misc/GeoBanned';

// App view for margin swapping
function SwapsView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const rowOrder = useRecoilValue(SwapsRowOrder);
  const networkState = useRecoilValue(NetworkStateAtom);
  const geoBanned = useRecoilValue(Geobanned);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.swapsView.title} | Jet Protocol`;
  }, [dictionary.swapsView.title]);
  // Row of Swap Entry and Swaps Graph

  const rowComponents: Record<string, JSX.Element> = {
    swapEntry: <SwapEntry key="swapEntry" />,
    swapsGraph: <SwapChart key="swapsGraph2" />
  };
  const swapsRow = (): JSX.Element => {
    const swapsRowComponents: JSX.Element[] = [];
    for (const component of rowOrder) {
      swapsRowComponents.push(rowComponents[component]);
    }
    return (
      <div key="viewRow" className="view-row swaps-row">
        {swapsRowComponents}
      </div>
    );
  };

  // Swaps view with ordered components
  const viewOrder = useRecoilValue(SwapsViewOrder);
  const viewComponents: Record<string, JSX.Element> = {
    accountSnapshot: <AccountSnapshot key="accountSnapshot" />,
    swapsRow: swapsRow(),
    fullAccountBalance: <FullAccountBalance key="fullAccountBalance" />
  };
  const accountView = (): JSX.Element => {
    const swapsViewComponents: JSX.Element[] = [];
    for (const component of viewOrder) {
      swapsViewComponents.push(viewComponents[component]);
    }
    return <div className="swaps-view view">{swapsViewComponents}</div>;
  };

  if (networkState !== 'connected' || geoBanned === undefined)
    return <WaitingForNetworkView networkState={networkState} />;

  if (geoBanned) {
    return <GeobannedComponent />;
  }
  return accountView();
}

export default SwapsView;
