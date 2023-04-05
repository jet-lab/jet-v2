import { FixedTermMarket, MarginAccount, MarketAndConfig, Pool, TokenAmount } from '@jet-lab/margin';
import { Deposit } from '@jet-lab/store';
import { Table } from 'antd';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';
import { useMemo } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { ColumnsType } from 'antd/lib/table';
const getDepositsColumns = (market: MarketAndConfig): ColumnsType<Deposit> => [
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
    title: 'Balance',
    dataIndex: 'balance',
    key: 'balance',
    render: (value: number) =>
      `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.balance - b.balance,
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

export const OpenDepositsTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  explorer
}: {
  data: Deposit[];
  market: MarketAndConfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
}) => {
  const columns = useMemo(() => getDepositsColumns(market), [market, marginAccount, provider, cluster, explorer]);
  return (
    <Table
      rowKey="address"
      className={'debt-table'}
      columns={columns}
      dataSource={data}
      pagination={{
        hideOnSinglePage: true
      }}
      locale={{ emptyText: 'No Data' }}
      rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
    />
  );
};
