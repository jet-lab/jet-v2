import { AnchorProvider } from "@project-serum/anchor"
import {
  AddressLookupTableAccount,
  ConfirmOptions,
  Connection,
  PublicKey,
  SendTransactionError,
  Signer,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  TransactionSignature,
  VersionedTransaction
} from "@solana/web3.js"
import axios from "axios"
import bs58 from "bs58"

/**
 * Sends the given transaction, paid for and signed by the provider's wallet.
 *
 * @param provider     The Anchor provider
 * @param instructions The transaction to send.
 * @param signers      The signers of the transaction.
 * @param opts         Transaction confirmation options.
 */
export async function sendAndConfirm(
  provider: AnchorProvider,
  instructions: TransactionInstruction[],
  signers?: Signer[],
  opts?: ConfirmOptions
): Promise<TransactionSignature> {
  if (opts === undefined) {
    opts = provider.opts
  }

  const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash(opts.preflightCommitment)
  const transaction = new Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(
    ...instructions
  )

  transaction.feePayer = provider.wallet.publicKey
  transaction.recentBlockhash = (await provider.connection.getRecentBlockhash(opts.preflightCommitment)).blockhash

  if (signers?.length) {
    transaction.partialSign(...signers)
  }
  const signedTransaction = await provider.wallet.signTransaction(transaction)

  const rawTx = signedTransaction.serialize()

  try {
    return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts)
  } catch (err: any) {
    // thrown if the underlying 'confirmTransaction' encounters a failed tx
    // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
    // choose the shortest available commitment for 'getTransaction'
    // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
    // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
    // commitment `sendAndConfirmRawTransaction` used
    await provider.connection.confirmTransaction(bs58.encode(signedTransaction.signature!), "confirmed")
    const failedTx = await provider.connection.getTransaction(bs58.encode(signedTransaction.signature!), {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0
    })
    const logs = failedTx?.meta?.logMessages
    const message = `${err.message}\n${JSON.stringify(logs, undefined, 2)}`
    throw !logs ? err : new SendTransactionError(message)
  }
}

/**
 * Sends the given transaction, paid for and signed by the provider's wallet.
 *
 * @param provider     The Anchor provider
 * @param instructions The transaction to send.
 * @param signers      The signers of the transaction.
 * @param opts         Transaction confirmation options.
 */
export async function sendAndConfirmV0(
  provider: AnchorProvider,
  instructions: TransactionInstruction[],
  authorities: string[],
  signers?: Signer[],
  opts?: ConfirmOptions
): Promise<TransactionSignature> {
  if (opts === undefined) {
    opts = provider.opts
  }
  let lookups: string[] = (await axios.post("http://localhost:3006/lookup/get_addresses", {
    instructions: instructions.map(i => {
      return {
        program: i.programId.toJSON(),
        accounts: i.keys.map(k => {
          return k.pubkey.toJSON()
        }),
        data: []
      }
    }),
    // TODO: hardcoded for now
    authorities: ["Ecxa8ZnbGJcDSPqBRDQQ3ZTjtBSnkvqPcMdMkx7JAZCM", ...authorities]
  })).data.addresses;


  const tablesResponse = await Promise.all(
    lookups.map(address => provider.connection.getAddressLookupTable(new PublicKey(address)).then(res => res.value))
  )
  const tables: AddressLookupTableAccount[] = []
  for (const table of tablesResponse) {
    if (table) {
      tables.push(table)
    }
  }

  const { blockhash } = await provider.connection.getLatestBlockhash(opts.preflightCommitment)
  const message = new TransactionMessage({
    payerKey: provider.wallet.publicKey,
    recentBlockhash: blockhash,
    instructions
  }).compileToV0Message(tables)

  const transaction = new VersionedTransaction(message)

  if (signers?.length) {
    transaction.sign(signers)
  }
  // This works, but ideally we shouldn't have to cast the v0tx
  const signedTransaction = await provider.wallet.signTransaction(transaction as any)
  const rawTx = signedTransaction.serialize()

  try {
    return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts)
  } catch (err: any) {
    // thrown if the underlying 'confirmTransaction' encounters a failed tx
    // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
    // choose the shortest available commitment for 'getTransaction'
    // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
    // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
    // commitment `sendAndConfirmRawTransaction` used
    // await provider.connection.confirmTransaction(bs58.encode(transaction.signatures[0]), "confirmed")
    const failedTx = await provider.connection.getTransaction(bs58.encode(transaction.signatures[0]), {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0
    })
    const logs = failedTx?.meta?.logMessages
    const message = `${err.message}\n${JSON.stringify(logs, undefined, 2)}`
    throw !logs ? err : new SendTransactionError(message)
  }
}

