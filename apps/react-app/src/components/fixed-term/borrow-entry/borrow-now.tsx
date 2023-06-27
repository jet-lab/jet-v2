import { Button, InputNumber, Switch } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import {
  bigIntToBn,
  bnToBigInt,
  borrowNow,
  FixedTermProductModel,
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
import { useRecoilValue } from 'recoil';
import { useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom } from '@state/fixed-term/fixed-term-market-sync';
import { RateDisplay } from '../shared/rate-display';
import { useJetStore } from '@jet-lab/store';
import { EditOutlined, LoadingOutlined } from '@ant-design/icons';
import { AutoRollChecks } from '../shared/autoroll-checks';
import { AutoRollModal } from '../shared/autoroll-modal';

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
  unfilledQty: number;
  hasEnoughCollateral: boolean;
  fees: number;
}

export const BorrowNow = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const { selectedPoolKey, airspaceLookupTables, marginAccountLookupTables, selectedMarginAccount } = useJetStore(
    state => ({
      selectedPoolKey: state.selectedPoolKey,
      airspaceLookupTables: state.airspaceLookupTables,
      marginAccountLookupTables: state.marginAccountLookupTables,
      selectedMarginAccount: state.selectedMarginAccount
    })
  );
  const lookupTables = useMemo(() => {
    if (!selectedMarginAccount) {
      return airspaceLookupTables;
    } else {
      return marginAccountLookupTables[selectedMarginAccount]?.length
        ? airspaceLookupTables.concat(marginAccountLookupTables[selectedMarginAccount])
        : airspaceLookupTables;
    }
  }, [selectedMarginAccount, airspaceLookupTables, marginAccountLookupTables]);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );
  const wallet = useWallet();
  const [amount, setAmount] = useState<BN | undefined>();
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const [forecast, setForecast] = useState<Forecast>();
  const [showAutorollModal, setShowAutorollModal] = useState(false);
  const [autorollEnabled, setAutorollEnabled] = useState(false);
  const [orderTooSmall, setOrderTooSmall] = useState(false);

  const { cluster, explorer } = useJetStore(state => state.settings);

  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (amount) {
      handleForecast(amount);
    } else {
      setForecast(undefined);
    }
  }, [amount, marginAccount?.address, marketAndConfig]);

  const enoughLiquidity = forecast && forecast.unfilledQty <= 0;

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    amount?.lte(new BN(0)) ||
    !forecast?.effectiveRate ||
    forecast.selfMatch ||
    !forecast.fulfilled ||
    !enoughLiquidity ||
    !forecast?.hasEnoughCollateral;

  const handleForecast = (amount: BN) => {
    if (bnToBigInt(amount) === BigInt(0)) {
      setForecast(undefined);
      return;
    }
    const orderbookModel = marketAndConfig.market.orderbookModel as OrderbookModel;
    try {
      const sim = orderbookModel.simulateTaker(
        'borrow',
        bnToBigInt(amount),
        undefined,
        marginAccount?.address.toBytes()
      );

      let correspondingPool = pools?.tokenPools[marketAndConfig.token.symbol];
      if (correspondingPool == undefined) {
        console.log('ERROR `correspondingPool` must be defined.');
        return;
      }

      const productModel = marginAccount
        ? FixedTermProductModel.fromMarginAccountPool(marginAccount, correspondingPool)
        : undefined;
      const setupCheckEstimate = productModel?.takerAccountForecast('borrow', sim, 'setup');
      const valuationEstimate = productModel?.takerAccountForecast('borrow', sim);

      const repayAmount = new TokenAmount(bigIntToBn(sim.filledBaseQty), token.decimals);
      const unfilledQty = new TokenAmount(bigIntToBn(sim.unfilledQuoteQty), token.decimals);
      const totalInterest = new TokenAmount(bigIntToBn(sim.filledBaseQty - sim.filledQuoteQty), token.decimals);
      const fees = new TokenAmount(bigIntToBn(sim.filledFeeQty), token.decimals);

      setForecast({
        repayAmount: repayAmount.tokens,
        interest: totalInterest.tokens,
        effectiveRate: sim.filledVwar,
        selfMatch: sim.selfMatch,
        fulfilled: sim.filledQuoteQty >= sim.totalQuoteQty - BigInt(1) * sim.matches, // allow 1 lamport rounding per match
        riskIndicator: valuationEstimate?.riskIndicator,
        unfilledQty: unfilledQty.tokens,
        hasEnoughCollateral: setupCheckEstimate && setupCheckEstimate.riskIndicator < 1 ? true : false,
        fees: fees.tokens
      });
    } catch (e) {
      console.log(e);
    }
  };

  const createBorrowOrder = async () => {
    if (!amount) return;
    setPending(true);
    let signature: string;
    try {
      if (disabled || !wallet.publicKey) return;
      signature = await borrowNow({
        market: marketAndConfig,
        marginAccount,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        amount,
        markets: markets.map(m => m.market),
        autorollEnabled,
        lookupTables
      });
      setTimeout(() => {
        notify(
          'Borrow Successful',
          `Your borrow order for ${amount
            .div(new BN(10 ** decimals))
            .toNumber()
            .toFixed(token.precision)} ${token.name} was filled successfully`,
          'success',
          getExplorerUrl(signature, cluster, explorer)
        );
        setPending(false);
      }, 3000); // TODO: Ugly and unneded, update when websocket is fully integrated
    } catch (e: any) {
      notify(
        'Borrow Order Failed',
        `Your borrow order for ${amount
          .div(new BN(10 ** decimals))
          .toNumber()
          .toFixed(token.precision)} ${token.name} failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false);
      console.error(e);
    } finally {
      setAmount(undefined);
      setForecast(undefined);
    }
  };

  useEffect(() => {
    setAutorollEnabled(false);
  }, [marketAndConfig]);

  useEffect(() => {
    if (!amount || !marketAndConfig) {
      setOrderTooSmall(false);
      return;
    }
    setOrderTooSmall(amount.toNumber() < marketAndConfig.config.minBaseOrderSize);
  }, [marketAndConfig, amount]);

  return (
    <div className="fixed-term order-entry-body">
      <p>
        You are borrowing as a taker. Your order will be filled at the prevailing market rates. Any unfilled quantity
        will not be posted to the order book.
      </p>
      <div className="borrow-now fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            value={amount ? new TokenAmount(amount, decimals).tokens : ''}
            onChange={e => {
              if (!e) {
                setAmount(undefined);
              } else {
                setAmount(new BN(e * 10 ** decimals));
              }
            }}
            placeholder={'10,000'}
            min={0}
            formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
      </div>

      <AutoRollChecks market={marketAndConfig.market} marginAccount={marginAccount}>
        {({ hasConfig, refresh, borrowRate, lendRate }) => (
          <div className="auto-roll-controls">
            <AutoRollModal
              onClose={() => {
                setShowAutorollModal(false);
              }}
              open={showAutorollModal}
              marketAndConfig={marketAndConfig}
              marginAccount={marginAccount}
              refresh={refresh}
              borrowRate={borrowRate}
              lendRate={lendRate}
            />
            <Switch
              checked={autorollEnabled}
              onClick={() => {
                if (hasConfig) {
                  setAutorollEnabled(!autorollEnabled);
                } else {
                  setShowAutorollModal(true);
                }
              }}
            />
            Auto-roll
            <EditOutlined onClick={() => setShowAutorollModal(true)} />
          </div>
        )}
      </AutoRollChecks>

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
          {forecast && enoughLiquidity ? (
            <span>
              {forecast.repayAmount.toFixed(token.precision)} {token.symbol}
            </span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          {forecast && enoughLiquidity ? (
            <span>{`~${forecast.interest.toFixed(token.precision)} ${token.symbol}`}</span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <RateDisplay rate={forecast?.effectiveRate} />
        </div>
        <div className="stat-line">
          <span>Fees</span>
          {forecast?.fees ? <span>{`~${forecast?.fees.toFixed(token.precision)} ${token.symbol}`}</span> : null}
        </div>
        <div className="stat-line">
          <span>Risk Indicator</span>
          {forecast?.riskIndicator && (
            <span>
              {marginAccount?.riskIndicator.toFixed(3)} → {forecast.riskIndicator?.toFixed(3)}
            </span>
          )}
        </div>
      </div>
      <Button className="submit-button" disabled={disabled || pending || orderTooSmall} onClick={createBorrowOrder}>
        {pending ? (
          <>
            <LoadingOutlined />
            Sending transaction
          </>
        ) : (
          `Borrow ${marketToString(marketAndConfig.config)}`
        )}
      </Button>
      {forecast?.selfMatch && (
        <div className="fixed-term-warning">The request would match with your own offers in this market.</div>
      )}
      {!forecast?.hasEnoughCollateral && amount && !amount.isZero() && (
        <div className="fixed-term-warning">Not enough collateral to submit this request</div>
      )}
      {forecast && !enoughLiquidity && (
        <div className="fixed-term-warning">Not enough liquidity on this market, try a smaller amount.</div>
      )}
      {forecast && forecast.effectiveRate === 0 && (
        <div className="fixed-term-warning">Zero rate loans are not supported. Try increasing the borrow amount.</div>
      )}
      {orderTooSmall && (
        <div className="fixed-term-warning">The minimum order size for this market is <strong>{marketAndConfig.config.minBaseOrderSize / Math.pow(10, token.decimals)} {marketAndConfig.config.symbol}</strong>.</div>
      )}
    </div>
  );
};
