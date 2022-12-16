import { useRecoilState, useRecoilValue } from 'recoil';
import { MarginAccount, TokenAmount } from '@jet-lab/margin';
import { AccountsViewOrder } from '@state/views/views';
import { CurrentAccount } from '@state/user/accounts';
import { Tabs, Table } from 'antd';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { CloseOutlined, LoadingOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useOrdersForUser, FixedOrder, FixedOrderFill } from '@jet-lab/store';
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import formatDistanceToNowStrict from 'date-fns/formatDistanceToNowStrict';
import BN from 'bn.js';
import { useEffect, useMemo } from 'react';
import { notify } from '@utils/notify';
import { cancelOrder, MarketAndconfig } from '@jet-lab/margin';
import { useProvider } from '@utils/jet/provider';
import { AnchorProvider } from '@project-serum/anchor';
import { getExplorerUrl } from '@utils/ui';
import { BlockExplorer, Cluster } from '@state/settings/settings';

const getFilledAmount = (fills: FixedOrderFill[]): BN => {
  let total = new BN(0);
  fills.map((f: FixedOrderFill) => {
    total = total.add(new BN(f.quote_filled));
  });
  return total;
};

const postOrderColumns: ColumnsType<FixedOrder> = [
  {
    title: 'Issue date',
    dataIndex: ['details', 'created_timestamp'],
    key: 'created_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date))} ago`
  },
  {
    title: 'Order side',
    dataIndex: ['details', 'order_type'],
    key: 'order_type'
  },
  {
    title: 'Total QTY',
    dataIndex: ['details', 'total_quote_qty'],
    key: 'orderSize',
    render: (value: number) => `USDC ${new TokenAmount(new BN(value), 6).tokens.toFixed(2)}`
  },
  {
    title: 'Filled QTY',
    dataIndex: ['fills'],
    key: 'filledSize',
    render: (fills: FixedOrderFill[]) => {
      const filled = getFilledAmount(fills);
      return `USDC ${new TokenAmount(filled, 6).tokens.toFixed(2)}`;
    }
  },
  {
    title: 'Rate',
    dataIndex: ['details', 'rate'],
    key: 'rate',
    render: (rate: number) => `${rate}%`
  },
  {
    title: 'Cancel',
    dataIndex: 'cancel',
    key: 'cancel',
    render: cancel => <CloseOutlined style={{ color: '#e36868' }} onClick={() => cancel()} /> // color: --dt-danger
  }
];

const fillOrderColumns = [
  {
    title: 'Side',
    dataIndex: 'fill_side',
    key: 'fill_side'
  },
  {
    title: 'Fill Date',
    dataIndex: 'fill_timestamp',
    key: 'fill_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date))} ago`
  },
  {
    title: 'Maturity Date',
    dataIndex: ['maturation_timestamp'],
    key: 'maturation_timestamp',
    render: (date: string) => `in ${formatDistanceToNowStrict(new Date(date))}`
  },
  {
    title: 'Fill QTY',
    dataIndex: 'quote_filled',
    key: 'quote_filled',
    render: (value: number) => `USDC ${new TokenAmount(new BN(value), 6).tokens.toFixed(2)}`
  },
  {
    title: 'Autoroll',
    dataIndex: '',
    key: 'autoroll'
  }
];

const cancel = async (
  market: MarketAndconfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  order: FixedOrder,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach'
) => {
  try {
    const filled = getFilledAmount(order.fills);
    await cancelOrder({
      market,
      marginAccount,
      provider,
      orderId: new BN(order.id),
      amount: new BN(order.details.total_quote_qty).sub(filled)
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

const OrdersTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  blockExplorer
}: {
  data: FixedOrder[];
  market: MarketAndconfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
}) => {
  const orders = useMemo(
    () =>
      data.reduce((all, item) => {
        if (item.details.order_status !== 'Filled') {
          item.cancel = () => cancel(market, marginAccount, provider, item, cluster, blockExplorer);
        }
        all.push(item);
        return all;
      }, []),
    [data]
  );

  return (
    <Table
      className={'debt-table'}
      columns={postOrderColumns}
      dataSource={orders}
      pagination={{
        hideOnSinglePage: true
      }}
      rowKey="id"
      locale={{ emptyText: 'No Data' }}
      rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
    />
  );
};

const PositionsTable = ({ data }: { data: FixedOrder[] }) => {
  const fills = data.map(d => d.fills).flat();
  return (
    <Table
      rowKey="id"
      className={'debt-table'}
      columns={fillOrderColumns}
      dataSource={fills}
      pagination={{
        hideOnSinglePage: true
      }}
      locale={{ emptyText: 'No Data' }}
      rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
    />
  );
};

// Table to show margin account's transaction history
export function DebtTable(): JSX.Element {
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const account = useRecoilValue(CurrentAccount);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const selectedMarket = useRecoilValue(SelectedFixedTermMarketAtom);
  const market = markets[selectedMarket];
  const { provider } = useProvider();
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);

  const { data, error, loading } = useOrdersForUser(market?.market, account);

  useEffect(() => {
    if (error)
      notify(
        'Error fetching data',
        'There was an unexpected error fetching your orders data, please try again soon',
        'error'
      );
  }, [error]);

  return (
    <div className="debt-detail account-table view-element flex-centered">
      <ConnectionFeedback />
      <Tabs
        defaultActiveKey="open-orders"
        destroyInactiveTabPane={true}
        items={[
          {
            label: 'Open Orders',
            key: 'open-orders',
            children:
              loading || !account ? (
                <LoadingOutlined />
              ) : (
                <OrdersTable
                  data={data || []}
                  provider={provider}
                  market={markets[selectedMarket]}
                  marginAccount={account}
                  cluster={cluster}
                  blockExplorer={blockExplorer}
                />
              )
          },
          {
            label: 'Open Positions',
            key: 'open-positions',
            children: loading ? <LoadingOutlined /> : <PositionsTable data={data || []} />
          }
        ]}
      />
      <ReorderArrows component="debtTable" order={accountsViewOrder} setOrder={setAccountsViewOrder} vertical />
    </div>
  );
}
