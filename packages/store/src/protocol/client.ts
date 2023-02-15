import { Connection } from '@solana/web3.js';
import { JetWebClient, initModule, MarginWebClient, MarginAccountWebClient } from '@jet-lab/jet-client-web';
import { useJetStore } from '../store';
import { SolanaConnectionAdapter } from './connection-adapter';
import { WalletContextState } from '@solana/wallet-adapter-react';

initModule();

interface ExtendedMargin extends MarginWebClient {
  accounts: () => MarginAccountWebClient[];
}
interface ExtendedJetClient extends JetWebClient {
  margin: () => ExtendedMargin;
}

export let jetClient: ExtendedJetClient | undefined;

export const initJetClient = async (wallet: WalletContextState) => {
  console.log('initializing client for ', wallet.publicKey);
  const { rpc, cluster } = useJetStore.getState().settings;
  const endpoint = rpc[cluster];

  const connection = new Connection(endpoint, 'recent');
  const adapter = new SolanaConnectionAdapter(wallet, connection);
  jetClient =
    adapter.userAddress &&
    (await JetWebClient.connect(adapter.userAddress, adapter, cluster === 'devnet' ? 'devnet0' : 'default'));

  await jetClient?.state().syncAccounts();

  const accounts = jetClient?.margin().accounts();
  console.log(accounts);
};
