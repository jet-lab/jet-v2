import React, { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { FixedPriceChartContainer } from '@components/FixedView/FixedPriceChart';
import { FullAccountBalance } from '@components/tables/FullAccountBalance';
import { Dictionary } from '@state/settings/localization/localization';
import { FixedBorrowOrderEntry } from '@components/FixedView/FixedBorrowOrderEntry';
import { FixedBorrowRowOrder, FixedBorrowViewOrder } from '@state/views/fixed-term';
import { FixedMarketSelector } from '@components/FixedView/FixedMarketSelector';

const rowComponents: Record<string, React.FC<any>> = {
  fixedBorrowEntry: FixedBorrowOrderEntry,
  fixedBorrowChart: FixedPriceChartContainer
};

const rowComponentsProps: Record<string, object> = {
  fixedBorrowEntry: { key: 'fixedBorrowEntry' },
  fixedBorrowChart: { key: 'fixedBorrowChart', type: 'asks' }
};

const FixedRow = (): JSX.Element => {
  const rowOrder = useRecoilValue(FixedBorrowRowOrder);
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
  fullAccountBalance: FullAccountBalance,
  marketSelector: FixedMarketSelector
};

const viewComponentsProps: Record<string, object> = {
  accountSnapshot: { key: 'accountSnapshot' },
  fixedRow: { key: 'fixedRow' },
  fullAccountBalance: { key: 'fullAccountBalance' },
  marketSelector: { key: 'marketSelector', type: 'asks' }
};

const MainView = (): JSX.Element => {
  const viewOrder = useRecoilValue(FixedBorrowViewOrder);
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

export function FixedBorrowView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  useEffect(() => {
    document.title = `${dictionary.fixedView.borrow.title} | Jet Protocol`;
  }, [dictionary.fixedView.borrow.title]);
  return <MainView />;
}

export default FixedBorrowView;
