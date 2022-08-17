import { AnchorProvider } from "@project-serum/anchor"
import {
  ConfirmOptions,
  Connection,
  SendTransactionError,
  Transaction,
  TransactionInstruction,
  TransactionSignature
} from "@solana/web3.js"
import bs58 from "bs58"

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
          })
          .filter(tx => !!tx) as Transaction[]
      } else {
        const ixs = tx as any as TransactionInstruction[]
        if (ixs.length > 0) {
          return [new Transaction({ feePayer: provider.wallet.publicKey, blockhash, lastValidBlockHeight }).add(...ixs)]
        }
      }
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

  for (let i = 0; i < signedUnflattened.length; i++) {
    const transactions = signedUnflattened[i]
    const txnArray = await Promise.all(
      transactions.map(async tx => {
        const rawTx = tx.serialize()

        try {
          return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts)
        } catch (err) {
          // thrown if the underlying 'confirmTransaction' encounters a failed tx
          // the 'confirmTransaction' error does not return logs so we make another rpc call to get them
          if (err instanceof ConfirmError) {
            // choose the shortest available commitment for 'getTransaction'
            // (the json RPC does not support any shorter than "confirmed" for 'getTransaction')
            // because that will see the tx sent with `sendAndConfirmRawTransaction` no matter which
            // commitment `sendAndConfirmRawTransaction` used
            const failedTx = await provider.connection.getTransaction(bs58.encode(tx.signature!), {
              commitment: "confirmed"
            })
            if (!failedTx) {
              throw err
            } else {
              const logs = failedTx.meta?.logMessages
              throw !logs ? err : new SendTransactionError(err.message, logs)
            }
          } else {
            throw err
          }
        }
      })
    )
    // Return the txid of the final transaction in the array
    // TODO: We should return an array instead of only the final txn
    lastTxn = txnArray[txnArray.length - 1] ?? ""
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
    throw new ConfirmError(`Raw transaction ${signature} failed (${JSON.stringify(status)})`)
  }

  return signature
}

class ConfirmError extends Error {
  constructor(message?: string) {
    super(message)
  }
}
