import { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../state/settings/localization/localization';
import { animateViewIn } from '../utils/ui';
import { AccountSnapshot } from '../components/misc/AccountSnapshot';
import { PoolsTable } from '../components/PoolsView/PoolsTable/PoolsTable';
import { PoolDetail } from '../components/PoolsView/PoolDetail/PoolDetail';
import { Radar } from '../components/PoolsView/Radar';
import { PoolsRowOrder, PoolsViewOrder } from '../state/views/views';

export function PoolsView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);

  // Localize page title
  useEffect(() => {
    document.title = `${dictionary.poolsView.title} | Jet Protocol`;
  }, [dictionary.poolsView.title]);

  // Row of Pool Detail and Radar components
  const rowOrder = useRecoilValue(PoolsRowOrder);
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
      <div key="viewRow" className="view-row borrow-row">
        {poolsRowComponents}
      </div>
    );
  };

  // Pools view with ordered components
  const viewOrder = useRecoilValue(PoolsViewOrder);
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

  useEffect(() => animateViewIn(), []);
  return PoolsView();
}
