import { MarginAccount, MarketAndConfig, TokenAmount } from '@jet-lab/margin';
import { useJetStore, useOpenPositions } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import { useProvider } from '@utils/jet/provider';
import { Button } from 'antd';
import BN from 'bn.js';
import { useEffect, useMemo, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { getOwedTokens, redeemDeposits, settleNow, submitRepay } from './market-selector-actions';

interface IMarketSelectorButtonProps {
  marginAccount?: MarginAccount;
  markets: MarketAndConfig[];
  selectedMarket?: MarketAndConfig;
}
export const MarketSelectorButtons = ({ marginAccount, markets, selectedMarket }: IMarketSelectorButtonProps) => {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const apiEndpoint = useMemo(
    () =>
      cluster === 'mainnet-beta'
        ? process.env.REACT_APP_DATA_API
        : cluster === 'devnet'
        ? process.env.REACT_APP_DEV_DATA_API
        : cluster === 'localnet'
        ? process.env.REACT_APP_LOCAL_DATA_API
        : '',
    [cluster]
  );
  const { data } = useOpenPositions(String(apiEndpoint), selectedMarket?.market, marginAccount);
  const { provider } = useProvider();
  const pools = useRecoilValue(Pools);

  const [repayAmount, setRepayAmount] = useState('0');

  const [owedTokens, setOwedTokens] = useState<TokenAmount>(
    new TokenAmount(new BN(0), selectedMarket?.token.decimals || 6)
  );

  const token = selectedMarket?.token;
  useEffect(() => {
    if (marginAccount?.address && selectedMarket?.token) {
      getOwedTokens(selectedMarket, marginAccount, provider, setOwedTokens);
    }
  }, [marginAccount?.address]);

  const [totalBorrowed, setTotalBorrowed] = useState(
    token && data && new TokenAmount(new BN(data.total_borrowed), token.decimals)
  );

  useEffect(() => {
    setTotalBorrowed(new TokenAmount(data ? new BN(data.total_borrowed) : new BN(0), token?.decimals || 0));
  }, [data]);

  if (!marginAccount || !data || !pools || !token) return null;

  const depositsToClaim = data.deposits.filter(
    deposit => new Date(deposit.maturation_timestamp).getTime() <= Date.now()
  );
  const amountToClaim = depositsToClaim.reduce((sum, item) => {
    const value = new BN(item.balance);
    return sum.add(value);
  }, new BN(0));
  const hasToClaim = depositsToClaim.length > 0;
  const hasToSettle = owedTokens?.tokens > 0;
  const hasToRepay = data.total_borrowed > 0 && totalBorrowed;

  const handleRepay = async () => {
    const bnAmount = new BN(parseFloat(repayAmount) * 10 ** token.decimals);
    const tokenAmount = new TokenAmount(bnAmount, token.decimals);
    if (!totalBorrowed) return;
    try {
      await submitRepay(
        marginAccount,
        provider,
        bnAmount,
        data.loans,
        pools.tokenPools,
        markets.map(m => m.market),
        selectedMarket,
        cluster,
        explorer
      );
      if (totalBorrowed.sub(tokenAmount).lte(new TokenAmount(new BN(0), token.decimals))) {
        setTotalBorrowed(undefined);
      } else {
        setTotalBorrowed(totalBorrowed.sub(tokenAmount));
      }
    } catch (e) {
      console.log(e);
    }
  };

  return (
    <div className="selector-actions">
      {hasToClaim ? (
        <div className="assets-to-settle">
          <>
            Need to claim {new TokenAmount(amountToClaim, token.decimals).uiTokens} {token.symbol} on this market.
            <Button
              size="small"
              onClick={() => {
                redeemDeposits(
                  selectedMarket,
                  marginAccount,
                  provider,
                  depositsToClaim,
                  cluster,
                  explorer,
                  pools.tokenPools,
                  markets.map(m => m.market)
                );
              }}>
              Claim
            </Button>
          </>
        </div>
      ) : hasToSettle ? (
        <div className="assets-to-settle">
          There are {owedTokens?.uiTokens} {selectedMarket?.token.symbol} currently pending settment on this account.
          <Button
            onClick={() =>
              settleNow(
                marginAccount,
                markets,
                selectedMarket,
                provider,
                setOwedTokens,
                cluster,
                explorer,
                pools,
                owedTokens
              )
            }>
            Settle Now
          </Button>
        </div>
      ) : hasToRepay ? (
        <div className="assets-to-settle">
          <span>
            You owe{' '}
            <span className="click-to-repay" onClick={() => setRepayAmount(totalBorrowed.tokens.toString())}>
              {totalBorrowed.tokens} {token.symbol}
            </span>{' '}
            on this market.
          </span>
          <span className="input-and-button">
            <input
              value={repayAmount}
              onChange={e => {
                const parsed = parseFloat(e.target.value);
                if (isNaN(parsed)) {
                  setRepayAmount('0');
                } else {
                  const total = new TokenAmount(new BN(data.total_borrowed), token.decimals);
                  const amount = parsed <= total.tokens ? e.target.value : total.uiTokens.replace(',', '');
                  setRepayAmount(amount);
                }
              }}
            />
            <Button onClick={handleRepay}>Repay Now</Button>
          </span>
        </div>
      ) : (
        <div>There are no outstanding actions on this market.</div>
      )}
    </div>
  );
};
