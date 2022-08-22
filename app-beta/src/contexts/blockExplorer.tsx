import { createContext, useContext, useState } from 'react';
import { useClusterSetting } from './clusterSetting';

// Block explorers
const blockExplorers: Record<string, Record<string, string>> = {
  solanaExplorer: {
    name: 'Solana Explorer',
    img: 'img/explorers/solana_explorer.svg',
    url: 'https://explorer.solana.com/tx/'
  },
  solscan: {
    name: 'Solscan',
    img: 'img/explorers/solscan.svg',
    url: 'https://solscan.io/tx/'
  },
  solanaBeach: {
    name: 'Solana Beach',
    img: 'img/explorers/solana_beach.svg',
    url: 'https://solanabeach.io/transaction/'
  }
};

// Block explorer context
interface BlockExplorer {
  preferredExplorer: string;
  setPreferredExplorer: (blockExplorer: string) => void;
}
const BlockExplorerContext = createContext<BlockExplorer>({
  preferredExplorer: '',
  setPreferredExplorer: () => null
});

// Block explorer context provider
export function BlockExplorerProvider(props: { children: JSX.Element }): JSX.Element {
  const [preferredExplorer, setPreferredExplorer] = useState(
    localStorage.getItem('jetPreferredExplorer') ?? 'solanaExplorer'
  );

  return (
    <BlockExplorerContext.Provider
      value={{
        preferredExplorer,
        setPreferredExplorer
      }}>
      {props.children}
    </BlockExplorerContext.Provider>
  );
}

// Block explorer hook
export const useBlockExplorer = () => {
  const { clusterSetting } = useClusterSetting();
  const { preferredExplorer, setPreferredExplorer } = useContext(BlockExplorerContext);
  const baseUrl = blockExplorers[preferredExplorer].url;
  const clusterParam = clusterSetting === 'devnet' ? '?cluster=devnet' : '';
  return {
    blockExplorers,
    preferredExplorer,
    changePreferredExplorer: (preferredExplorer: string) => {
      localStorage.setItem('jetPreferredExplorer', preferredExplorer);
      setPreferredExplorer(preferredExplorer);
    },
    getExplorerUrl: (txId: string) => baseUrl + txId + clusterParam
  };
};
