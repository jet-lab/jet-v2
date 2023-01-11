import { FixedTermMarket, MarginAccount, MarketAndconfig, Pool, redeem, TokenAmount } from '@jet-lab/margin';
import { Deposit } from '@jet-lab/store';
import { Button, Table } from 'antd';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';
import { useMemo } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
const getDepositsColumns = (
  market: MarketAndconfig,
  markets: FixedTermMarket[],
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pools: Record<string, Pool>
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
  },
  {
    title: 'Actions',
    key: 'actions',
    render: (deposit: Deposit) => {
      const maturationDate = new Date(deposit.maturation_timestamp);
      if (maturationDate.getTime() <= Date.now()) {
        return (
          <Button
            size="small"
            onClick={() =>
              redeemDeposit(market, marginAccount, provider, deposit, cluster, blockExplorer, pools, markets)
            }>
            Claim
          </Button>
        );
      }
      return null;
    }
  }
];

const redeemDeposit = async (
  market: MarketAndconfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  deposit: Deposit,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pools: Record<string, Pool>,
  markets: FixedTermMarket[]
) => {
  try {
    await redeem({
      market,
      marginAccount,
      provider,
      pools,
      markets,
      deposit
    });
    notify('Deposit Redeemed', 'Your deposit was successfully redeem', 'success');
  } catch (e: any) {
    notify(
      'Deposit redemption failed',
      'There was an error redeeming your deposit',
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
    throw e;
  }
};

export const OpenDepositsTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  blockExplorer,
  pools,
  markets
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
    () => getDepositsColumns(market, markets, marginAccount, provider, cluster, blockExplorer, pools),
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
