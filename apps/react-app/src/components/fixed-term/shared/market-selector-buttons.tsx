import { MarginAccount, MarketAndconfig, TokenAmount } from '@jet-lab/margin';
import { useOpenPositions } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useProvider } from '@utils/jet/provider';
import { Button } from 'antd';
import BN from 'bn.js';
import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { getOwedTokens, redeemDeposits, settleNow, submitRepay } from './market-selector-actions';

interface IMarketSelectorButtonProps {
  marginAccount?: MarginAccount;
  markets: MarketAndconfig[];
  selectedMarket: MarketAndconfig;
}
export const MarketSelectorButtons = ({ marginAccount, markets, selectedMarket }: IMarketSelectorButtonProps) => {
  const { data } = useOpenPositions(selectedMarket?.market, marginAccount);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();
  const pools = useRecoilValue(Pools);

  const [repayAmount, setRepayAmount] = useState('0');

  const [owedTokens, setOwedTokens] = useState<TokenAmount>(
    new TokenAmount(new BN(0), selectedMarket?.token.decimals || 6)
  );

  const token = selectedMarket?.token;
  useEffect(() => {
    if (marginAccount?.address && selectedMarket.token) {
      getOwedTokens(selectedMarket, marginAccount, provider, setOwedTokens);
    }
  }, [marginAccount?.address]);

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
  const hasToRepay = data.total_borrowed > 0;

  return (
    <div className="selector-actions">
      {hasToClaim ? (
        <div className="assets-to-settle">
          <>
            Need to claim {new TokenAmount(amountToClaim, token.decimals).uiTokens} {token.symbol}
            <Button
              size="small"
              onClick={() => {
                redeemDeposits(
                  selectedMarket,
                  marginAccount,
                  provider,
                  depositsToClaim,
                  cluster,
                  blockExplorer,
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
          There are {owedTokens?.uiTokens} {selectedMarket?.token.symbol} currently pending settment on this market.
          <Button
            onClick={() =>
              settleNow(
                marginAccount,
                markets,
                selectedMarket,
                provider,
                setOwedTokens,
                cluster,
                blockExplorer,
                pools,
                owedTokens
              )
            }>
            Settle Now
          </Button>
        </div>
      ) : hasToRepay ? (
        <div className="assets-to-settle">
          You owe {new TokenAmount(new BN(data.total_borrowed), token.decimals).tokens} {token.symbol} on this market.
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
          <Button
            onClick={() =>
              submitRepay(
                marginAccount,
                provider,
                new BN(parseFloat(repayAmount) * 10 ** token.decimals),
                data.loans,
                pools.tokenPools,
                markets.map(m => m.market),
                selectedMarket,
                cluster,
                blockExplorer
              )
            }>
            Repay Now
          </Button>
        </div>
      ) : (
        <div>There are no outstanding actions on this market.</div>
      )}
    </div>
  );
};
