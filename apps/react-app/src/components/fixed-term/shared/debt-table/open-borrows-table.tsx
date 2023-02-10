import { MarginTokenConfig, MarketAndConfig, TokenAmount } from '@jet-lab/margin';
import { Loan } from '@jet-lab/store';
import { Table } from 'antd';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';

const getBorrowColumns = (token: MarginTokenConfig) => [
  {
    title: 'Created',
    dataIndex: 'created_timestamp',
    key: 'created_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date), { addSuffix: true })}`
  },
  {
    title: 'Maturity',
    dataIndex: 'maturation_timestamp',
    key: 'maturation_timestamp',
    render: (date: string) => `${formatDistanceToNowStrict(new Date(date), { addSuffix: true })}`
  },
  {
    title: 'Balance',
    dataIndex: 'balance',
    key: 'balance',
    render: (value: number) => `${token.symbol} ${new TokenAmount(new BN(value), token.decimals).tokens.toFixed(2)}`
  },
  {
    title: 'Rate',
    dataIndex: 'rate',
    key: 'rate',
    render: (rate: number) => `${100 * rate}%`
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
