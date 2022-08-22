import './styles/App.less';
import { RecoilRoot } from 'recoil';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { WalletProvider } from '@solana/wallet-adapter-react';
import {
  PhantomWalletAdapter,
  MathWalletAdapter,
  SolflareWalletAdapter,
  SolongWalletAdapter,
  SolletWalletAdapter,
  BraveWalletAdapter
} from '@solana/wallet-adapter-wallets';
import { E2EWalletAdapter } from '@jet-lab/e2e-react-adapter';
import { WalletTokensWrapper } from './state/user/walletTokens';
import { AccountsWrapper } from './state/user/accounts';
import { PoolsWrapper } from './state/borrow/pools';
import { PriceHistoryWrapper } from './state/trade/priceHistory';
import { MarketWrapper } from './state/trade/market';
import { RecentTradesWrapper } from './state/trade/recentTrades';
import { Navbar } from './components/misc/Navbar';
import { TradeView } from './views/TradeView';
import { PoolsView } from './views/PoolsView';
import { AccountsView } from './views/AccountsView';
import { Modals } from './components/modals/Modals';
import { TermsPrivacy } from './components/misc/TermsPrivacy';

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

  return (
    <BrowserRouter>
      <RecoilRoot>
        <WalletProvider wallets={wallets} autoConnect localStorageKey="jetAppWallet">
          <WalletTokensWrapper>
            <AccountsWrapper>
              <PoolsWrapper>
                <PriceHistoryWrapper>
                  <RecentTradesWrapper>
                    <MarketWrapper>
                      <Navbar />
                      <Routes>
                        <Route path="/" element={<TradeView />} />
                        <Route path="/pools" element={<PoolsView />} />
                        <Route path="/accounts" element={<AccountsView />} />
                      </Routes>
                      <Modals />
                      <TermsPrivacy />
                    </MarketWrapper>
                  </RecentTradesWrapper>
                </PriceHistoryWrapper>
              </PoolsWrapper>
            </AccountsWrapper>
          </WalletTokensWrapper>
        </WalletProvider>
      </RecoilRoot>
    </BrowserRouter>
  );
}