/**
 * Sends all transactions. If an entry in the transactions array is
 * a sub-array, then transactions within the sub array are sent in parallel
 */
export async function sendAll(
  provider: AnchorProvider,
  transactions: (TransactionInstruction[] | TransactionInstruction[][])[],
  opts?: ConfirmOptions
): Promise<string> {
  if (opts === undefined) {
    opts = provider.opts
  }
  const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash(opts.preflightCommitment)

  const txs = transactions
    .map(tx => {
      if (Array.isArray(tx[0])) {
        return tx
          .map((tx: any) => {
            const ixs = tx as any as TransactionInstruction[]
            if (ixs.length > 0) {
              return new Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(
                ...ixs
              )
            }
            return
          })
          .filter(tx => !!tx) as Transaction[]
      } else {
        const ixs = tx as any as TransactionInstruction[]
        if (ixs.length > 0) {
          return [new Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(...ixs)]
        }
      }
      return
    })
    .filter(tx => !!tx) as (Transaction | Transaction[])[]

  let start = 0
  const slices = txs.map(tx => {
    const length = Array.isArray(tx) ? tx.length : 1
    const end = start + length
    const slice = [start, end]
    start = end
    return slice
  })

  // signedTxs has been flattened. unflatten it
  const signedTxs = await provider.wallet.signAllTransactions(txs.flat(1))
  const signedUnflattened = slices.map(slice => signedTxs.slice(...slice))

  let lastTxn = ""
  try {
    for (let i = 0; i < signedUnflattened.length; i++) {
      const transactions = signedUnflattened[i]
      const txnArray: string[] = []
      for (const tx of transactions) {
        const rawTx = tx.serialize()
        const sent = await sendAndConfirmRawTransaction(provider.connection, rawTx, opts).catch(err => {
          let customErr = new ConfirmError(err.message)
          customErr.signature = bs58.encode(tx.signature!)
          throw customErr
        })
        txnArray.push(sent)
      }
      // Return the txid of the final transaction in the array
      // TODO: We should return an array instead of only the final txn
      lastTxn = txnArray[txnArray.length - 1] ?? ""
    }
  } catch (e: any) {
    throw e
  }
  return lastTxn
}

// Copy of Connection.sendAndConfirmRawTransaction that throws
// a better error if 'confirmTransaction` returns an error status
async function sendAndConfirmRawTransaction(
  connection: Connection,
  rawTransaction: Buffer,
  options?: ConfirmOptions
): Promise<TransactionSignature> {
  const sendOptions = options && {
    skipPreflight: options.skipPreflight,
    preflightCommitment: options.preflightCommitment || options.commitment
  }

  const signature = await connection.sendRawTransaction(rawTransaction, sendOptions)

  const status = (await connection.confirmTransaction(signature, options && options.commitment)).value

  if (status.err) {
    const error = new ConfirmError(`Raw transaction ${signature} failed (${JSON.stringify(status)})`)
    throw error
  }

  return signature
}

class ConfirmError extends Error {
  signature?: string
  constructor(message?: string) {
    super(message)
  }
}
