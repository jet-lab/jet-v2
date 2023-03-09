import { MarginTokenConfig, MarketAndConfig, TokenAmount } from '@jet-lab/margin';
import { Loan } from '@jet-lab/store';
import { Table } from 'antd';
import { ColumnsType } from 'antd/lib/table';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';

const getBorrowColumns = (token: MarginTokenConfig): ColumnsType<Loan> => [
  {
    title: 'Created',
    dataIndex: 'created_timestamp',
    key: 'created_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date), { addSuffix: true })}`,
    sorter: (a, b) => a.created_timestamp - b.created_timestamp,
    sortDirections: ['descend'],
  },
  {
    title: 'Maturity',
    dataIndex: 'maturation_timestamp',
    key: 'maturation_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date), { addSuffix: true })}`,
    sorter: (a, b) => a.maturation_timestamp - b.maturation_timestamp,
    sortDirections: ['descend'],
  },
  {
    title: 'Balance',
    dataIndex: 'balance',
    key: 'balance',
    render: (value: number) => `${token.symbol} ${new TokenAmount(new BN(value), token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.balance - b.balance,
    sortDirections: ['descend'],
  },
  {
    title: 'Rate',
    dataIndex: 'rate',
    key: 'rate',
    render: (rate: number) => `${(100 * rate).toFixed(3)}%`,
    sorter: (a, b) => a.rate - b.rate,
    sortDirections: ['descend'],
  }
];

export const OpenBorrowsTable = ({ data, market }: { data: Loan[]; market: MarketAndConfig }) => {
  return (
    <Table
      rowKey="id"
      className={'debt-table'}
      columns={getBorrowColumns(market.token)}
      dataSource={data}
      pagination={{
        hideOnSinglePage: true
      }}
      locale={{ emptyText: 'No Data' }}
      rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
    />
  );
};
