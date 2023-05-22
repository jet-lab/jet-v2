type Explorer = 'solanaExplorer' | 'solscan' | 'solanaBeach';

type Cluster = 'localnet' | 'devnet' | 'mainnet-beta';
type NodeOption = 'default' | 'custom';

interface RPC {
  name: string;
  devnet: string;
  localnet: string;
  'mainnet-beta': string;
  pings: Record<Cluster, number>;
}

const rpc = {
  name: 'Default',
  devnet: `https://jetprot-develope-26c4.devnet.rpcpool.com/${process.env.REACT_APP_RPC_DEV_TOKEN ?? ''}`,
  'mainnet-beta': `https://jetprot-main-0d7b.mainnet.rpcpool.com/${process.env.REACT_APP_RPC_TOKEN ?? ''}`,
  localnet: 'http://localhost:8899',
  pings: {
    'mainnet-beta': 0,
    devnet: 0,
    localnet: 0
  }
};

interface Settings {
  cluster: Cluster;
  explorer: Explorer;
  rpc: RPC;
}
interface SettingsSlice {
  settings: Settings;
  updateSettings: (payload: Partial<Settings>) => void;
}
