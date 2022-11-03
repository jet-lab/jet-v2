import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { RecoilRoot } from 'recoil';
import { E2EWalletAdapter } from '@jet-lab/e2e-react-adapter';
import { WalletProvider } from '@solana/wallet-adapter-react';
import {
  PhantomWalletAdapter,
  MathWalletAdapter,
  SolflareWalletAdapter,
  SolongWalletAdapter,
  SolletWalletAdapter,
  BraveWalletAdapter
} from '@solana/wallet-adapter-wallets';
import { AccountsView, PoolsView, SwapsView } from './views';
import { lazy, Suspense, useMemo } from 'react';
import './styles/App.less';
import { Navbar } from '@components/misc/Navbar/Navbar';
import { Modals } from '@components/modals/Modals';
import { TermsPrivacy } from '@components/misc/TermsPrivacy';
import '@styles/App.less';

const StateSyncer = lazy(() => import('@state/StateSyncer'));
const FixedLendView = lazy(() => import('@views/FixedLendView'));
const FixedBorrowView = lazy(() => import('@views/FixedBorrowView'));

export const isDebug =
  window.location.href.includes('?debug-environment=true') && window.location.href.includes('localhost');

export function App(): JSX.Element {
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new BraveWalletAdapter(),
      new SolflareWalletAdapter(),
      new SolongWalletAdapter(),
      new MathWalletAdapter(),
      new SolletWalletAdapter(),
      ...(isDebug ? [new E2EWalletAdapter()] : [])
    ],
    [isDebug]
  )

  return (
    <BrowserRouter>
      <RecoilRoot>
        <WalletProvider wallets={wallets} autoConnect localStorageKey="jetAppWallet">
          <Navbar />
          <Routes>
            <Route path="/" element={<PoolsView />} />
            <Route path="/swaps" element={<SwapsView />} />
            <Route path="/accounts" element={<AccountsView />} />

            <Route
              path="/fixed-borrow"
              element={
                <Suspense>
                  <FixedBorrowView />
                </Suspense>
              }
            />
            <Route
              path="/fixed-lend"
              element={
                <Suspense>
                  <FixedLendView />
                </Suspense>
              }
            />
          </Routes>
          <Modals />
          <TermsPrivacy />
          <StateSyncer />
        </WalletProvider>
      </RecoilRoot>
    </BrowserRouter>
  );
}
