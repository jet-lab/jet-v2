import { useEffect } from 'react';
import { atom, useRecoilValue, useSetRecoilState } from 'recoil';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { MarginAccount, MarginWalletTokens } from '@jet-lab/margin';
import { ActionRefresh, ACTION_REFRESH_INTERVAL } from '../actions/actions';
import { useProvider } from '../../utils/jet/provider';

export const WalletTokens = atom({
  key: 'walletTokens',
  default: undefined as MarginWalletTokens | undefined
});
export const WalletLoading = atom({
  key: 'walletLoading',
  default: false as boolean
});
export const WalletInit = atom({
  key: 'walletInit',
  default: false as boolean
});

// Wrapper to provide contextual updates to Wallet
export function WalletTokensWrapper(props: { children: JSX.Element }) {
  const { programs, provider } = useProvider();
  const { publicKey } = useWallet();
  const walletParam = new URLSearchParams(document.location.search).get('wallet');
  const walletKey = publicKey ?? (walletParam ? new PublicKey(walletParam) : null);
  const setWalletTokens = useSetRecoilState(WalletTokens);
  const setWalletLoading = useSetRecoilState(WalletLoading);
  const setWalletInit = useSetRecoilState(WalletInit);
  const actionRefresh = useRecoilValue(ActionRefresh);

  // Fetch wallet tokens on init / wallet / programs change, and update on interval
  // Re-fetch upon an actionRefresh
  useEffect(() => {
    async function getWalletTokens() {
      if (!programs || !walletKey) {
        return;
      }

      setWalletLoading(true);
      try {
        const walletTokens = await MarginAccount.loadTokens(programs, walletKey);
        setWalletTokens(walletTokens);
      } catch (err) {
        console.error(err);
      }
      setWalletInit(true);
      setWalletLoading(false);
    }

    getWalletTokens();
    const walletTokensInterval = setInterval(getWalletTokens, ACTION_REFRESH_INTERVAL);
    return () => clearInterval(walletTokensInterval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [provider.connection, publicKey, actionRefresh]);

  return <>{props.children}</>;
}
