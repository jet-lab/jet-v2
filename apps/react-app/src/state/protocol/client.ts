import { useMemo, useEffect } from 'react';
import { atom, useRecoilValue, useSetRecoilState } from 'recoil';
import { Connection, PublicKey, AccountInfo, VersionedTransaction } from '@solana/web3.js';
import { useWallet, WalletContextState } from '@solana/wallet-adapter-react';
import { Cluster, rpcNodes, PreferredRpcNode } from '../../state/settings/settings';
import { JetWebClient, Pubkey, initModule } from '@jet-lab/jet-client-web';

initModule()

// Client object for interacting with the protocol
export const ProtocolClient = atom<JetWebClient | undefined>({
  key: 'protocolClient',
  default: undefined
});

class SolanaConnectionAdapter {
  userAddress?: Pubkey;

  constructor(public wallet: WalletContextState, public connection: Connection) {
    if (wallet.publicKey) {
      this.userAddress = new Pubkey(wallet.publicKey.toBase58());
    }
  }

  async getGenesisHash(): Promise<string> {
    return await this.connection.getGenesisHash();
  }

  async getAccounts(addresses: PublicKey[]): Promise<(AccountInfo<Buffer> | null)[]> {
    try {
      return await this.connection.getMultipleAccountsInfo(addresses);
    } catch (e) {
      console.log(e);
      throw e;
    }
  }

  async getLatestBlockhash(): Promise<any> {
    return await this.connection.getLatestBlockhash();
  }

  async send(transactionDatas: Uint8Array[]): Promise<string[]> {
    try {
      const {
        value: { blockhash, lastValidBlockHeight }
      } = await this.connection.
getLatestBlockhashAndContext();

      const transactions = transactionDatas.map(txData => {
        let tx = VersionedTransaction.deserialize(txData);
        tx.message.recentBlockhash = blockhash;
        return tx;
      });
      console.log(transactionDatas);
      console.log(transactions);
      const signed = [];

      if (!this.wallet.signTransaction) {
        console.log("wallet missing signer function");
        return [];
      }

      for (const tx of transactions) {
        signed.push(await this.wallet.signTransaction(tx));
      }

      console.log(signed);
      const signatures = [];

      for (const tx of signed) {
        const signature = await this.connection.sendTransaction(tx);
        await this.connection.confirmTransaction({ blockhash, lastValidBlockHeight, signature }, 'processed');

        signatures.push(signature);
      }

      return signatures;
    } catch (e) {
      console.log(e);
      throw e;
    }
  }
}

// Create client for using protocol
export function useProtocolClientSyncer() {
  const cluster = useRecoilValue(Cluster);
  const node = useRecoilValue(PreferredRpcNode);
  const setProtocolClient = useSetRecoilState(ProtocolClient);

  const endpoint =
    cluster === 'localnet'
      ? 'http://localhost:8899'
      : rpcNodes[node][cluster === 'mainnet-beta' ? 'mainnetBeta' : cluster];
  const connection = useMemo(() => new Connection(endpoint, 'recent'), [endpoint]);
  const wallet = useWallet();

  async function createClient() {
    const adapter = new SolanaConnectionAdapter(wallet, connection);
    const webClient = adapter.userAddress && await JetWebClient.connect(adapter.userAddress, adapter, true)
    setProtocolClient(webClient);
  }

  useEffect(() => {
    if (!wallet.publicKey) {
      return;
    }

    createClient();
  }, [cluster, node, wallet.publicKey]);
}
