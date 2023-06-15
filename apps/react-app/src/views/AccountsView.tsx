import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { FullAccountBalance } from '@components/tables/FullAccountBalance';
import { FullAccountHistory } from '@components/tables/FullAccountHistory';
import { Dictionary, Geobanned } from '@state/settings/localization/localization';
import { AccountsViewOrder } from '@state/views/views';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';
import { GeobannedComponent } from '@components/misc/GeoBanned';

// App view for managing / checking margin accounts
function AccountsView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const networkState = useRecoilValue(NetworkStateAtom);
  const viewOrder = useRecoilValue(AccountsViewOrder);
  const geoBanned = useRecoilValue(Geobanned);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.accountsView.title} | Jet Protocol`;
  }, [dictionary.accountsView.title]);

  // Account view with ordered components
  const viewComponents: Record<string, JSX.Element> = {
    accountSnapshot: <AccountSnapshot key="accountSnapshot" />,
    fullAccountHistory: <FullAccountHistory key="fullAccountHistory" />,
    fullAccountBalance: <FullAccountBalance key="fullAccountBalance" />
  };

  const accountView = (): JSX.Element => {
    const accountViewComponents: JSX.Element[] = [];
    for (const component of viewOrder) {
      accountViewComponents.push(viewComponents[component]);
    }
    return <div className="accounts-view view">{accountViewComponents}</div>;
  };

  if (networkState !== 'connected' || geoBanned === undefined)
    return <WaitingForNetworkView networkState={networkState} />;

  if (geoBanned.banned) {
    return <GeobannedComponent countryCode={geoBanned.countryCode} />;
  }

  return accountView();
}

export default AccountsView;
