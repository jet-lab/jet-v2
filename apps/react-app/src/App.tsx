import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { RecoilRoot } from 'recoil';
import { E2EWalletAdapter } from '@jet-lab/e2e-react-adapter';
import { WalletProvider } from '@solana/wallet-adapter-react';
import {
  // PhantomWalletAdapter,
  BackpackWalletAdapter,
  MathWalletAdapter,
  SolflareWalletAdapter,
  SolongWalletAdapter,
  BraveWalletAdapter
} from '@solana/wallet-adapter-wallets';
import { lazy, Suspense, useMemo } from 'react';
import './styles/App.less';
import { Navbar } from '@components/misc/Navbar/Navbar';
import { Modals } from '@components/modals/Modals';
import { TermsPrivacy } from '@components/misc/TermsPrivacy';
import '@styles/App.less';

const AccountsView = lazy(() => import('@views/AccountsView'));
const PoolsView = lazy(() => import('@views/PoolsView'));
const SwapsView = lazy(() => import('@views/SwapsView'));

const StateSyncer = lazy(() => import('@state/StateSyncer'));
const FixedLendView = lazy(() => import('@views/FixedLendView'));
const FixedBorrowView = lazy(() => import('@views/FixedBorrowView'));

const isDebug = window.location.href.includes('localhost');

export const App = (): JSX.Element => {
  const wallets = useMemo(
    () => [
      new BraveWalletAdapter(),
      new SolflareWalletAdapter(),
      new SolongWalletAdapter(),
      new MathWalletAdapter(),
      ...(isDebug ? [new E2EWalletAdapter()] : [])
    ],
    [isDebug]
  );

  return (
    <BrowserRouter>
      <RecoilRoot>
        <WalletProvider wallets={wallets} autoConnect localStorageKey="jetAppWallet">
          <Navbar />
          <Routes>
            <Route
              path="/"
              element={
                <Suspense>
                  <PoolsView />
                </Suspense>
              }
            />
            <Route
              path="/swaps"
              element={
                <Suspense>
                  <SwapsView />
                </Suspense>
              }
            />
            <Route
              path="/accounts"
              element={
                <Suspense>
                  <AccountsView />
                </Suspense>
              }
            />

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
          <Suspense>
            <StateSyncer />
          </Suspense>
        </WalletProvider>
      </RecoilRoot>
    </BrowserRouter>
  );
};
