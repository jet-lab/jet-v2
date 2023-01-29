import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import {
  bigIntToBn,
  bnToBigInt,
  MarketAndconfig,
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
import { CurrentPool, Pools } from '@state/pools/pools';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useRecoilRefresher_UNSTABLE, useRecoilValue } from 'recoil';
import { useEffect, useState } from 'react';
import { MarginConfig, MarginTokenConfig } from '@jet-lab/margin';
import { AllFixedTermMarketsAtom, AllFixedTermMarketsOrderBooksAtom } from '@state/fixed-term/fixed-term-market-sync';
import { formatWithCommas } from '@utils/format';
import debounce from 'lodash.debounce';
import { RateDisplay } from '../shared/rate-display';

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndconfig;
  marginConfig: MarginConfig;
}

interface Forecast {
  totalRepayAmount?: string;
  totalInterest?: string;
  totalEffectiveRate?: number;
  matchedAmount?: string;
  matchedInterest?: string;
  matchedRate?: number;
  selfMatch: boolean;
}

export const OfferLoan = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
  const marginAccount = useRecoilValue(CurrentAccount);
  const { provider } = useProvider();
  const cluster = useRecoilValue(Cluster);
  const pools = useRecoilValue(Pools);
  const currentPool = useRecoilValue(CurrentPool);
  const wallet = useWallet();
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [amount, setAmount] = useState(new BN(0));
  const [basisPoints, setBasisPoints] = useState(new BN(0));
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const refreshOrderBooks = useRecoilRefresher_UNSTABLE(AllFixedTermMarketsOrderBooksAtom);
  const [forecast, setForecast] = useState<Forecast>();

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    basisPoints.lte(new BN(0)) ||
    amount.lte(new BN(0)) ||
    forecast?.selfMatch;

  const createLendOrder = async (amountParam?: BN, basisPointsParam?: BN) => {
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
      notify(
        'Lend Offer Created',
        `Your lend offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
          basisPoints.toNumber() / 100
        }% was created successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
      refreshOrderBooks();
    } catch (e: any) {
      notify(
        'Lend Offer Failed',
        `Your lend offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
          basisPoints.toNumber() / 100
        }% failed`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
      throw e;
    }
  };

  // Simulation demo logic
  function orderbookModelLogic(amount: bigint, limitPrice: bigint) {
    const model = marketAndConfig.market.orderbookModel as OrderbookModel;
    const sim = model.simulateMaker('lend', amount, limitPrice, marginAccount?.address.toBytes());

    if (sim.self_match) {
      // TODO Integrate with forecast panel
      console.log('WARNING Order would be rejected for self-matching');
    }

    const matchRepayAmount = new TokenAmount(bigIntToBn(sim.filled_base_qty), token.decimals);
    const matchBorrowAmount = new TokenAmount(bigIntToBn(sim.filled_quote_qty), token.decimals);
    const matchRate = sim.filled_vwar;
    const totalRepayAmount = new TokenAmount(bigIntToBn(sim.full_base_qty), token.decimals);
    const totalBorrowAmount = new TokenAmount(bigIntToBn(sim.full_quote_qty), token.decimals);
    const totalRate = sim.full_vwar;

    setForecast({
      matchedAmount: matchRepayAmount.uiTokens,
      matchedInterest: matchRepayAmount.sub(matchBorrowAmount).uiTokens,
      matchedRate: matchRate,
      totalRepayAmount: totalRepayAmount.uiTokens,
      totalInterest: totalRepayAmount.sub(totalBorrowAmount).uiTokens,
      totalEffectiveRate: totalRate,
      selfMatch: sim.self_match
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
          <span>Total Repayment Amount</span>
          {forecast?.totalRepayAmount && (
            <span>
              {forecast?.totalRepayAmount}
              {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          {forecast?.totalInterest && (
            <span>
              {forecast?.totalInterest} {token.symbol}
            </span>
          )}
        </div>
        <div className="stat-line">
          <span>Total Effective Rate</span>
          <RateDisplay rate={forecast?.totalEffectiveRate} />
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
        <div className="stat-line">Risk Level</div>
        <div className="stat-line">
          <span>Auto Roll</span>
          <span>Off</span>
        </div>
      </div>
      <Button className="submit-button" disabled={disabled} onClick={() => createLendOrder()}>
        Offer {marketToString(marketAndConfig.config)} loan
      </Button>
    </div>
  );
};
