import {
  MarginCluster,
  MarginConfig,
  MarginAccount,
  Pool,
  PoolManager,
  AssociatedToken,
  MarginClient
} from '@jet-lab/margin';
import { useWallet } from '@solana/wallet-adapter-react';
import { createContext, useContext, useEffect, useMemo, useState } from 'react';
import { useQuery, useQueryClient } from 'react-query';
import { AnchorProvider } from '@project-serum/anchor';
import { ConfirmOptions, Connection, PublicKey } from '@solana/web3.js';
import { useRpcNode } from './rpcNode';
import { Cluster, useClusterSetting } from './clusterSetting';

interface MarginContextState {
  connection?: Connection;
  manager?: PoolManager;
  config?: MarginConfig;
  poolsFetched: boolean;
  pools?: Record<string, Pool>;
  userFetched: boolean;
  marginAccount?: MarginAccount;
  walletBalances?: Record<string, AssociatedToken>;
  cluster: MarginCluster;
  refresh: () => Promise<void>;
}

const MarginContext = createContext<MarginContextState>({
  poolsFetched: false,
  userFetched: false
} as MarginContextState);

const confirmOptions = {
  skipPreflight: true,
  commitment: 'recent',
  preflightCommitment: 'recent'
} as ConfirmOptions;

const endpoints: Record<Cluster, string> = {
  'mainnet-beta': `https://jetprot-main-0d7b.mainnet.rpcpool.com/${process.env.REACT_APP_RPC_TOKEN ?? ''}`,
  devnet: `https://jetprot-develope-26c4.devnet.rpcpool.com/${process.env.REACT_APP_RPC_DEV_TOKEN ?? ''}`
};

function useProvider(): { manager?: PoolManager } {
  const config = useConfig();
  const { clusterSetting } = useClusterSetting();
  const { preferredNode } = useRpcNode();
  const wallet = useWallet();
  const manager = useMemo(() => {
    if (!config) {
      return;
    }
    const connection = new Connection(preferredNode ?? endpoints[clusterSetting], 'recent');
    const provider = new AnchorProvider(connection, wallet as any, confirmOptions);
    const programs = MarginClient.getPrograms(provider, config);
    return new PoolManager(programs, provider);
  }, [config, clusterSetting, preferredNode, wallet]);
  return { manager };
}

function useConfig() {
  const { clusterSetting } = useClusterSetting();
  const [config, setConfig] = useState<MarginConfig | undefined>(undefined);
  useEffect(() => {
    MarginClient.getConfig(clusterSetting).then(config => setConfig(config));
  }, [clusterSetting]);

  return config;
}

// Trade info context provider
export function MarginContextProvider(props: { children: JSX.Element }): JSX.Element {
  const config = useConfig();
  const { clusterSetting } = useClusterSetting();
  const queryClient = useQueryClient();
  const { publicKey } = useWallet();

  const { manager } = useProvider();
  const { connection } = manager ? manager.provider : { connection: undefined };
  const endpoint = connection?.rpcEndpoint;

  const { data: pools, isFetched: poolsFetched } = useQuery(
    ['pools', endpoint],
    async () => {
      if (manager) {
        return await manager.loadAll();
      }
    },
    { enabled: manager && !!manager.programs }
  );

  const { data: user, isFetched: userFetched } = useQuery(
    ['user', endpoint, publicKey?.toBase58()],
    async () => {
      if (!publicKey || !manager) {
        return;
      }
      const walletTokens = await MarginAccount.loadTokens(manager.programs, publicKey);
      const walletBalances = walletTokens.map;
      let marginAccount: MarginAccount | undefined;
      try {
        marginAccount = await MarginAccount.load({
          programs: manager.programs,
          provider: manager.provider,
          pools,
          walletTokens,
          owner: publicKey,
          seed: 0
        });
      } catch (err) {
        console.log(err);
      }
      return { marginAccount, walletBalances };
    },
    { enabled: manager && !!manager.programs && !!pools && !!publicKey }
  );

  async function refresh() {
    setTimeout(() => {
      queryClient.invalidateQueries('user');
      queryClient.invalidateQueries('pools');
    }, 2000);
  }

  const DEFAULT_WALLET_BALANCES = config
    ? (Object.fromEntries(
        Object.values(config.tokens).map(token => [
          token.symbol,
          AssociatedToken.zeroAux(PublicKey.default, token.decimals)
        ])
      ) as Record<string, AssociatedToken>)
    : undefined;

  return (
    <MarginContext.Provider
      value={{
        connection,
        manager,
        config,
        poolsFetched,
        pools,
        userFetched,
        marginAccount: user?.marginAccount,
        walletBalances: user?.walletBalances ?? DEFAULT_WALLET_BALANCES,
        cluster: clusterSetting,
        refresh
      }}>
      {props.children}
    </MarginContext.Provider>
  );
}

// Trade info hook
export const useMargin = () => {
  const context = useContext(MarginContext);
  return context;
};
