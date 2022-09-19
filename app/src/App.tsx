import './styles/App.less';
import { RecoilRoot } from 'recoil';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { WalletProvider } from '@solana/wallet-adapter-react';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom';
import { MathWalletAdapter } from '@solana/wallet-adapter-mathwallet';
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare';
import { SolongWalletAdapter } from '@solana/wallet-adapter-solong';
import { SolletWalletAdapter } from '@solana/wallet-adapter-sollet';
import { BraveWalletAdapter } from '@solana/wallet-adapter-brave';
import { E2EWalletAdapter } from '@jet-lab/e2e-react-adapter';
import { PoolsView } from './views/PoolsView';
import { SwapsView } from './views/SwapsView';
import { AccountsView } from './views/AccountsView';
import { Navbar } from './components/misc/Navbar/Navbar';
import { Modals } from './components/modals/Modals';
import { TermsPrivacy } from './components/misc/TermsPrivacy';
import { lazy, Suspense } from 'react';
import { Navigate } from 'react-router-dom';

const StateSyncer = lazy(() => import('./state/StateSyncer'));
const FixedLendView = lazy(() => import('./views/FixedLendView'));
const FixedBorrowView = lazy(() => import('./views/FixedBorrowView'));

export function App(): JSX.Element {
  const wallets = [
    new PhantomWalletAdapter(),
    new BraveWalletAdapter(),
    new SolflareWalletAdapter(),
    new SolongWalletAdapter(),
    new MathWalletAdapter(),
    new SolletWalletAdapter(),
    new E2EWalletAdapter()
  ];

  const isDev = window.location.href.includes('http://localhost:3000');

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
                isDev ? (
                  <Suspense fallback={<></>}>
                    <FixedBorrowView />
                  </Suspense>
                ) : (
                  <Navigate to="/" />
                )
              }
            />
            <Route
              path="/fixed-lend"
              element={
                isDev ? (
                  <Suspense fallback={<></>}>
                    <FixedLendView />
                  </Suspense>
                ) : (
                  <Navigate to="/" />
                )
              }
            />
          </Routes>
          <Modals />
          <TermsPrivacy />
          <Suspense fallback={<></>}>
            <StateSyncer />
          </Suspense>
        </WalletProvider>
      </RecoilRoot>
    </BrowserRouter>
  );
}
