import { useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Connection, ConfirmOptions } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { AnchorProvider, Wallet } from '@project-serum/anchor';
import { MarginClient } from '@jet-lab/margin';
import { Cluster, rpcNodes, PreferredRpcNode } from '../../state/settings/settings';
import { MarginConfig } from '../../state/config/marginConfig';

// Anchor connection / provider hook
export function useProvider() {
  const cluster = useRecoilValue(Cluster);
  const node = useRecoilValue(PreferredRpcNode);
  const endpoint = rpcNodes[node][cluster === 'mainnet-beta' ? 'mainnetBeta' : cluster];
  const connection = useMemo(() => new Connection(endpoint, 'recent'), [endpoint]);
  const config = useRecoilValue(MarginConfig);
  const wallet = useWallet();

  const provider = useMemo(() => {
    const confirmOptions = {
      skipPreflight: true,
      commitment: 'recent',
      preflightCommitment: 'recent'
    } as ConfirmOptions;

    return new AnchorProvider(connection, wallet as unknown as Wallet, confirmOptions);
  }, [connection, wallet]);

  const programs = useMemo(() => (config ? MarginClient.getPrograms(provider, config) : undefined), [config, provider]);
  return { programs, provider };
}
