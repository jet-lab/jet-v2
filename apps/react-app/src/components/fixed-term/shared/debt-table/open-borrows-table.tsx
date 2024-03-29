import {
  FixedTermMarket,
  MarginAccount,
  MarketAndConfig,
  Pool,
  TokenAmount,
  toggleAutorollPosition
} from '@jet-lab/margin';
import { Switch, Table } from 'antd';
import { ColumnsType } from 'antd/lib/table';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';
import { AnchorProvider } from '@project-serum/anchor';
import { Dispatch, SetStateAction, useMemo, useState } from 'react';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { LoadingOutlined } from '@ant-design/icons';
import { AutoRollChecks } from '../autoroll-checks';
import { AutoRollModal } from '../autoroll-modal';

const getBorrowColumns = (
  market: MarketAndConfig,
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  setPendingPositions: Dispatch<SetStateAction<string[]>>,
  pendingPositions: string[],
  setShowAutorollModal: Dispatch<SetStateAction<boolean>>,
  showAutorollModal: boolean,
  lookupTables: LookupTable[]
): ColumnsType<Loan> => [
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
    render: (date: number) =>
      Date.now() < date
        ? `${formatDistanceToNowStrict(date.toString().length === 10 ? date * 1000 : date)} from now`
        : `${formatDistanceToNowStrict(date.toString().length === 10 ? date * 1000 : date)} ago`,
    sorter: (a, b) => a.maturation_timestamp - b.maturation_timestamp,
    sortDirections: ['descend']
  },
  {
    title: 'Principal',
    dataIndex: 'principal',
    key: 'principal',
    render: (value: number) =>
      `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.principal - b.principal,
    sortDirections: ['descend']
  },
  {
    title: 'Remaining Balance',
    dataIndex: 'remaining_balance',
    key: 'remaining_balance',
    render: (value: number) =>
      `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.remaining_balance - b.remaining_balance,
    sortDirections: ['descend']
  },
  {
    title: 'Interest',
    dataIndex: 'interest',
    key: 'interest',
    render: (value: number) =>
      `${market.token.symbol} ${new TokenAmount(new BN(value), market.token.decimals).tokens.toFixed(2)}`,
    sorter: (a, b) => a.interest - b.interest,
    sortDirections: ['descend']
  },
  {
    title: 'Autoroll',
    key: 'is_auto_roll',
    align: 'center',
    render: (position: Loan) => {
      return pendingPositions.includes(position.address) ? (
        <LoadingOutlined />
      ) : (
        <AutoRollChecks market={market.market} marginAccount={marginAccount}>
          {({ hasConfig, refresh, borrowRate, lendRate }) => (
            <div className="auto-roll-controls">
              <AutoRollModal
                onClose={() => {
                  setShowAutorollModal(false);
                }}
                open={showAutorollModal}
                marketAndConfig={market}
                marginAccount={marginAccount}
                refresh={refresh}
                borrowRate={borrowRate}
                lendRate={lendRate}
              />
              <Switch
                className="debt-table-switch"
                checked={position.is_auto_roll}
                disabled={position.principal + position.interest < market.config.minBaseOrderSize}
                onClick={() => {
                  if (hasConfig) {
                    togglePosition(
                      marginAccount,
                      market,
                      provider,
                      position,
                      pools,
                      markets,
                      cluster,
                      explorer,
                      pendingPositions,
                      setPendingPositions,
                      lookupTables
                    );
                  } else {
                    setShowAutorollModal(true);
                  }
                }}
              />
            </div>
          )}
        </AutoRollChecks>
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

const togglePosition = async (
  marginAccount: MarginAccount,
  market: MarketAndConfig,
  provider: AnchorProvider,
  position: Loan,
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pendingPositions: string[],
  setPendingPositions: Dispatch<SetStateAction<string[]>>,
  lookupTables: LookupTable[]
) => {
  try {
    setPendingPositions([...pendingPositions, position.address]);
    await toggleAutorollPosition({
      marginAccount,
      market: market.market,
      provider,
      position,
      pools,
      markets,
      lookupTables
    });
    notify('Autoroll toggled', 'Your term loan autoroll settings have been succsesfully toggled', 'success');
  } catch (e: any) {
    notify(
      'Failed to toggle autoroll',
      'We were unable to toggle your term loan autoroll settings, please try again.',
      'error',
      getExplorerUrl(e.signature, cluster, explorer)
    );
    console.error(e);
  } finally {
    setPendingPositions(pendingPositions.filter(p => p !== position.address));
  }
};

interface IOpenBorrowsTable {
  data: Loan[];
  market: MarketAndConfig;
  marginAccount: MarginAccount;
  provider: AnchorProvider;
  pools: Record<string, Pool>;
  markets: FixedTermMarket[];
  cluster: 'mainnet-beta' | 'localnet' | 'devnet';
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach';
  lookupTables: LookupTable[];
}

export const OpenBorrowsTable = ({
  data,
  market,
  provider,
  marginAccount,
  cluster,
  explorer,
  pools,
  markets,
  lookupTables
}: IOpenBorrowsTable) => {
  const [pendingPositions, setPendingPositions] = useState<string[]>([]);
  const [showAutorollModal, setShowAutorollModal] = useState(false);
  const columns = useMemo(
    () =>
      getBorrowColumns(
        market,
        marginAccount,
        provider,
        cluster,
        explorer,
        pools,
        markets,
        setPendingPositions,
        pendingPositions,
        setShowAutorollModal,
        showAutorollModal,
        lookupTables
      ),
    [market, marginAccount, provider, cluster, explorer, pendingPositions, setPendingPositions, showAutorollModal]
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
