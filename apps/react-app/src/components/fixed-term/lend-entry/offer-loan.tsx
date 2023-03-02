import { Button, InputNumber, Switch, Tooltip } from 'antd';
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
import { useRecoilRefresher_UNSTABLE, useRecoilValue } from 'recoil';
import { useEffect, useMemo, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom, AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';
import { formatWithCommas } from '@utils/format';
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
  postedRepayAmount?: string;
  postedInterest?: string;
  postedRate?: number;
  matchedAmount?: string;
  matchedInterest?: string;
  matchedRate?: number;
  selfMatch: boolean;
  riskIndicator?: number;
}

export const OfferLoan = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
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
  const [basisPoints, setBasisPoints] = useState(new BN(0));
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [forecast, setForecast] = useState<Forecast>();

  const { cluster, explorer } = useJetStore(state => state.settings);

  const [pending, setPending] = useState(false)

  
  const tokenBalance = marginAccount?.poolPositions[token.symbol].depositBalance
  const hasEnoughTokens = tokenBalance?.gte(new TokenAmount(amount, token.decimals))

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    basisPoints.lte(new BN(0)) ||
    amount.lte(new BN(0)) ||
    forecast?.selfMatch ||
    !hasEnoughTokens;

  const createLendOrder = async (amountParam?: BN, basisPointsParam?: BN) => {
    setPending(true)
    let signature: string;
    try {
      if (disabled || !wallet.publicKey) return;
      signature = await offerLoan({
        market: marketAndConfig,
        marginAccount,
        provider,
        walletAddress: wallet.publicKey,
        pools: pools.tokenPools,
        amount: amountParam || amount,
        basisPoints: basisPointsParam || basisPoints,
        marketConfig: marketAndConfig.config,
        markets: markets.map(m => m.market)
      });
      setTimeout(() => {
        refreshOrderBooks();
        notify(
          'Lend Offer Created',
          `Your lend offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
            basisPoints.toNumber() / 100
          }% was created successfully`,
          'success',
          getExplorerUrl(signature, cluster, explorer)
        );
        setPending(false)
      }, 2000); // TODO: Ugly and unneded, update when websocket is fully integrated
    } catch (e: any) {
      notify(
        'Lend Offer Failed',
        `Your lend offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
          basisPoints.toNumber() / 100
        }% failed`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false)
      throw e;
    }
  };

  // Simulation demo logic
  function orderbookModelLogic(amount: bigint, limitPrice: bigint) {
    const model = marketAndConfig.market.orderbookModel as OrderbookModel;
    const sim = model.simulateMaker('lend', amount, limitPrice, marginAccount?.address.toBytes());

    if (sim.self_match) {
      // FIXME Integrate with forecast panel
      console.log('ERROR Order would be rejected for self-matching');
    }

    let correspondingPool = pools?.tokenPools[marketAndConfig.token.symbol];
    if (correspondingPool == undefined) {
      console.log('ERROR `correspondingPool` must be defined.');
      return;
    }

    const productModel = marginAccount
      ? FixedTermProductModel.fromMarginAccountPool(marginAccount, correspondingPool)
      : undefined;
    const setupCheckEstimate = productModel?.makerAccountForecast('lend', sim, 'setup');
    if (setupCheckEstimate && setupCheckEstimate.riskIndicator >= 1.0) {
      // FIXME Disable form submission
      console.log('WARNING Trade violates setup check and should not be allowed');
    }

    const valuationEstimate = productModel?.makerAccountForecast('lend', sim);

    const matchRepayAmount = new TokenAmount(bigIntToBn(sim.filled_base_qty), token.decimals);
    const matchBorrowAmount = new TokenAmount(bigIntToBn(sim.filled_quote_qty), token.decimals);
    const matchRate = sim.filled_vwar;
    const postedRepayAmount = new TokenAmount(bigIntToBn(sim.posted_base_qty), token.decimals);
    const postedBorrowAmount = new TokenAmount(bigIntToBn(sim.posted_quote_qty), token.decimals);
    const postedRate = sim.posted_vwar;

    setForecast({
      matchedAmount: matchRepayAmount.uiTokens,
      matchedInterest: matchRepayAmount.sub(matchBorrowAmount).uiTokens,
      matchedRate: matchRate,
      postedRepayAmount: postedRepayAmount.uiTokens,
      postedInterest: postedRepayAmount.sub(postedBorrowAmount).uiTokens,
      postedRate,
      selfMatch: sim.self_match,
      riskIndicator: valuationEstimate?.riskIndicator
    });
  }

  useEffect(() => {
    if (amount.eqn(0) || basisPoints.eqn(0)) return;
    orderbookModelLogic(
      bnToBigInt(amount),
      rate_to_price(bnToBigInt(basisPoints), BigInt(marketAndConfig.config.borrowTenor))
    );
  }, [amount, basisPoints, marginAccount?.address, marketAndConfig]);
  // End simulation demo logic

  return (
    <div className="fixed-term order-entry-body">
      <div className="offer-loan fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={debounce(e => setAmount(new BN(e * 10 ** decimals)), 300)}
            placeholder={'10,000'}
            min={0}
            formatter={formatWithCommas}
            controls={false}
            addonAfter={marketAndConfig.config.symbol}
          />
        </label>
        <label>
          Min Interest Rate
          <InputNumber
            className="input-rate"
            onChange={debounce(e => {
              setBasisPoints(bigIntToBn(BigInt(Math.floor(e * 100)))); // Ensure we submit basis points
            }, 300)}
            placeholder={'6.50'}
            type="number"
            step={0.01}
            min={0}
            controls={false}
            addonAfter="%"
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
          {forecast?.postedRepayAmount && (
            <span>
              {forecast?.postedRepayAmount}
              {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Posted Interest</span>
          {forecast?.postedInterest && (
            <span>
              {forecast?.postedInterest} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Posted Rate</span>
          <RateDisplay rate={forecast?.postedRate} />
        </div>
        <div className="stat-line">
          <span>Matched Repayment Amount</span>
          {forecast?.matchedAmount && (
            <span>
              {forecast.matchedAmount}
              {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Matched Interest</span>
          {forecast?.matchedInterest && (
            <span>
              {forecast.matchedInterest} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Matched Effective Rate</span>
          <RateDisplay rate={forecast?.matchedRate} />
        </div>
        <div className="stat-line">
          <span>Risk Indicator</span>
          {forecast && <span>{forecast.riskIndicator}</span>}
        </div>
      </div>
      <Button className="submit-button" disabled={disabled || pending} onClick={() => createLendOrder()}>
      {pending ? <><LoadingOutlined />Sending transaction</> :  `Offer ${marketToString(marketAndConfig.config)} loan`}
      </Button>
      {forecast?.selfMatch && <div className='fixed-term-warning'>The offer would match with your own requests in this market.</div>}
      {!hasEnoughTokens && <div className='fixed-term-warning'>Not enough deposited {token.symbol} to submit this offer</div>}
    </div>
  );
};
