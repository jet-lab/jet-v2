import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '../components/misc/AccountSnapshot/AccountSnapshot';
import { FullAccountBalance } from '../components/tables/FullAccountBalance';
import { FullAccountHistory } from '../components/tables/FullAccountHistory';
import { Dictionary } from '../state/settings/localization/localization';
import { AccountsViewOrder } from '../state/views/views';

// App view for managing / checking margin accounts
export function AccountsView(): JSX.Element {
  alert("Accounts rendering");
  const dictionary = useRecoilValue(Dictionary);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.accountsView.title} | Jet Protocol`;
  }, [dictionary.accountsView.title]);

  // Account view with ordered components
  const viewOrder = useRecoilValue(AccountsViewOrder);
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

  return accountView();
}
