import React, { useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { AccountSnapshot } from '@components/misc/AccountSnapshot/AccountSnapshot';
import { FixedPriceChartContainer } from '@components/fixed-term/shared/fixed-term-market-chart';
import { FullAccountBalance } from '@components/tables/FullAccountBalance';
import { Dictionary } from '@state/settings/localization/localization';
import { FixedLendOrderEntry } from '@components/fixed-term/lend-entry';
import { FixedLendRowOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { FixedTermMarketSelector } from '@components/fixed-term/shared/market-selector';
import { NetworkStateAtom } from '@state/network/network-state';
import { WaitingForNetworkView } from './WaitingForNetwork';
import { DebtTable } from '@components/fixed-term/shared/debt-table/debt-table';
import { Modal, Carousel } from '@jet-lab/ui';

const rowComponents: Record<string, React.FC<any>> = {
  fixedLendEntry: FixedLendOrderEntry,
  fixedLendChart: FixedPriceChartContainer
};

const rowComponentsProps: Record<string, object> = {
  fixedLendEntry: { key: 'fixedLendEntry' },
  fixedLendChart: { key: 'fixedLendChart', type: 'bids' }
};

const FixedRow = (): JSX.Element => {
  const rowOrder = useRecoilValue(FixedLendRowOrder);
  return (
    <div key="fixedRow" className="view-row fixed-row">
      {rowOrder.map(key => {
        const Comp = rowComponents[key];
        const props = rowComponentsProps[key];
        return <Comp {...props} />;
      })}
    </div>
  );
};

const viewComponents: Record<string, React.FC<any>> = {
  accountSnapshot: AccountSnapshot,
  fixedRow: FixedRow,
  debtTable: DebtTable,
  fullAccountBalance: FullAccountBalance,
  marketSelector: FixedTermMarketSelector
};

const viewComponentsProps: Record<string, object> = {
  accountSnapshot: { key: 'accountSnapshot' },
  fixedRow: { key: 'fixedRow' },
  debtTable: { key: 'debtTable' },
  fullAccountBalance: { key: 'fullAccountBalance' },
  marketSelector: { key: 'marketSelector', type: 'bids' }
};

const MainView = (): JSX.Element => {
  const viewOrder = useRecoilValue(FixedLendViewOrder);

  return (
    <div className="fixed-term-view view">

    {/* components: it's fine to hard code
      - title
      - page / total pages 
      - bullet point of the user step
      - details of the user step
      - screenshot
      - link to litepaper 
      - button to previous page and next page

      // todo - question: 
      1. how to style the modal to contain all content
      2. how to scroll to the last page? currently nextPage doesn't take to the last page
      3. why is the app unresponsive when the modal is closed?

      // todo: add pictures from figma
      */}
      <div className="relative top-44 left-96 rounded">
        <Modal title="Fixed Rate Debt Markets">
          <Carousel
            pages={({ previousPage, nextPage }) => [
              <div key="1" className="flex">
                <div className="basis-1/2">
                  {/* <h2 className="absolute w-64 h-6 top-6 left-6">Fixed Rate Debt Markets</h2> */}
                  <p className="absolute left-6 top-10 font-light text-4xl">1/4</p>
                  <p className="absolute left-6 top-20 font-normal text-2xl">Select a market</p>
                  <p className="absolute left-6 top-24 font-normal text-base">
                    The term of a fixed rate loan is determined in advance by the market you choose to transact in. For
                    example, in a 7-day SOL market loans are repaid after seven days, and in a 1-day USDC market loans
                    are repaid after 1 day.
                    <br />
                    The interest rate of a fixed rate loan is also determined in advance by the market participants.
                  </p>
                  <span className="flex left-6 top-52 font-normal text 2xl">
                    <a
                      className="flex-1 pr-6 font-normal text-sm"
                      href="https://uploads-ssl.webflow.com/620e4761998cce492a7c9c8d/62ebf0ff41fac7359bfb2964_litepaper-v0.0.1.pdf"
                      target="_blank"
                      rel="noopener noreferrer">
                      Read the Litepaper
                    </a>
                    <button className="flex-1" onClick={nextPage}>
                      Next &#8594;
                    </button>
                  </span>
                </div>
                <div className="basis-1/2">swap picture</div>
              </div>,
              <div key="2" className="flex">
                <div className="basis-1/2">
                  {/* <h2 className="absolute w-64 h-6 top-6 left-6">Fixed Rate Debt Markets</h2> */}
                  <p className="absolute left-6 top-10 font-light text-4xl">2/4</p>
                  <p className="absolute left-6 top-20 font-normal text-2xl">Transact as a maker or taker</p>
                  <p className="absolute left-6 top-24 font-normal text-base">
                    Interest rates in a fixed rate market are determined by lenders and borrowers who are transacting as
                    makers by <b>offering loans</b> and <b>requesting loans</b>. Each loan offer has a fixed rate chosen
                    by the lender and each loan request has has a fixed rate chosen by the borrowers.
                    <br />
                    Borrowers seeking immediate liquidity may choose to <b>borrow now</b> by accepting loans on offer
                    which will determine in advance the interest rate for a loan of whatever size they choose. Lenders
                    may choose to <b>lend now</b> by satisfying loan requests on the book. The interest rate will be
                    determined in advance by the rate associated with the requests that are filled.
                  </p>
                  <span className="flex left-6 top-52 font-normal text 2xl">
                    <a
                      className="flex-1 pr-6 font-normal text-sm"
                      href="https://uploads-ssl.webflow.com/620e4761998cce492a7c9c8d/62ebf0ff41fac7359bfb2964_litepaper-v0.0.1.pdf"
                      target="_blank"
                      rel="noopener noreferrer">
                      Read the Litepaper
                    </a>
                    <button className="flex-1" onClick={previousPage}>
                      &#8592; Previous
                    </button>
                    <button className="flex-1" onClick={nextPage}>
                      Next &#8594;
                    </button>
                  </span>
                </div>
                <div className="basis-1/2">swap picture</div>
              </div>,
              <div key="3" className="flex">
                <div className="basis-1/2">
                  {/* <h2 className="absolute w-64 h-6 top-6 left-6">Fixed Rate Debt Markets</h2> */}
                  <p className="absolute left-6 top-10 font-light text-4xl">3/4</p>
                  <p className="absolute left-6 top-20 font-normal text-2xl">Submit an order</p>
                  <p className="absolute left-6 top-24 font-normal text-base">
                    However you choose to transact you will be shown a plot of the available liquidity in the market and
                    an order input panel. If you are a taker lending or borrowing immediately you only have to input the
                    amount you'd like to borrow. If you are a maker offering or requesting a loan you have to input the
                    amount and the interest rate.
                    <br />
                    Once you have completed the order form a summary of the expected outcome will be presented for your
                    review prior to submitting the order.
                  </p>
                  <span className="flex left-6 top-52 font-normal text 2xl">
                    <a
                      className="flex-1 pr-6 font-normal text-sm"
                      href="https://uploads-ssl.webflow.com/620e4761998cce492a7c9c8d/62ebf0ff41fac7359bfb2964_litepaper-v0.0.1.pdf"
                      target="_blank"
                      rel="noopener noreferrer">
                      Read the Litepaper
                    </a>
                    <button className="flex-1" onClick={previousPage}>
                      &#8592; Previous
                    </button>
                    <button className="flex-1" onClick={nextPage}>
                      Next &#8594;
                    </button>
                  </span>
                </div>
                <div className="basis-1/2">swap picture</div>
              </div>,
              <div key="4" className="flex">
                <div className="basis-1/2">
                  {/* <h2 className="absolute w-64 h-6 top-6 left-6">Fixed Rate Debt Markets</h2> */}
                  <p className="absolute left-6 top-10 font-light text-4xl">4/4</p>
                  <p className="absolute left-6 top-20 font-normal text-2xl">Manage your assets and liabilities</p>
                  <p className="absolute left-6 top-24 font-normal text-base">
                    When you have borrowed tokens in a fixed rate market you end up with a <b>term loan</b>. It is
                    important to keep track of your term loans and repay them by their maturity date. Otherwise some of
                    your collateral may be sold by the protocol to repay them for you.
                    <br />
                    It is possible to configure term loans and deposits to be automatically rolled for another term at
                    maturity by using the <b>auto roll</b> feature.
                  </p>
                  <span className="flex left-6 top-52 font-normal text 2xl">
                    <a
                      className="flex-1 pr-6 font-normal text-sm"
                      href="https://uploads-ssl.webflow.com/620e4761998cce492a7c9c8d/62ebf0ff41fac7359bfb2964_litepaper-v0.0.1.pdf"
                      target="_blank"
                      rel="noopener noreferrer">
                      Read the Litepaper
                    </a>
                    <button className="flex-1" onClick={previousPage}>
                      &#8592; Previous
                    </button>
                  </span>
                </div>
                <div className="basis-1/2">swap picture</div>
              </div>
            ]}
          />
        </Modal>
      </div>

      {viewOrder.map(key => {
        const Comp = viewComponents[key];
        const props = viewComponentsProps[key];
        return <Comp {...props} />;
      })}
    </div>
  );
};

export function FixedLendView(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const networkState = useRecoilValue(NetworkStateAtom);
  useEffect(() => {
    document.title = `${dictionary.fixedView.lend.title} | Jet Protocol`;
  }, [dictionary.fixedView.lend.title]);
  if (networkState !== 'connected') return <WaitingForNetworkView networkState={networkState} />;
  return <MainView />;
}

export default FixedLendView;
