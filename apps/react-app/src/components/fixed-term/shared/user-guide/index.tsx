import { Carousel, DismissModal } from '@jet-lab/ui';

interface IUserGuidePage {
  nextPage: () => void;
  previousPage: () => void;
  dismiss?: () => void;
  currentPage: number;
  totalPages: number;
  headline: string;
  content: JSX.Element;
  picture?: string;
  isEndPage: boolean;
  isFirstPage: boolean;
}

const UserGuidePage = ({
  nextPage,
  previousPage,
  dismiss,
  currentPage,
  totalPages,
  headline,
  content,
  picture,
  isFirstPage,
  isEndPage
}: IUserGuidePage) => (
  <div className="flex flex-col">
    <div className="flex pt-12">
      <div className="flex flex-col flex-1 px-4">
        <p className="font-light my-2 text-4xl leading-normal">
          {currentPage}/{totalPages}
        </p>
        <p className="font-normal my-2 text-2xl">{headline}</p>
        {content}
      </div>
      {picture && (
        <div className="flex flex-1">
          <img src={picture} />
        </div>
      )}
    </div>
    <div className="flex font-normal items-center text-2xl my-2 pt-20">
      <a
        className="flex-1 pr-6 font-normal text-sm"
        href="https://uploads-ssl.webflow.com/620e4761998cce492a7c9c8d/62ebf0ff41fac7359bfb2964_litepaper-v0.0.1.pdf"
        target="_blank"
        rel="noopener noreferrer">
        Read the Litepaper
      </a>
      {!isFirstPage && (
        <button className="flex-1" onClick={previousPage}>
          &#8592; Previous
        </button>
      )}
      {!isEndPage && (
        <button className="flex-1" onClick={nextPage}>
          Next &#8594;
        </button>
      )}
      {dismiss && (
        <button className="flex-1" onClick={dismiss}>
          Get Started &#8594;
        </button>
      )}
    </div>
  </div>
);

const Page1Content = () => (
  <p className="font-normal my-2 text-base">
    The term of a fixed rate loan is determined in advance by the market you choose to transact in. For example, in a
    7-day SOL market loans are repaid after seven days, and in a 1-day USDC market loans are repaid after 1 day.
    <br />
    The interest rate of a fixed rate loan is also determined in advance by the market participants.
  </p>
);

const Page2Content = () => (
  <p className="font-normal text-base">
    Interest rates in a fixed rate market are determined by lenders and borrowers who are transacting as makers by{' '}
    <b className="font-bold">offering loans</b> and <b className="font-bold">requesting loans</b>. Each loan offer has a
    fixed rate chosen by the lender and each loan request has has a fixed rate chosen by the borrowers.
    <br />
    Borrowers seeking immediate liquidity may choose to <b className="font-bold">borrow now</b> by accepting loans on
    offer which will determine in advance the interest rate for a loan of whatever size they choose. Lenders may choose
    to <b className="font-bold">lend now</b> by satisfying loan requests on the book. The interest rate will be
    determined in advance by the rate associated with the requests that are filled.
  </p>
);

const Page3Content = () => (
  <p className="font-normal text-base">
    However you choose to transact you will be shown a plot of the available liquidity in the market and an order input
    panel. If you are a taker lending or borrowing immediately you only have to input the amount you'd like to borrow.
    If you are a maker offering or requesting a loan you have to input the amount and the interest rate.
    <br />
    Once you have completed the order form a summary of the expected outcome will be presented for your review prior to
    submitting the order.
  </p>
);

const Page4Content = () => (
  <p className="font-normal text-base">
    When you have borrowed tokens in a fixed rate market you end up with a <b className="font-bold">term loan</b>. It is
    important to keep track of your term loans and repay them by their maturity date. Otherwise some of your collateral
    may be sold by the protocol to repay them for you.
    <br />
    It is possible to configure term loans and deposits to be automatically rolled for another term at maturity by using
    the <b className="font-bold">auto roll</b> feature.
  </p>
);

export const UserGuide = () => (
  // <div className="relative top-44 left-96 rounded">
  <DismissModal storageKey="fixed-term-guide" title="Fixed Rate Debt Markets" className="w-3/4">
    {({ dismiss }) => (
      <Carousel
        pagesToRender={4}
        pages={({ previousPage, nextPage, pageNumber, isEndPage, isFirstPage }) => [
          <UserGuidePage
            key={1}
            isEndPage={isEndPage}
            isFirstPage={isFirstPage}
            headline="Select a market"
            nextPage={nextPage}
            previousPage={previousPage}
            currentPage={pageNumber}
            totalPages={4}
            content={<Page1Content />}
            picture="img/guide/dropdown.png"
          />,
          <UserGuidePage
            key={2}
            isEndPage={isEndPage}
            isFirstPage={isFirstPage}
            headline="Transact as a maker or taker"
            nextPage={nextPage}
            previousPage={previousPage}
            currentPage={pageNumber}
            totalPages={4}
            content={<Page2Content />}
            picture="img/guide/entry_chart.png"
          />,
          <UserGuidePage
            key={3}
            isEndPage={isEndPage}
            isFirstPage={isFirstPage}
            headline="Submit an order"
            nextPage={nextPage}
            previousPage={previousPage}
            currentPage={pageNumber}
            totalPages={4}
            content={<Page3Content />}
            picture="img/guide/lend_borrow.png"
          />,
          <UserGuidePage
            dismiss={dismiss}
            key={4}
            isEndPage={isEndPage}
            isFirstPage={isFirstPage}
            headline="Manage your assets and liabilities"
            nextPage={nextPage}
            previousPage={previousPage}
            currentPage={pageNumber}
            totalPages={4}
            content={<Page4Content />}
          />
        ]}
      />
    )}
  </DismissModal>
);
