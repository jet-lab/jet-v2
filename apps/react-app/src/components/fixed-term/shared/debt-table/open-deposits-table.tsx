import {
  FixedTermMarket,
  MarginAccount,
  MarketAndConfig,
  Pool,
  TokenAmount,
  toggleAutorollPosition
} from '@jet-lab/margin';
import { Switch, Table } from 'antd';
import BN from 'bn.js';
import { formatDistanceToNowStrict } from 'date-fns';
import { Dispatch, SetStateAction, useMemo, useState } from 'react';
import { AnchorProvider } from '@project-serum/anchor';
import { ColumnsType } from 'antd/lib/table';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { LoadingOutlined } from '@ant-design/icons';
import { AutoRollChecks } from '../autoroll-checks';
import { AutoRollModal } from '../autoroll-modal';

const getDepositsColumns = (
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
  showAutorollModal: boolean
): ColumnsType<Deposit> => [
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
    render: (position: Deposit) => {
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
                      setPendingPositions
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
  position: Deposit,
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  explorer: 'solanaExplorer' | 'solscan' | 'solanaBeach',
  pendingPositions: string[],
  setPendingPositions: Dispatch<SetStateAction<string[]>>
) => {
  try {
    setPendingPositions([...pendingPositions, position.address]);
    await toggleAutorollPosition({
      marginAccount,
      market: market.market,
      provider,
      position,
      pools,
      markets
    });
    notify('Autoroll toggled', 'Your term deposit autoroll settings have been succsesfully toggled', 'success');
  } catch (e: any) {
    notify(
      'Failed to toggle autoroll',
      'We were unable to toggle your term deposit autoroll settings, please try again.',
      'error',
      getExplorerUrl(e.signature, cluster, explorer)
    );
    console.error(e);
  } finally {
    setPendingPositions(pendingPositions.filter(p => p !== position.address));
  }
};

export const OpenDepositsTable = ({
  data,
  market,
  marginAccount,
  provider,
  cluster,
  explorer,
  pools,
  markets
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
  const [pendingPositions, setPendingPositions] = useState<string[]>([]);
  const [showAutorollModal, setShowAutorollModal] = useState(false);

  const columns = useMemo(
    () =>
      getDepositsColumns(
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
        showAutorollModal
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
