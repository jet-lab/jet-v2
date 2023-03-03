import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import {
  bigIntToBn,
  bnToBigInt,
  FixedTermProductModel,
  lendNow,
  MarketAndConfig,
  OrderbookModel,
  TokenAmount
} from '@jet-lab/margin';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import BN from 'bn.js';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProvider } from '@utils/jet/provider';
import { Pools } from '@state/pools/pools';
import { useRecoilRefresher_UNSTABLE, useRecoilValue } from 'recoil';
import { useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom, AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';
import debounce from 'lodash.debounce';
import { RateDisplay } from '../shared/rate-display';
import { useJetStore } from '@jet-lab/store';
import { LoadingOutlined } from '@ant-design/icons';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndConfig;
  marginConfig: MarginConfig;
}

interface Forecast {
  repayAmount: number;
  interest: number;
  effectiveRate: number;
  selfMatch: boolean;
  fulfilled: boolean;
  riskIndicator?: number;
}

export const LendNow = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const wallet = useWallet();
  const [amount, setAmount] = useState(new BN(0));
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [forecast, setForecast] = useState<Forecast>();

  const { cluster, explorer } = useJetStore(state => state.settings);

  const [pending, setPending] = useState(false);

  const tokenBalance = marginAccount?.poolPositions[token.symbol].depositBalance;
  const hasEnoughTokens = tokenBalance?.gte(new TokenAmount(amount, token.decimals));

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    amount.lte(new BN(0)) ||
    !forecast?.effectiveRate ||
    forecast.selfMatch ||
    !forecast.fulfilled ||
    !hasEnoughTokens;

  const handleForecast = (amount: BN) => {
    if (bnToBigInt(amount) === BigInt(0)) {
      setForecast(undefined);
      return;
    }
    const orderbookModel = marketAndConfig.market.orderbookModel as OrderbookModel;
    try {
      const sim = orderbookModel.simulateTaker('lend', bnToBigInt(amount), undefined);

      let correspondingPool = pools?.tokenPools[marketAndConfig.token.symbol];
      if (correspondingPool == undefined) {
        console.log('ERROR `correspondingPool` must be defined.');
        return;
      }

      const productModel = marginAccount
        ? FixedTermProductModel.fromMarginAccountPool(marginAccount, correspondingPool)
        : undefined;
      const setupCheckEstimate = productModel?.takerAccountForecast('lend', sim, 'setup');
      if (setupCheckEstimate && setupCheckEstimate.riskIndicator >= 1.0) {
        // FIXME Disable form submission
        console.log('WARNING Trade violates setup check and should not be allowed');
      }

      const valuationEstimate = productModel?.takerAccountForecast('lend', sim);

      const repayAmount = new TokenAmount(bigIntToBn(sim.filled_base_qty), token.decimals);
      const lendAmount = new TokenAmount(bigIntToBn(sim.filled_quote_qty), token.decimals);
      setForecast({
        repayAmount: repayAmount.tokens,
        interest: repayAmount.sub(lendAmount).tokens,
        effectiveRate: sim.filled_vwar,
        selfMatch: sim.self_match,
        fulfilled: sim.filled_quote_qty >= sim.order_quote_qty - BigInt(1) * sim.matches,
        riskIndicator: valuationEstimate?.riskIndicator
      });
    } catch (e) {
      console.log(e);
    }
  };

  const marketLendOrder = async () => {
    setPending(true);
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
      setTimeout(() => {
        refreshOrderBooks();
        notify(
          'Lend Successful',
          `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} was filled successfully`,
          'success',
          getExplorerUrl(signature, cluster, explorer)
        );
        setPending(false);
      }, 2000); // TODO: Ugly and unneded. update when websocket is fully integrated
    } catch (e: any) {
      notify(
        'Lend Order Failed',
        `Your lend order for ${amount.div(new BN(10 ** decimals))} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false);
      throw e;
    }
  };

  useEffect(() => {
    handleForecast(amount);
  }, [amount, marginAccount?.address, marketAndConfig]);


  // for test debugging
  useEffect(() => {
    if(!amount.isZero() && disabled) {
      console.log(
        'hasMarginAccount', !!marginAccount,
        'hasPublicKey', !!wallet.publicKey,
        'hasCurrentPool', !!currentPool,
        'hasEnoughTokens', hasEnoughTokens,
        'poolsAvailable', pools?.tokenPools.length || 0,
        'amountSelected', amount.toNumber(),
        'wouldSelfMatch', forecast?.selfMatch,
      )
    }
  }, [disabled, amount])

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
              {forecast.repayAmount.toFixed(token.precision)} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          {forecast?.interest && (
            <span>
              {forecast.interest.toFixed(token.precision)} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <RateDisplay rate={forecast?.effectiveRate} />
        </div>
        <div className="stat-line">
          <span>Risk Indicator</span>
          {forecast && (
            <span>
              {marginAccount?.riskIndicator.toFixed(3)} → {forecast.riskIndicator?.toFixed(3)}
            </span>
          )}
        </div>
      </div>
      <Button className="submit-button" disabled={disabled || pending} onClick={marketLendOrder}>
        {pending ? (
          <>
            <LoadingOutlined />
            Sending transaction
          </>
        ) : (
          `Lend ${marketToString(marketAndConfig.config)}`
        )}
      </Button>
      {forecast?.selfMatch && (
        <div className="fixed-term-warning">The request would match with your own offers in this market.</div>
      )}
      {!hasEnoughTokens && (
        <div className="fixed-term-warning">Not enough deposited {token.symbol} to submit this request</div>
      )}
    </div>
  );
};
