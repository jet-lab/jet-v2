import { useMargin } from '../contexts/marginContext';
import { PoolTokenChange, TokenFormat } from '@jet-lab/margin';

export enum TxResponseType {
  Success = 'SUCCESS',
  Failed = 'FAILED',
  Cancelled = 'CANCELLED'
}

export interface TransactionResponse {
  txid: string | undefined;
  response: TxResponseType;
}

export const useMarginActions = () => {
  const { pools, marginAccount, refresh } = useMargin();

  // Deposit
  const deposit = async (abbrev: string, change: PoolTokenChange): Promise<TransactionResponse> => {
    if (!marginAccount || !pools) {
      console.log('Accounts not loaded', marginAccount, pools);
      throw new Error();
    }

    const pool = pools[abbrev];

    try {
      const txid = await pool.deposit({ marginAccount, change });
      await refresh();
      return { txid: txid, response: TxResponseType.Success };
    } catch (err: any) {
      console.log(err);
      await refresh();
      if (err.toString().search('User rejected the request') > -1) {
        return { txid: undefined, response: TxResponseType.Cancelled };
      } else {
        return { txid: undefined, response: TxResponseType.Failed };
      }
    }
  };

  // Withdraw
  const withdraw = async (abbrev: string, change: PoolTokenChange): Promise<TransactionResponse> => {
    if (!marginAccount || !pools) {
      throw new Error();
    }

    const pool = pools[abbrev];

    try {
      const txid = await pool.withdraw({
        marginAccount,
        pools: Object.values(pools),
        change
      });
      await refresh();
      return { txid: txid, response: TxResponseType.Success };
    } catch (err: any) {
      console.log(err);
      await refresh();
      if (err.toString().search('User rejected the request') > -1) {
        return { txid: undefined, response: TxResponseType.Cancelled };
      } else {
        return { txid: undefined, response: TxResponseType.Failed };
      }
    }
  };

  // Borrow
  const borrow = async (abbrev: string, change: PoolTokenChange): Promise<TransactionResponse> => {
    if (!marginAccount || !pools) {
      throw new Error();
    }

    const pool = pools[abbrev];
    try {
      const txid = await pool.marginBorrow({
        marginAccount,
        pools: Object.values(pools),
        change,
        destination: TokenFormat.unwrappedSol
      });
      await refresh();
      return { txid: txid, response: TxResponseType.Success };
    } catch (err: any) {
      console.log(err);
      await refresh();
      if (err.toString().search('User rejected the request') > -1) {
        return { txid: undefined, response: TxResponseType.Cancelled };
      } else {
        return { txid: undefined, response: TxResponseType.Failed };
      }
    }
  };

  // Repay
  const repay = async (abbrev: string, change: PoolTokenChange): Promise<TransactionResponse> => {
    if (!marginAccount || !pools) {
      throw new Error();
    }

    const pool = pools[abbrev];

    try {
      const txid = await pool.marginRepay({
        marginAccount,
        pools: Object.values(pools),
        change,
        source: TokenFormat.unwrappedSol
      });
      await refresh();
      return { txid: txid, response: TxResponseType.Success };
    } catch (err: any) {
      console.log(err);
      await refresh();
      if (err.toString().search('User rejected the request') > -1) {
        return { txid: undefined, response: TxResponseType.Cancelled };
      } else {
        return { txid: undefined, response: TxResponseType.Failed };
      }
    }
  };

  return {
    deposit,
    withdraw,
    borrow,
    repay
  };
};
