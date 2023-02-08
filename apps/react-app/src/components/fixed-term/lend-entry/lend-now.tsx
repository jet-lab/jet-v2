import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import { bigIntToBn, bnToBigInt, lendNow, MarketAndconfig, OrderbookModel, TokenAmount } from '@jet-lab/margin';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProvider } from '@utils/jet/provider';
import { CurrentPool, Pools } from '@state/pools/pools';
import { useRecoilRefresher_UNSTABLE, useRecoilValue } from 'recoil';
import { useEffect, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom, AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';
import debounce from 'lodash.debounce';
import { RateDisplay } from '../shared/rate-display';
import { useJetStore } from '@jet-lab/store';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndconfig;
  marginConfig: MarginConfig;
}

interface Forecast {
  repayAmount: string;
  interest: string;
  effectiveRate: number;
  selfMatch: boolean;
  fulfilled: boolean;
}

export const LendNow = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const [amount, setAmount] = useState(new BN(0));
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [forecast, setForecast] = useState<Forecast>();

  const { cluster, explorer } = useJetStore(state => state.settings);

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    amount.lte(new BN(0)) ||
    !forecast?.effectiveRate ||
    forecast.selfMatch ||
    !forecast.fulfilled;

  const handleForecast = (amount: BN) => {
    if (bnToBigInt(amount) === BigInt(0)) {
      setForecast(undefined);
      return;
    }
    const orderbookModel = marketAndConfig.market.orderbookModel as OrderbookModel;
    try {
      const sim = orderbookModel.simulateTaker('lend', bnToBigInt(amount), undefined);
      const repayAmount = new TokenAmount(bigIntToBn(sim.filled_base_qty), token.decimals);
      const lendAmount = new TokenAmount(bigIntToBn(sim.filled_quote_qty), token.decimals);
      setForecast({
        repayAmount: repayAmount.uiTokens,
        interest: repayAmount.sub(lendAmount).uiTokens,
        effectiveRate: sim.filled_vwar,
        selfMatch: sim.self_match,
        fulfilled: sim.filled_quote_qty >= sim.order_quote_qty - BigInt(1) * sim.matches
      });
    } catch (e) {
      console.log(e);
    }
  };

  const marketLendOrder = async () => {
    let signature: string;
    try {
      if (disabled || !wallet.publicKey) return;
      signature = await lendNow({
        market: marketAndConfig,
        marginAccount,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        amount,
        markets: markets.map(m => m.market)
      });
      notify(
        'Lend Successful',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} was filled successfully`,
        'success',
        getExplorerUrl(signature, cluster, explorer)
      );
      refreshOrderBooks();
    } catch (e: any) {
      notify(
        'Lend Order Failed',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      throw e;
    }
  };

  useEffect(() => {
    handleForecast(amount);
  }, [amount, marginAccount?.address, marketAndConfig]);

  return (
    <div className="fixed-term order-entry-body">
      <div className="lend-now fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={debounce(e => {
              setAmount(new BN(e * 10 ** decimals));
            }, 300)}
            placeholder={'10,000'}
            min={0}
            formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
      </div>

      <div className="auto-roll-controls">
        <Tooltip title="Coming soon...">
          <Switch disabled={true} />
        </Tooltip>
        Auto-roll Off
      </div>

      <div className="stats">
        <div className="stat-line">
          <span>Repayment Date</span>
          <span>
            {`in ${formatDuration(
              intervalToDuration({
                start: new Date(0),
                end: new Date(marketAndConfig.config.borrowTenor * 1000)
              })
            )}`}
          </span>
        </div>
        <div className="stat-line">
          <span>Repayment Amount</span>
          {forecast?.repayAmount && (
            <span>
              {forecast.repayAmount} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          {forecast?.interest && (
            <span>
              {forecast.interest} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <RateDisplay rate={forecast?.effectiveRate} />
        </div>
        <div className="stat-line">Risk Level</div>
        <div className="stat-line">
          <span>Auto Roll</span>
          <span>Off</span>
        </div>
      </div>
      <Button className="submit-button" disabled={disabled} onClick={marketLendOrder}>
        Lend {marketToString(marketAndConfig.config)}
      </Button>
    </div>
  );
};
