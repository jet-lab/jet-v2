import { useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Connection, ConfirmOptions } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { AnchorProvider, Wallet } from '@project-serum/anchor';
import { MarginClient } from '@jet-lab/margin';
import { MainConfig } from '@state/config/marginConfig';
import { NetworkStateAtom } from '@state/network/network-state';
import { useJetStore } from '@jet-lab/store';

// Anchor connection / provider hook
export function useProvider() {
  const { cluster, rpc } = useJetStore(state => ({ cluster: state.settings.cluster, rpc: state.settings.rpc }));
  const networkStatus = useRecoilValue(NetworkStateAtom);
  const endpoint = rpc[cluster];
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

  const programs = useMemo(() => {
    if (config && networkStatus === 'connected') {
      // Allow this to fail, in case the currently connected network state is incompatible
      // with the current versions of the libraries
      try {
        return MarginClient.getPrograms(provider, config);
      } catch (e) {
        console.error('failed to initialize program clients', e);
      }
    }

    return undefined;
  }, [config, provider, networkStatus]);

  return { programs, provider };
}
