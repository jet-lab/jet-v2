import { FixedTermMarket, MarginAccount, Pool, TokenAmount } from '@jet-lab/margin';
import { Table } from 'antd';
import { CloseOutlined, LoadingOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import formatDistanceToNowStrict from 'date-fns/formatDistanceToNowStrict';
import BN from 'bn.js';
import { cancelOrder, MarketAndConfig } from '@jet-lab/margin';
import { Dispatch, SetStateAction, useMemo, useState } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { getExplorerUrl } from '@utils/ui';
import { OpenOrder } from '@jet-lab/store/dist/types';
import { notify } from '@utils/notify';
import { useRecoilRefresher_UNSTABLE } from 'recoil';
import { AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';

type UpdateOrders = Dispatch<SetStateAction<string[]>>;

interface GetPostOrderColumnes {
  market: MarketAndConfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
  ordersPendingDeletion: string[];
  setOrdersPendingDeletion: UpdateOrders;
  refreshOrderBooks: () => void;
}
const getPostOrderColumns = ({
  market,
  marginAccount,
  provider,
  cluster,
  explorer,
  pools,
  markets,
  refreshOrderBooks,
  ordersPendingDeletion,
  setOrdersPendingDeletion
}: GetPostOrderColumnes): ColumnsType<OpenOrder> => [
    {
      title: 'Issue date',
      dataIndex: 'created_timestamp',
      key: 'created_timestamp',
      render: (date: number) => `${formatDistanceToNowStrict(date)} ago`,
      sorter: (a, b) => a.created_timestamp - b.created_timestamp,
      sortDirections: ['descend'],
    },
    {
      title: 'Total QTY',
      dataIndex: 'total_quote_qty',
      key: 'total_quote_qty',
      render: (value: number) => `${market.token.symbol} ${new TokenAmount(new BN(value), 6).tokens.toFixed(2)}`,
      sorter: (a, b) => a.total_quote_qty - b.total_quote_qty,
      sortDirections: ['descend'],
    },
    {
      title: 'Filled QTY',
      dataIndex: 'filled_quote_qty',
      key: 'filled_quote_qty',
      render: (filled: number) => {
        return `${market.token.symbol} ${new TokenAmount(new BN(filled), 6).tokens.toFixed(2)}`;
      },
      sorter: (a, b) => a.filled_quote_qty - b.filled_quote_qty,
      sortDirections: ['descend'],
    },
    {
      title: 'Rate',
      dataIndex: 'rate',
      key: 'rate',
      render: (rate: number) => `${(100 * rate).toFixed(3)}%`,
      sorter: (a, b) => a.rate - b.rate,
      sortDirections: ['descend'],
    },
    {
      title: 'Cancel',
      key: 'cancel',
      render: (order: OpenOrder) => {
        return ordersPendingDeletion.includes(order.order_id) ? (
          <LoadingOutlined />
        ) : (
          <CloseOutlined
            style={{ color: '#e36868' }}
            onClick={() => {
              cancel(
                market,
                marginAccount,
                provider,
                order,
                cluster,
                explorer,
                pools,
                markets,
                refreshOrderBooks,
                ordersPendingDeletion,
                setOrdersPendingDeletion
              );
            }}
          />
        );
      }
    }
  ];

const cancel = async (
  market: MarketAndConfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  order: OpenOrder,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  refreshOrderBooks: () => void,
  ordersPendingDeletion: string[],
  setOrdersPendingDeletion: UpdateOrders
) => {
  try {
    await cancelOrder({
      market,
      marginAccount,
      provider,
      orderId: new BN(order.order_id),
      pools,
      markets
    });
    notify('Order Cancelled', 'Your order was cancelled successfully', 'success');
    setOrdersPendingDeletion([...ordersPendingDeletion, order.order_id]);
    refreshOrderBooks();
  } catch (e: any) {
    notify(
      'Cancel order failed',
      'There was an error cancelling your order',
      'error',
      getExplorerUrl(e.signature, cluster, explorer)
    );

    throw e;
  }
};

export const PostedOrdersTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  explorer,
  pools,
  markets
}: {
  data: OpenOrder[];
  market: MarketAndConfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
}) => {
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [ordersPendingDeletion, setOrdersPendingDeletion] = useState<string[]>([]);

  const columns = useMemo(
    () =>
      getPostOrderColumns({
        market,
        marginAccount,
        provider,
        cluster,
        explorer,
        pools,
        markets,
        refreshOrderBooks,
        ordersPendingDeletion,
        setOrdersPendingDeletion
      }),
    [market, marginAccount, provider, cluster, explorer, ordersPendingDeletion]
  );

  return (
    <Table
      className="debt-table"
      columns={columns}
      dataSource={data}
      pagination={{
        hideOnSinglePage: true
      }}
      rowKey="order_id"
      locale={{ emptyText: 'No Data' }}
      rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
    />
  );
};
