// import { Pubkey } from '@jet-lab/jet-client-web';
// import { WalletContextState } from '@solana/wallet-adapter-react';
// import { AccountInfo, Connection, PublicKey, VersionedTransaction } from '@solana/web3.js';

// export class SolanaConnectionAdapter {
//   userAddress?: Pubkey;

//   constructor(public wallet: WalletContextState, public connection: Connection) {
//     if (wallet.publicKey) {
//       this.userAddress = new Pubkey(wallet.publicKey.toBase58());
//     }
//   }

//   async getGenesisHash(): Promise<string> {
//     return await this.connection.getGenesisHash();
//   }

//   async getAccounts(addresses: PublicKey[]): Promise<(AccountInfo<Buffer> | null)[]> {
//     try {
//       return await this.connection.getMultipleAccountsInfo(addresses);
//     } catch (e) {
//       console.log(e);
//       throw e;
//     }
//   }

//   async getLatestBlockhash(): Promise<any> {
//     return await this.connection.getLatestBlockhash();
//   }

//   async send(transactionDatas: Uint8Array[]): Promise<string[]> {
//     try {
//       const {
//         value: { blockhash, lastValidBlockHeight }
//       } = await this.connection.getLatestBlockhashAndContext();

//       const transactions = transactionDatas.map(txData => {
//         let tx = VersionedTransaction.deserialize(txData);
//         tx.message.recentBlockhash = blockhash;
//         return tx;
//       });
//       console.log(transactionDatas);
//       console.log(transactions);
//       const signed: VersionedTransaction[] = [];

//       if (!this.wallet.signTransaction) {
//         console.log('wallet missing signer function');
//         return [];
//       }

//       for (const tx of transactions) {
//         signed.push(await this.wallet.signTransaction(tx));
//       }

//       console.log(signed);
//       const signatures: string[] = [];

//       for (const tx of signed) {
//         const signature = await this.connection.sendTransaction(tx);
//         await this.connection.confirmTransaction({ blockhash, lastValidBlockHeight, signature }, 'processed');

//         signatures.push(signature);
//       }

//       return signatures;
//     } catch (e) {
//       console.log(e);
//       throw e;
//     }
//   }
// }
export {};
