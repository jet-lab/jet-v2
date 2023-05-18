import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { RecoilRoot } from 'recoil';
import { WalletProvider } from '@solana/wallet-adapter-react';
import {
  MathWalletAdapter,
  SolflareWalletAdapter,
  SolongWalletAdapter,
  BraveWalletAdapter,
  UnsafeBurnerWalletAdapter
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

export const isDebug = window.location.href.includes('localhost');

export const App = (): JSX.Element => {
  const wallets = useMemo(
    () => [
      new BraveWalletAdapter(),
      new SolflareWalletAdapter(),
      new SolongWalletAdapter(),
      new MathWalletAdapter(),
      ...(isDebug ? [new UnsafeBurnerWalletAdapter()] : [])
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
