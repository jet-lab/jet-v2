import { useRecoilState, useRecoilValue } from 'recoil';
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useEffect, useState } from 'react';
import {
  TokenAmount,
} from '@jet-lab/margin';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useProvider } from '@utils/jet/provider';
import { Pools } from '@state/pools/pools';
import BN from 'bn.js';
import { useOpenPositions } from '@jet-lab/store';
import { getOwedTokens, redeemDeposits, settleNow, submitRepay } from './market-selector-actions';

const { Option } = Select;

interface FixedTermMarketSelectorProps {
  type: 'asks' | 'bids';
}

export const FixedTermMarketSelector = ({ type }: FixedTermMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedTermMarketAtom);
  const pools = useRecoilValue(Pools);

  const [repayAmount, setRepayAmount] = useState('0');

  const [owedTokens, setOwedTokens] = useState<TokenAmount>(
    new TokenAmount(new BN(0), markets[selectedMarket]?.token.decimals || 6)
  );

  useEffect(() => {
    if (marginAccount?.address && markets[selectedMarket].token) {
      getOwedTokens(markets[selectedMarket], marginAccount, provider, setOwedTokens);
    }
  }, [marginAccount?.address]);

  const { data } = useOpenPositions(markets[selectedMarket]?.market, marginAccount);

  const token = markets[selectedMarket]?.token;

  if (!marginAccount || !pools || !markets[selectedMarket] || !data || !token) return null;

  const depositsToClaim = data.deposits.filter(deposit => new Date(deposit.maturation_timestamp).getTime() <= Date.now())
  const amountToClaim = depositsToClaim.reduce((sum, item) => {
    const value = new BN(item.balance)
    return sum.add(value)
  }, new BN(0))
  const hasToClaim = depositsToClaim.length > 0;
  const hasToSettle = owedTokens?.tokens > 0;
  const hasToRepay = data.total_borrowed > 0;

  return (
    <div className="fixed-term-selector-view view-element">
      <div className="fixed-term-selector-view-container">
        <Select
          value={selectedMarket + 1}
          showSearch={true}
          suffixIcon={<AngleDown className="jet-icon" />}
          onChange={value => setSelectedMarket(value - 1)}>
          {markets.map((market, index) => (
            <Option key={market.name} value={index + 1}>
              {marketToString(market.config)}
            </Option>
          ))}
        </Select>
        <div className="selector-actions">
          {hasToClaim ?
            <div className="assets-to-settle"><>Need to claim {new TokenAmount(amountToClaim, token.decimals).uiTokens} {token.symbol}
              <Button
                size='small'
                onClick={() => {
                  redeemDeposits(markets[selectedMarket], marginAccount, provider, depositsToClaim, cluster, blockExplorer, pools.tokenPools, markets.map(m => m.market))
                }}
              >
                Claim
              </Button>
            </>
            </div>
            : hasToSettle ? (
              <div className="assets-to-settle">
                There are {owedTokens?.uiTokens} {markets[selectedMarket]?.token.symbol} currently pending settment on
                this market.
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
                You owe {new TokenAmount(new BN(data.total_borrowed), token.decimals).tokens} {token.symbol} on this
                market.
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
                      markets[selectedMarket],
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
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
