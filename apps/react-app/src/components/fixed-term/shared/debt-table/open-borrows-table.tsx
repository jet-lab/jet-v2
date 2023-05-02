import { MarginTokenConfig, MarketAndConfig, TokenAmount } from '@jet-lab/margin';
import { Loan } from '@jet-lab/store';
import { Switch, Table } from 'antd';
import { ColumnsType } from 'antd/lib/table';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';

const getBorrowColumns = (token: MarginTokenConfig): ColumnsType<Loan> => [
  {
    title: 'Created',
    dataIndex: 'created_timestamp',
    key: 'created_timestamp',
    render: (date: number) => `${formatDistanceToNowStrict(date.toString().length === 10 ? date * 1000 : date)} ago`,
    sorter: (a, b) => a.created_timestamp - b.created_timestamp,
    sortDirections: ['descend']
  },
  {
    title: 'Maturity',
    dataIndex: 'maturation_timestamp',
    key: 'maturation_timestamp',
    render: (date: number) => `${formatDistanceToNowStrict(date.toString().length === 10 ? date * 1000 : date)} ago`,
    sorter: (a, b) => a.maturation_timestamp - b.maturation_timestamp,
    sortDirections: ['descend']
  },
  {
    title: 'Principal',
    dataIndex: 'principal',
    key: 'principal',
    render: (value: number) => `${token.symbol} ${new TokenAmount(new BN(value), token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.principal - b.principal,
    sortDirections: ['descend']
  },
  {
    title: 'Remaining Balance',
    dataIndex: 'remaining_balance',
    key: 'remaining_balance',
    render: (value: number) => `${token.symbol} ${new TokenAmount(new BN(value), token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.remaining_balance - b.remaining_balance,
    sortDirections: ['descend']
  },
  {
    title: 'Interest',
    dataIndex: 'interest',
    key: 'interest',
    render: (value: number) => `${token.symbol} ${new TokenAmount(new BN(value), token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.interest - b.interest,
    sortDirections: ['descend']
  },
  {
    title: 'Autoroll',
    dataIndex: 'is_auto_roll',
    key: 'is_auto_roll',
    align: 'center',
    render: (is_auto_roll: boolean) => {
      return (
        <Switch
          className="debt-table-switch"
          checked={is_auto_roll}
          onClick={() => {
            console.log(is_auto_roll);
          }}
        />
      );
    },
    sorter: (a, b) => Number(a.is_auto_roll) - Number(b.is_auto_roll),
    sortDirections: ['descend']
  },
  {
    title: 'Rate',
    dataIndex: 'rate',
    key: 'rate',
    render: (rate: number) => `${(100 * rate).toFixed(3)}%`,
    sorter: (a, b) => a.rate - b.rate,
    sortDirections: ['descend']
  }
];

export const OpenBorrowsTable = ({ data, market }: { data: Loan[]; market: MarketAndConfig }) => {
  return (
    <Table
      rowKey="address"
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
