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

interface Settings {
  cluster: Cluster;
  explorer: Explorer;
  rpc: RPC;
}
interface SettingsSlice {
  settings: Settings;
  updateSettings: (payload: Partial<Settings>) => void;
}
