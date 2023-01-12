import { FixedTermMarket, MarginAccount, MarketAndconfig, Pool, TokenAmount } from '@jet-lab/margin';
import { Deposit } from '@jet-lab/store';
import { Table } from 'antd';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';
import { useMemo } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
const getDepositsColumns = (
  market: MarketAndconfig,
) => [
  {
    title: 'Created',
    dataIndex: 'created_timestamp',
    key: 'created_timestamp',
    render: (date: string) => formatDistanceToNowStrict(new Date(date), { addSuffix: true })
  },
  {
    title: 'Maturity',
    dataIndex: 'maturation_timestamp',
    key: 'maturation_timestamp',
    render: (date: string) => formatDistanceToNowStrict(new Date(date), { addSuffix: true })
  },
  {
    title: 'Balance',
    dataIndex: 'balance',
    key: 'balance',
    render: (value: number) =>
      `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`
  },
  {
    title: 'Rate',
    dataIndex: 'rate',
    key: 'rate',
    render: (rate: number) => `${rate}%`
  }
];

export const OpenDepositsTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  blockExplorer,
}: {
  data: Deposit[];
  market: MarketAndconfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
}) => {
  const columns = useMemo(
    () => getDepositsColumns(market),
    [market, marginAccount, provider, cluster, blockExplorer]
  );
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
