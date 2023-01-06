import { MarginAccount, TokenAmount } from '@jet-lab/margin';
import { Table } from 'antd';
import { CloseOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import formatDistanceToNowStrict from 'date-fns/formatDistanceToNowStrict';
import BN from 'bn.js';
import { cancelOrder, MarketAndconfig } from '@jet-lab/margin';
import { useMemo } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { getExplorerUrl } from '@utils/ui';
import { OpenOrder } from '@jet-lab/store/dist/types';
import { notify } from '@utils/notify';

interface GetPostOrderColumnes {
  market: MarketAndconfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
}
const getPostOrderColumns = ({
  market,
  marginAccount,
  provider,
  cluster,
  blockExplorer
}: GetPostOrderColumnes): ColumnsType<OpenOrder> => [
  {
    title: 'Issue date',
    dataIndex: 'created_timestamp',
    key: 'created_timestamp',
    render: (date: number) => `${formatDistanceToNowStrict(date)} ago`
  },
  {
    title: 'Total QTY',
    dataIndex: 'total_quote_qty',
    key: 'total_quote_qty',
    render: (value: number) => `${market.token.symbol} ${new TokenAmount(new BN(value), 6).tokens.toFixed(2)}`
  },
  {
    title: 'Filled QTY',
    dataIndex: 'filled_quote_qty',
    key: 'filled_quote_qty',
    render: (filled: number) => {
      return `${market.token.symbol} ${new TokenAmount(new BN(filled), 6).tokens.toFixed(2)}`;
    }
  },
  {
    title: 'Rate',
    dataIndex: 'rate',
    key: 'rate',
    render: (rate: number) => `${rate}%`
  },
  {
    title: 'Cancel',
    key: 'cancel',
    render: order => {
      return (
        <CloseOutlined
          style={{ color: '#e36868' }}
          onClick={() => cancel(market, marginAccount, provider, order, cluster, blockExplorer)}
        />
      );
    }
  }
];

const cancel = async (
  market: MarketAndconfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  order: OpenOrder,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach'
) => {
  try {
    await cancelOrder({
      market,
      marginAccount,
      provider,
      orderId: new BN(order.order_id)
    });
    notify('Order Cancelled', 'Your order was cancelled successfully', 'success');
  } catch (e: any) {
    notify(
      'Cancel order failed',
      'There was an error cancelling your order',
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
    throw e;
  }
};

export const OrdersTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  blockExplorer
}: {
  data: OpenOrder[];
  market: MarketAndconfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
}) => {
  const columns = useMemo(
    () =>
      getPostOrderColumns({
        market,
        marginAccount,
        provider,
        cluster,
        blockExplorer
      }),
    [market, marginAccount, provider, cluster, blockExplorer]
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
