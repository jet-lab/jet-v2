import React, { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { FixedPriceChartContainer } from '@components/fixed-term/shared/fixed-term-market-chart';
import { FullAccountBalance } from '@components/tables/FullAccountBalance';
import { Dictionary } from '@state/settings/localization/localization';
import { FixedLendOrderEntry } from '@components/fixed-term/lend-entry';
import { FixedLendRowOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { FixedTermMarketSelector } from '@components/fixed-term/shared/market-selector';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';
import { DebtTable } from '@components/fixed-term/shared/debt-table';

const rowComponents: Record<string, React.FC<any>> = {
  fixedLendEntry: FixedLendOrderEntry,
  fixedLendChart: FixedPriceChartContainer
};

const rowComponentsProps: Record<string, object> = {
  fixedLendEntry: { key: 'fixedLendEntry' },
  fixedLendChart: { key: 'fixedLendChart', type: 'bids' }
};

const FixedRow = (): JSX.Element => {
  const rowOrder = useRecoilValue(FixedLendRowOrder);
  return (
    <div key="fixedRow" className="view-row fixed-row">
      {rowOrder.map(key => {
        const Comp = rowComponents[key];
        const props = rowComponentsProps[key];
        return <Comp {...props} />;
      })}
    </div>
  );
};

const viewComponents: Record<string, React.FC<any>> = {
  accountSnapshot: AccountSnapshot,
  fixedRow: FixedRow,
  debtTable: DebtTable,
  fullAccountBalance: FullAccountBalance,
  marketSelector: FixedTermMarketSelector
};

const viewComponentsProps: Record<string, object> = {
  accountSnapshot: { key: 'accountSnapshot' },
  fixedRow: { key: 'fixedRow' },
  debtTable: { key: 'debtTable' },
  fullAccountBalance: { key: 'fullAccountBalance' },
  marketSelector: { key: 'marketSelector', type: 'bids' }
};

const MainView = (): JSX.Element => {
  const viewOrder = useRecoilValue(FixedLendViewOrder);

  return (
    <div className="fixed-term-view view">
      {viewOrder.map(key => {
        const Comp = viewComponents[key];
        const props = viewComponentsProps[key];
        return <Comp {...props} />;
      })}
    </div>
  );
};

export function FixedLendView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const networkState = useRecoilValue(NetworkStateAtom);
  useEffect(() => {
    document.title = `${dictionary.fixedView.lend.title} | Jet Protocol`;
  }, [dictionary.fixedView.lend.title]);
  if (networkState !== 'connected') return <WaitingForNetworkView networkState={networkState} />;
  return <MainView />;
}

export default FixedLendView;
