import { AccountInfo, Commitment, Connection, Context, PublicKey } from '@solana/web3.js';

/**
 * Fetch an account for the specified public key and subscribe a callback
 * to be invoked whenever the specified account changes.
 *
 * @param connection Connection to use
 * @param publicKey Public key of the account to monitor
 * @param callback Function to invoke whenever the account is changed
 * @param commitment Specify the commitment level account changes must reach before notification
 * @return subscription id
 */
export async function getAccountInfoAndSubscribe(
  connection: Connection,
  publicKey: PublicKey,
  callback: (acc: AccountInfo<Buffer> | null, context: Context) => void,
  commitment?: Commitment | undefined
): Promise<number> {
  let latestSlot = -1;
  const subscriptionId = connection.onAccountChange(
    publicKey,
    (account: AccountInfo<Buffer>, context: Context) => {
      if (context.slot >= latestSlot) {
        latestSlot = context.slot;
        callback(account, context);
      }
    },
    commitment
  );

  const response = await connection.getAccountInfoAndContext(publicKey, commitment);
  if (response.context.slot >= latestSlot) {
    latestSlot = response.context.slot;
    if (response.value != null) {
      callback(response.value, response.context);
    } else {
      callback(null, response.context);
    }
  }

  return subscriptionId;
}
