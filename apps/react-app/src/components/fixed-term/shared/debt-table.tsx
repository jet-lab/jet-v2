import { useRecoilState, useRecoilValue } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
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
import { useEffect } from 'react';
import { notify } from '@utils/notify';

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
      let total = new BN(0);
      fills.map(f => {
        total = total.add(new BN(f.quote_filled));
      });
      return `USDC ${new TokenAmount(new BN(total), 6).tokens.toFixed(2)}`;
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
    render: () => <CloseOutlined style={{ color: '#e36868' }} onClick={() => {}} /> // color: --dt-danger
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

const OrdersTable = ({ data }: { data: FixedOrder[] }) => {
  return (
    <Table
      className={'debt-table'}
      columns={postOrderColumns}
      dataSource={data}
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
            children: loading ? <LoadingOutlined /> : <OrdersTable data={data || []} />
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
