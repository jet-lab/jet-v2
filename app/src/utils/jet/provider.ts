import { useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Connection, ConfirmOptions } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { AnchorProvider, Wallet } from '@project-serum/anchor';
import { MarginClient } from '@jet-lab/margin';
import { Cluster, rpcNodes, PreferredRpcNode } from '@state/settings/settings';
import { MainConfig } from '@state/config/marginConfig';
import { NetworkStateAtom } from '@state/network/network-state';

// Anchor connection / provider hook
export function useProvider() {
  const cluster = useRecoilValue(Cluster);
  const node = useRecoilValue(PreferredRpcNode);
  const networkStatus = useRecoilValue(NetworkStateAtom);
  const endpoint =
    cluster === 'localnet'
      ? 'http://localhost:8899'
      : rpcNodes[node][cluster === 'mainnet-beta' ? 'mainnetBeta' : cluster];
  const connection = useMemo(() => new Connection(endpoint, 'recent'), [endpoint]);
  const config = useRecoilValue(MainConfig);
  const wallet = useWallet();

  const provider = useMemo(() => {
    const confirmOptions = {
      skipPreflight: true,
      commitment: 'recent',
      preflightCommitment: 'recent'
    } as ConfirmOptions;

    return new AnchorProvider(connection, wallet as unknown as Wallet, confirmOptions);
  }, [connection, wallet]);

  const programs = useMemo(
    () => (config && networkStatus === 'connected' ? MarginClient.getPrograms(provider, config) : undefined),
    [config, provider, networkStatus]
  );
  return { programs, provider };
}
