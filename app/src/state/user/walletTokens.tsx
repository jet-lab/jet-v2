import { useEffect } from 'react';
import { atom, useRecoilValue, useSetRecoilState } from 'recoil';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MarginAccount, MarginWalletTokens } from '@jet-lab/margin';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { useProvider } from '@utils/jet/provider';

// If user wants to view someone else's accounts
export const walletParam = new URLSearchParams(document.location.search).get('wallet');

// The connected solana wallet's token balances
export const WalletTokens = atom({
  key: 'walletTokens',
  dangerouslyAllowMutability: true,
  default: undefined as MarginWalletTokens | undefined
});

// A syncer to be called so that we can have dependent atom state
export function useWalletTokensSyncer() {
  const { programs, provider } = useProvider();
  const { publicKey } = useWallet();
  const walletKey = publicKey ?? (walletParam ? new PublicKey(walletParam) : null);
  const setWalletTokens = useSetRecoilState(WalletTokens);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Fetch wallet tokens on wallet connection
  useEffect(() => {
    async function getWalletTokens() {
      if (!programs || !walletKey) {
        return;
      }

      const walletTokens = await MarginAccount.loadTokens(programs, walletKey);
      setWalletTokens(walletTokens);
    }

    getWalletTokens();
    const walletTokensInterval = setInterval(getWalletTokens, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(walletTokensInterval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [publicKey, provider.connection, actionRefresh]);

  return <></>;
}
