import { Button, InputNumber, Switch } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import {
  bigIntToBn,
  bnToBigInt,
  FixedTermProductModel,
  MarketAndConfig,
  offerLoan,
  OrderbookModel,
  rate_to_price,
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
import { useCallback, useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom } from '@state/fixed-term/fixed-term-market-sync';
import { formatWithCommas } from '@utils/format';
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
  postedRepayAmount?: number;
  postedInterest?: number;
  postedRate?: number;
  matchedAmount?: number;
  matchedInterest?: number;
  matchedRate?: number;
  selfMatch: boolean;
  riskIndicator?: number;
  hasEnoughCollateral: boolean;
}

export const OfferLoan = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const { selectedPoolKey, airspaceLookupTables, marginAccountLookupTables, selectedMarginAccount } = useJetStore(
    state => {
      return {
        selectedPoolKey: state.selectedPoolKey,
        airspaceLookupTables: state.airspaceLookupTables,
        marginAccountLookupTables: state.marginAccountLookupTables,
        selectedMarginAccount: state.selectedMarginAccount
      };
    }
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
  const [basisPoints, setBasisPoints] = useState<number>();
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const [forecast, setForecast] = useState<Forecast>();
  const [showAutorollModal, setShowAutorollModal] = useState(false);
  const [autorollEnabled, setAutorollEnabled] = useState(false);
  const [orderTooSmall, setOrderTooSmall] = useState(false);

  const preprocessInput = useCallback((e: number) => Math.round(e * 100) / 100, []);

  const { cluster, explorer } = useJetStore(state => state.settings);

  const [pending, setPending] = useState(false);

  const tokenBalance = marginAccount?.poolPositions[token.symbol].depositBalance;
  const hasEnoughTokens = tokenBalance?.gte(new TokenAmount(amount || new BN(0), token.decimals));

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    amount?.lte(new BN(0)) ||
    forecast?.selfMatch ||
    !hasEnoughTokens ||
    !forecast?.hasEnoughCollateral;

  const createLendOrder = async () => {
    if (!amount || !basisPoints || !marginAccount) return;
    setPending(true);
    let signature: string;
    const rateBPS = new BN(Math.round(basisPoints * 100));

    try {
      if (disabled || !wallet.publicKey) return;
      signature = await offerLoan({
        market: marketAndConfig,
        marginAccount,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        amount: amount,
        basisPoints: rateBPS,
        marketConfig: marketAndConfig.config,
        markets: markets.map(m => m.market),
        autorollEnabled,
        lookupTables
      });
      setTimeout(() => {
        notify(
          'Lend Offer Created',
          `Your lend offer for ${amount
            .div(new BN(10 ** decimals))
            .toNumber()
            .toFixed(token.precision)} ${token.name} at ${(rateBPS.toNumber() / 100).toFixed(
              2
            )}% was created successfully`,
          'success',
          getExplorerUrl(signature, cluster, explorer)
        );
        setPending(false);
      }, 3000); // TODO: Ugly and unneded, update when websocket is fully integrated
    } catch (e: any) {
      notify(
        'Lend Offer Failed',
        `Your lend offer for ${amount
          .div(new BN(10 ** decimals))
          .toNumber()
          .toFixed(token.precision)} ${token.name} at ${(rateBPS.toNumber() / 100).toFixed(2)}% failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false);
      console.error(e);
    } finally {
      setAmount(undefined);
      setBasisPoints(undefined);
      setForecast(undefined);
    }
  };

  // Simulation demo logic
  function orderbookModelLogic(amount: bigint, limitPrice: bigint) {
    const model = marketAndConfig.market.orderbookModel as OrderbookModel;
    const sim = model.simulateMaker('lend', amount, limitPrice, marginAccount?.address.toBytes());
    let correspondingPool = pools?.tokenPools[marketAndConfig.token.symbol];
    if (correspondingPool == undefined) {
      console.log('ERROR `correspondingPool` must be defined.');
      return;
    }

    const productModel = marginAccount
      ? FixedTermProductModel.fromMarginAccountPool(marginAccount, correspondingPool)
      : undefined;
    const setupCheckEstimate = productModel?.makerAccountForecast('lend', sim, 'setup');
    const valuationEstimate = productModel?.makerAccountForecast('lend', sim);

    const matchRepayAmount = new TokenAmount(bigIntToBn(sim.filledBaseQty), token.decimals);
    const matchBorrowAmount = new TokenAmount(bigIntToBn(sim.filledQuoteQty), token.decimals);
    const matchRate = sim.filledVwar;
    const postedRepayAmount = new TokenAmount(bigIntToBn(sim.postedBaseQty), token.decimals);
    const postedBorrowAmount = new TokenAmount(bigIntToBn(sim.postedQuoteQty), token.decimals);
    const postedRate = sim.postedVwar;

    setForecast({
      matchedAmount: matchRepayAmount.tokens,
      matchedInterest: matchRepayAmount.sub(matchBorrowAmount).tokens,
      matchedRate: matchRate,
      postedRepayAmount: postedRepayAmount.tokens,
      postedInterest: postedRepayAmount.sub(postedBorrowAmount).tokens,
      postedRate,
      selfMatch: sim.selfMatch,
      riskIndicator: valuationEstimate?.riskIndicator,
      hasEnoughCollateral: setupCheckEstimate && setupCheckEstimate.riskIndicator < 1 ? true : false
    });
  }

  useEffect(() => {
    if (!amount || !basisPoints || amount.eqn(0) || basisPoints === 0) {
      setForecast(undefined);
      return;
    }
    orderbookModelLogic(
      bnToBigInt(amount),
      rate_to_price(bnToBigInt(new BN(Math.round(basisPoints * 100))), BigInt(marketAndConfig.config.borrowTenor))
    );
  }, [amount, basisPoints, marginAccount?.address, marketAndConfig]);
  // End simulation demo logic

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
        You are lending as a maker. Your loan offers wil be filled at the input interest rate or better. Any part of
        your offer that is not filled immediately will be posted to the orderbook.
      </p>
      <div className="offer-loan fixed-order-entry-fields">
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
            formatter={formatWithCommas}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
        <label>
          Interest Rate
          <InputNumber
            className="input-rate"
            value={basisPoints && basisPoints > 0 ? basisPoints : undefined}
            onChange={e => setBasisPoints(preprocessInput(e || 0))}
            placeholder={'6.50'}
            type="number"
            step={0.01}
            min={0}
            controls={false}
            addonAfter="%"
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
            {`${formatDuration(
              intervalToDuration({
                start: new Date(0),
                end: new Date(marketAndConfig.config.borrowTenor * 1000)
              })
            )} from fill`}
          </span>
        </div>
        <div className="stat-line">
          <span>Posted Repayment Amount</span>
          {forecast?.postedRepayAmount ? (
            <span>
              {forecast?.postedRepayAmount.toFixed(token.precision)}
              {token.symbol}
            </span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Posted Interest</span>
          {forecast?.postedInterest ? (
            <span>
              {forecast?.postedInterest.toFixed(token.precision)} {token.symbol}
            </span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Posted Rate</span>
          <RateDisplay rate={forecast?.postedRate} />
        </div>
        <div className="stat-line">
          <span>Matched Repayment Amount</span>
          {forecast?.matchedAmount ? (
            <span>{`${forecast.matchedAmount.toFixed(token.precision)} ${token.symbol}`}</span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Matched Interest</span>
          {forecast?.matchedInterest ? (
            <span>
              {forecast.matchedInterest.toFixed(token.precision)} {token.symbol}
            </span>
          ) : null}
        </div>
        <div className="stat-line">
          <span>Matched Effective Rate</span>
          <RateDisplay rate={forecast?.matchedRate} />
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
      <Button className="submit-button" disabled={disabled || pending || orderTooSmall} onClick={() => createLendOrder()}>
        {pending ? (
          <>
            <LoadingOutlined />
            Sending transaction
          </>
        ) : (
          `Offer ${marketToString(marketAndConfig.config)} loan`
        )}
      </Button>
      {forecast?.selfMatch && (
        <div className="fixed-term-warning">The offer would match with your own requests in this market.</div>
      )}
      {!hasEnoughTokens && (
        <div className="fixed-term-warning">Not enough deposited {token.symbol} to submit this offer</div>
      )}
      {!forecast?.hasEnoughCollateral && amount && basisPoints && !amount.isZero() && basisPoints !== 0 && (
        <div className="fixed-term-warning">Not enough collateral to submit this request</div>
      )}
      {orderTooSmall && (
        <div className="fixed-term-warning">The minimum order size for this market is <strong>{marketAndConfig.config.minBaseOrderSize / Math.pow(10, token.decimals)} {marketAndConfig.config.symbol}</strong>.</div>
      )}
    </div>
  );
};
