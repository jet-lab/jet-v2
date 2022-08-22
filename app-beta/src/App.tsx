import './styles/App.less';
import { useMemo } from 'react';
import { QueryClient, QueryClientProvider } from 'react-query';
import { HashRouter, Route, Routes } from 'react-router-dom';
import { LocalizationProvider } from './contexts/localization/localization';
import { WalletProvider } from '@solana/wallet-adapter-react';
import {
  PhantomWalletAdapter,
  MathWalletAdapter,
  SolflareWalletAdapter,
  SolongWalletAdapter,
  SolletWalletAdapter,
  SlopeWalletAdapter,
  BraveWalletAdapter
} from '@solana/wallet-adapter-wallets';
import { E2EWalletAdapter } from '@jet-lab/e2e-react-adapter';
import { MarginContextProvider } from './contexts/marginContext';
import { RpcNodeContextProvider } from './contexts/rpcNode';
import { BlockExplorerProvider } from './contexts/blockExplorer';
import { TransactionsProvider } from './contexts/transactionLogs';
import { TradeContextProvider } from './contexts/tradeContext';
import { NativeValuesProvider } from './contexts/nativeValues';
import { ConnectWalletModalProvider } from './contexts/connectWalletModal';
import { LiquidationModalProvider } from './contexts/LiquidationModal';
import { SettingsModalProvider } from './contexts/settingsModal';
import { RadarModalProvider } from './contexts/radarModal';
import { ConnectWalletModal } from './components/ConnectWalletModal';
import { Settings } from './views/Settings';
import { RadarModal } from './components/RadarModal';
import { DisclaimerModal } from './components/DisclaimerModal';
import { TermsPrivacy } from './components/TermsPrivacy';
import { Navbar } from './components/Navbar';
import { NetworkWarningBanner } from './components/NetworkWarningBanner';
import { Cockpit } from './views/Cockpit';
import { TransactionLogs } from './views/TransactionLogs';
import { LiquidationModal } from './components/LiquidationModal';
import { Keypair } from '@solana/web3.js';
import { bs58 } from '@project-serum/anchor/dist/cjs/utils/bytes';
import { ClusterSettingProvider } from './contexts/clusterSetting';

const queryClient = new QueryClient();
export function App(): JSX.Element {
  const urlParams = new URLSearchParams(window.location.search);
  const debugWallet: string = urlParams.get('debug-wallet-secret-key') as string;
  const isDevnet = window.location.href.includes('devnet') || window.location.href.includes('localhost');
  const wallets = useMemo(() => {
    const walletArray: (
      | PhantomWalletAdapter
      | BraveWalletAdapter
      | MathWalletAdapter
      | SolflareWalletAdapter
      | SolongWalletAdapter
      | SolletWalletAdapter
      | SlopeWalletAdapter
      | E2EWalletAdapter
    )[] = [
      new PhantomWalletAdapter(),
      new BraveWalletAdapter(),
      new MathWalletAdapter(),
      new SolflareWalletAdapter(),
      new SolongWalletAdapter(),
      new SolletWalletAdapter(),
      new SlopeWalletAdapter()
    ];
    if (isDevnet) {
      walletArray.push(
        new E2EWalletAdapter(
          debugWallet && debugWallet.length > 0
            ? {
                keypair: Keypair.fromSecretKey(bs58.decode(debugWallet))
              }
            : undefined
        )
      );
    }
    return walletArray;
  }, [debugWallet, isDevnet]);

  return (
    <HashRouter basename={'/'}>
      <QueryClientProvider client={queryClient}>
        <LocalizationProvider>
          <ClusterSettingProvider>
            <WalletProvider wallets={wallets} autoConnect>
              <MarginContextProvider>
                <SettingsModalProvider>
                  <RpcNodeContextProvider>
                    <ConnectWalletModalProvider>
                      <LiquidationModalProvider>
                        <BlockExplorerProvider>
                          <TransactionsProvider>
                            <TradeContextProvider>
                              <NativeValuesProvider>
                                <RadarModalProvider>
                                  <NetworkWarningBanner />
                                  <Navbar />
                                  <Routes>
                                    <Route path="/" element={<Cockpit />} />
                                    <Route path="/transactions" element={<TransactionLogs />} />
                                  </Routes>
                                  <ConnectWalletModal />
                                  <Settings />
                                  <RadarModal />
                                  <DisclaimerModal />
                                  <LiquidationModal />
                                  <TermsPrivacy />
                                </RadarModalProvider>
                              </NativeValuesProvider>
                            </TradeContextProvider>
                          </TransactionsProvider>
                        </BlockExplorerProvider>
                      </LiquidationModalProvider>
                    </ConnectWalletModalProvider>
                  </RpcNodeContextProvider>
                </SettingsModalProvider>
              </MarginContextProvider>
            </WalletProvider>
          </ClusterSettingProvider>
        </LocalizationProvider>
      </QueryClientProvider>
    </HashRouter>
  );
}
