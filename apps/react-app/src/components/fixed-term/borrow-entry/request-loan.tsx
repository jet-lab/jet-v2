import { Button, InputNumber, Switch, Tooltip } from 'antd';
import { formatDuration, intervalToDuration } from 'date-fns';
import { MarketAndconfig, OrderbookModel, bnToBigInt, rate_to_price, requestLoan, ui_price } from '@jet-lab/margin';
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

interface RequestLoanProps {
  decimals: number;
  token: MarginTokenConfig;
  marketAndConfig: MarketAndconfig;
  marginConfig: MarginConfig;
}

export const RequestLoan = ({ token, decimals, marketAndConfig }: RequestLoanProps) => {
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

  const disabled =
    !marginAccount ||
    !wallet.publicKey ||
    !currentPool ||
    !pools ||
    basisPoints.lte(new BN(0)) ||
    amount.lte(new BN(0));

  const createBorrowOrder = async (amountParam?: BN, basisPointsParam?: BN) => {
    let signature: string;
    try {
      if (disabled || !wallet.publicKey) return;
      signature = await requestLoan({
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
        'Borrow Offer Created',
        `Your borrow offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
          basisPoints.toNumber() / 100
        }% was created successfully`,
        'success',
        getExplorerUrl(signature, cluster, blockExplorer)
      );
      refreshOrderBooks();
    } catch (e: any) {
      notify(
        'Borrow Offer Failed',
        `Your borrow offer for ${amount.div(new BN(10 ** decimals))} ${token.name} at ${
          basisPoints.toNumber() / 100
        }% failed`,
        'error',
        getExplorerUrl(e.signature, cluster, blockExplorer)
      );
      throw e;
    }
  };
  
  function orderbookModelLogic(amount: bigint, limitPrice: bigint) {
    console.log(amount, limitPrice, ui_price(limitPrice));
    const model = marketAndConfig.market.orderbookModel as OrderbookModel;
    if (model.wouldMatch("borrow", limitPrice)) {
      const fillSim = model.simulateFills("borrow", amount, limitPrice);
      if (fillSim.unfilled_quote_qty > 0) {
        console.log("Order would partially fill immediately");
        console.log(fillSim);
        console.log("Unfilled quantity would be posted to the top of the book");
      } else {
        console.log("Order would completely fill immediately");
        console.log(fillSim);
        console.log("Nothing would be posted");
      }
    } else {
      const queueSim = model.simulateQueuing("borrow", limitPrice);
      if (queueSim.depth > 0) {
        console.log("Order would post without fills into the the book");
        console.log(queueSim);
      } else {
        console.log("Order would post without fills to the top of the book");
      }
    }
  }

  useEffect(() => {
    if (amount.eqn(0) || basisPoints.eqn(0)) return;
    console.log(amount.toNumber(), basisPoints.toNumber());
    orderbookModelLogic(
      bnToBigInt(amount),
      rate_to_price(
        bnToBigInt(basisPoints),
        BigInt(marketAndConfig.config.borrowTenor)
      )
    );
  }, [amount, basisPoints])

  return (
    <div className="fixed-term order-entry-body">
      <div className="request-loan fixed-order-entry-fields">
        <label>
          Loan amount
          <InputNumber
            className="input-amount"
            onChange={
              debounce(e => {
                // ORDERBOOKMODEL WASM CHECK
                //  -> want to call a function here that uses input amount AND input rate
                // END CHECK TODO deleteme

                setAmount(new BN(e * 10 ** decimals));
              }, 300)}
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
            onChange={debounce(e => {
              // ORDERBOOKMODEL WASM CHECK
              //  -> want to call a function here that uses input amount AND input rate
              // END CHECK TODO deleteme

              setBasisPoints(new BN(e * 100));
            }, 300)}
            placeholder={'1.5'}
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
          <span>Repayment Amount</span>
          <span>
            {formatWithCommas(
              ((amount.toNumber() / 10 ** decimals) * (1 + basisPoints.toNumber() / 10000)).toFixed(token.precision)
            )}{' '}
            {token.symbol}
          </span>
        </div>
        <div className="stat-line">
          <span>Total Interest</span>
          <span>
            {(amount.toNumber() / 10 ** decimals) * (basisPoints.toNumber() / 10000)} {token.symbol}
          </span>
        </div>
        <div className="stat-line">
          <span>Interest Rate</span>
          <span>{basisPoints.toNumber() / 100}%</span>
        </div>
        <div className="stat-line">Risk Level</div>
        <div className="stat-line">
          <span>Auto Roll</span>
          <span>Off</span>
        </div>
      </div>
      <Button className="submit-button" disabled={disabled} onClick={() => createBorrowOrder()}>
        Request {marketToString(marketAndConfig.config)} loan
      </Button>
    </div>
  );
};
