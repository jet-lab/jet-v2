import { FixedTermMarket, MarginAccount, Pool, TokenAmount } from '@jet-lab/margin';
import { Table } from 'antd';
import { CheckOutlined, CloseOutlined, LoadingOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import formatDistanceToNowStrict from 'date-fns/formatDistanceToNowStrict';
import BN from 'bn.js';
import { cancelOrder, MarketAndConfig } from '@jet-lab/margin';
import { Dispatch, SetStateAction, useMemo, useState } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { getExplorerUrl } from '@utils/ui';
import { notify } from '@utils/notify';

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
  lookupTables: LookupTable[];
}
const getPostOrderColumns = ({
  market,
  marginAccount,
  provider,
  cluster,
  explorer,
  pools,
  markets,
  ordersPendingDeletion,
  setOrdersPendingDeletion,
  lookupTables
}: GetPostOrderColumnes): ColumnsType<OpenOrder> => [
    {
      title: 'Issue date',
      dataIndex: 'created_timestamp',
      key: 'created_timestamp',
      render: (date: number) => `${formatDistanceToNowStrict(date.toString().length === 10 ? date * 1000 : date)} ago`,
      sorter: (a, b) => a.created_timestamp - b.created_timestamp,
      sortDirections: ['descend']
    },
    {
      title: 'Total QTY',
      dataIndex: 'total_quote_qty',
      key: 'total_quote_qty',
      render: (value: number) =>
        `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`,
      sorter: (a, b) => a.total_quote_qty - b.total_quote_qty,
      sortDirections: ['descend']
    },
    {
      title: 'Filled QTY',
      dataIndex: 'filled_quote_qty',
      key: 'filled_quote_qty',
      render: (filled: number) => {
        return `${market.token.symbol} ${new TokenAmount(new BN(filled), market.token.decimals).tokens.toFixed(2)}`;
      },
      sorter: (a, b) => a.filled_quote_qty - b.filled_quote_qty,
      sortDirections: ['descend']
    },
    {
      title: 'Rate',
      dataIndex: 'rate',
      key: 'rate',
      render: (rate: number) => `${(100 * rate).toFixed(3)}%`,
      sorter: (a, b) => a.rate - b.rate,
      sortDirections: ['descend']
    },
    {
      title: 'Autoroll',
      dataIndex: 'is_auto_roll',
      key: 'is_auto_roll',
      align: 'center',
      render: (is_auto_roll: boolean) => {
        return is_auto_roll ? <CheckOutlined /> : null;
      },
      sorter: (a, b) => Number(a.is_auto_roll) - Number(b.is_auto_roll),
      sortDirections: ['descend']
    },
    {
      title: 'Cancel',
      key: 'cancel',
      align: 'center',
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
                ordersPendingDeletion,
                setOrdersPendingDeletion,
                lookupTables
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
  ordersPendingDeletion: string[],
  setOrdersPendingDeletion: UpdateOrders,
  lookupTables: LookupTable[]
) => {
  try {
    await cancelOrder({
      market,
      marginAccount,
      provider,
      orderId: new BN(order.order_id),
      pools,
      markets,
      lookupTables
    });
    notify('Order Cancelled', 'Your order was cancelled successfully', 'success');
    setOrdersPendingDeletion([...ordersPendingDeletion, order.order_id]);
  } catch (e: any) {
    notify(
      'Cancel order failed',
      'There was an error cancelling your order',
      'error',
      getExplorerUrl(e.signature, cluster, explorer)
    );
    console.error(e);
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
  markets,
  lookupTables
}: {
  data: OpenOrder[];
  market: MarketAndConfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
  lookupTables: LookupTable[];
}) => {
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
        ordersPendingDeletion,
        setOrdersPendingDeletion,
        lookupTables
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
