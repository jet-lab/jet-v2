import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '../components/misc/AccountSnapshot';
import { FullAccountBalance } from '../components/tables/FullAccountBalance';
import { FullAccountHistory } from '../components/tables/FullAccountHistory';
import { Dictionary } from '../state/settings/localization/localization';
import { AccountsViewOrder } from '../state/views/views';
import { animateViewIn } from '../utils/ui';

export function AccountsView(): JSX.Element {
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
    return <div className="borrow-view view">{accountViewComponents}</div>;
  };

  useEffect(() => animateViewIn(), []);
  return accountView();
}
