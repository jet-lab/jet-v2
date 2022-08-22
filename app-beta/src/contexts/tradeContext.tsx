import { Pool, PoolAction } from '@jet-lab/margin';
import { createContext, useContext, useState } from 'react';

// Current trade info UI context
interface TradeInfo {
  currentPool: Pool | undefined;
  setCurrentPool: (pool: Pool) => void;
  currentAction: PoolAction;
  setCurrentAction: (action: PoolAction) => void;
  currentAmount: number | null;
  setCurrentAmount: (amount: number | null) => void;
  sendingTrade: boolean;
  setSendingTrade: (sending: boolean) => void;
}
const TradeContext = createContext<TradeInfo>({
  currentPool: undefined,
  setCurrentPool: () => null,
  currentAction: 'deposit',
  setCurrentAction: () => null,
  currentAmount: null,
  setCurrentAmount: () => null,
  sendingTrade: false,
  setSendingTrade: () => null
});

// Trade info context provider
export function TradeContextProvider(props: { children: JSX.Element }): JSX.Element {
  const [currentPool, setCurrentPool] = useState<Pool | undefined>();
  const [currentAction, setCurrentAction] = useState<PoolAction>('deposit');
  const [currentAmount, setCurrentAmount] = useState<number | null>(null);
  const [sendingTrade, setSendingTrade] = useState<boolean>(false);

  return (
    <TradeContext.Provider
      value={{
        currentPool,
        setCurrentPool,
        currentAction,
        setCurrentAction,
        currentAmount,
        setCurrentAmount,
        sendingTrade,
        setSendingTrade
      }}>
      {props.children}
    </TradeContext.Provider>
  );
}

// Trade info hook
export const useTradeContext = () => {
  const context = useContext(TradeContext);
  return context;
};
