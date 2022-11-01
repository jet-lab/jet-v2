import { cancelOrder, Order } from '@jet-lab/jet-bonds-client';
import base58 from 'bs58';
import { useRecoilValue } from 'recoil';
import { AllFixedMarketsOrderBooksAtom, FixedMarketAtom } from '../../state/fixed/fixed-term-market-sync';
import { CurrentPool, Pools } from '../../state/pools/pools';
import { BlockExplorer, Cluster } from '../../state/settings/settings';
import { CurrentAccount } from '../../state/user/accounts';
import { useProvider } from '../../utils/jet/provider';
import { notify } from '../../utils/notify';
import { getExplorerUrl } from '../../utils/ui';

export const OrderList = () => {
  const books = useRecoilValue(AllFixedMarketsOrderBooksAtom);
  const marketAndConfig = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);

  const executeCancel = async (order: Order) => {
    let signature: string;
    try {
      await cancelOrder({
        market: marketAndConfig.market,
        marginAccount,
        provider,
        orderId: order.order_id,
        pools: pools.tokenPools,
        currentPool
      });
      notify(
        'Order cancelled',
        `Your order was cancelled`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
    } catch (e) {
      notify(
        'Failed to cancel',
        `We failed to cancel the order, please try again later.`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
    }
  };
  return (
    <div>
      <h3>Lends</h3>
      {books[0]?.bids
        .filter(order => base58.encode(order.owner) === marginAccount.address.toBase58())
        .map(order => {
          const key = base58.encode(order.order_id);
          return (
            <div key={key}>
              {key} - <div onClick={() => executeCancel(order)}>cancel</div>
            </div>
          );
        })}
      <h3>Borrows</h3>
      {books[0]?.asks
        .filter(order => base58.encode(order.owner) === marginAccount.address.toBase58())
        .map(order => {
          const key = base58.encode(order.order_id);
          return (
            <div key={key}>
              {key} - <div onClick={() => executeCancel(order)}>cancel</div>
            </div>
          );
        })}
    </div>
  );
};
