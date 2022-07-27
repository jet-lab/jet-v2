import { AnchorProvider } from "@project-serum/anchor"
import { ConfirmOptions, sendAndConfirmRawTransaction, Transaction, TransactionInstruction } from "@solana/web3.js"

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

  let lastTxn = "";

  for (let i = 0; i < signedUnflattened.length; i++) {
    const transactions = signedUnflattened[i]
    try {
      const txnArray = await Promise.all(
        transactions.map(async tx => {
          if (tx.instructions.length > 0) {
            const rawTx = tx.serialize()
            return await sendAndConfirmRawTransaction(provider.connection, rawTx, opts)
          }
        })
      )
      // Return the txid of the final transaction in the array
      lastTxn = txnArray[txnArray.length - 1] ?? ""
    } catch (err: any) {
      // preserve stacktrace
      console.log(err, JSON.stringify(err.logs))
      throw err
    }
  }
  return lastTxn;
}