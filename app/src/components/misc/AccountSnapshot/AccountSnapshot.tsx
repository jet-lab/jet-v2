import { useLocation } from 'react-router-dom';
import { useRecoilState, SetterOrUpdater } from 'recoil';
import { PoolsViewOrder, SwapsViewOrder, AccountsViewOrder } from '../../../state/views/views';
import { ReorderArrows } from '../ReorderArrows';
import { SnapshotHead } from './SnapshotHead';
import { SnapshotBody } from './SnapshotBody';
import { SnapshotFooter } from './SnapshotFooter';

// The snapshot of data / base actions that follows the user around the app
export function AccountSnapshot(): JSX.Element {
  const { pathname } = useLocation();
  const [poolsViewOrder, setPoolsViewOrder] = useRecoilState(PoolsViewOrder);
  const [swapsViewOrder, setSwapsViewOrder] = useRecoilState(SwapsViewOrder);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);

  // Determine which component ordering state/reordering method to utilize
  function getOrderContext(): {
    order: string[];
    setOrder: SetterOrUpdater<string[]>;
  } {
    switch (pathname) {
      case '/':
        return {
          order: poolsViewOrder,
          setOrder: setPoolsViewOrder
        };
      case '/swaps':
        return {
          order: swapsViewOrder,
          setOrder: setSwapsViewOrder
        };
      default:
        return {
          order: accountsViewOrder,
          setOrder: setAccountsViewOrder
        };
    }
  }

  return (
    <div className="account-snapshot view-element flex-centered column">
      <SnapshotHead />
      <SnapshotBody />
      <SnapshotFooter />
      <ReorderArrows
        component="accountSnapshot"
        order={getOrderContext().order}
        setOrder={getOrderContext().setOrder}
        vertical
      />
    </div>
  );
}
