import { Carousel, DismissModal, Title, SubTitle, Paragraph } from '@jet-lab/ui';
import { useNavigate } from 'react-router-dom'

interface IUserGuidePage {
  nextPage: () => void;
  previousPage: () => void;
  dismiss?: () => void;
  currentPage: number;
  totalPages: number;
  headline: string;
  content: JSX.Element;
  sidebar: JSX.Element;
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
  sidebar,
  isFirstPage,
  isEndPage
}: IUserGuidePage) => (
  <div className="flex flex-col">
    <div className="flex pt-12">
      <div className="flex flex-col flex-1 px-4">
        <Title>{currentPage}/{totalPages}</Title>
        <SubTitle classNameOverride="my-2">{headline}</SubTitle>
        {content}
      </div>
      {sidebar}
    </div>
    <div className="flex font-normal items-center text-2xl my-2 pt-8">
      <a
        className="flex-1 pr-6 font-normal text-sm"
        href="https://docs.jetprotocol.io/jet-protocol/protocol/jet-products/fixed-rate"
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
  <Paragraph>
    The term of a fixed rate loan is determined in advance by the market you choose to transact in. For example, in a
    7-day SOL market loans are repaid after seven days, and in a 1-day USDC market loans are repaid after 1 day.
    <br />
    The interest rate of a fixed rate loan is also determined in advance by the market participants.
  </Paragraph>
);

const Page1Sidebar = () => <div className='flex flex-1 items-center justify-center'>
  <img className='object-contain' srcSet="img/guide/page_1_sidebar/bg@1x.png 480w,
             img/guide/page_1_sidebar/bg@2x.png 800w,
             img/guide/page_1_sidebar/bg@3x.png 2000w"
    sizes="(max-width: 600px) 480px,
            (max-width: 1000px) 800px,
            1000px"
    src="img/guide/page_1_sidebar/bg@1x.png"
  />
</div>

const Page2Content = () => (
  <Paragraph>
    Interest rates in a fixed rate market are determined by lenders and borrowers who are transacting as makers by{' '}
    <b className="font-bold">offering loans</b> and <b className="font-bold">requesting loans</b>. Each loan offer has a
    fixed rate chosen by the lender and each loan request has has a fixed rate chosen by the borrowers.
    <br />
    Borrowers seeking immediate liquidity may choose to <b className="font-bold">borrow now</b> by accepting loans on
    offer which will determine in advance the interest rate for a loan of whatever size they choose. Lenders may choose
    to <b className="font-bold">lend now</b> by satisfying borrow requests on the book. The interest rate will be
    determined in advance by the rate associated with the requests that are filled.
  </Paragraph>
);

const Page2Sidebar = () => <div className='flex flex-1 items-center justify-center'>
  <img className='object-contain' srcSet="img/guide/page_2_sidebar/bg@1x.png 480w,
           img/guide/page_2_sidebar/bg@2x.png 800w,
           img/guide/page_2_sidebar/bg@3x.png 2000w"
    sizes="(max-width: 600px) 480px,
          (max-width: 1000px) 800px,
          1000px"
    src="img/guide/page_2_sidebar/bg@1x.png"
  />
</div>

const Page3Content = () => (
  <Paragraph>
    However you choose to transact you will be shown a plot of the available liquidity in the market and an order input
    panel. If you are a taker lending or borrowing immediately you only have to input the amount you'd like to borrow.
    If you are a maker offering or requesting a loan you have to input the amount and the interest rate.
    <br />
    Once you have completed the order form a summary of the expected outcome will be presented for your review prior to
    submitting the order.
  </Paragraph>
);

const Page3Sidebar = () => <div className='flex flex-1 items-center justify-center'>
  <img className='object-contain' srcSet="img/guide/page_3_sidebar/bg@1x.png 480w,
         img/guide/page_3_sidebar/bg@2x.png 800w,
         img/guide/page_3_sidebar/bg@3x.png 2000w"
    sizes="(max-width: 600px) 480px,
        (max-width: 1000px) 800px,
        1000px"
    src="img/guide/page_3_sidebar/bg@1x.png"
  />
</div>

const Page4Content = () => (
  <Paragraph>
    When you have borrowed tokens in a fixed rate market you end up with a <b className="font-bold">term loan</b>. It is
    important to keep track of your term loans and repay them by their maturity date. Otherwise some of your collateral
    may be sold by the protocol to repay them for you.
    <br />
    It is possible to configure term loans and deposits to be automatically rolled for another term at maturity by using
    the <b className="font-bold">auto roll</b> feature.
  </Paragraph>
);

const Page4Sidebar = () => {
  const navigate = useNavigate()
  return <div className='flex flex-col flex-1 items-center justify-center'>
    <Title classNameOverride='my-2'>Explore more on the Protocol</Title>
    <div onClick={() => navigate('/swaps')} className='my-2 relative cursor-pointer'>
      <div className='absolute top-0 left-0 right-0 bottom-0 h-full w-full flex flex-col rounded-lg p-4 bg-opacity-0 hover:bg-opacity-60 bg-slate-900 transition-colors'>
        <div className='flex items-center justify-between'>
          <SubTitle>Swaps</SubTitle>
          <span>Explore &#8594;</span>
        </div>
        <div className='h-full flex items-center'>
          <Paragraph>Trade on margin with Jet’s integrated swap routing service.</Paragraph>
        </div>
      </div>
      <img className='object-contain' srcSet="img/guide/page_4_sidebar/swaps@1x.png 480w,
           img/guide/page_4_sidebar/swaps@2x.png 800w,
           img/guide/page_4_sidebar/swaps@3x.png 2000w"
        sizes="(max-width: 600px) 480px,
          (max-width: 1000px) 800px,
          1000px"
        src="img/guide/page_4_sidebar/swaps@1x.png"
      />
    </div>
    <div onClick={() => navigate('/')} className='my-2 relative cursor-pointer'>
      <div className='absolute top-0 left-0 right-0 bottom-0 h-full w-full flex flex-col rounded-lg p-4 bg-opacity-0 hover:bg-opacity-60 bg-slate-900 transition-colors'>
        <div className='flex items-center justify-between'>
          <SubTitle>Pools</SubTitle>
          <span>Explore &#8594;</span>
        </div>
        <div className='h-full flex items-center'>
          <Paragraph>Lend and borrow with Jet’s variable rate pools.</Paragraph>
        </div>
      </div>
      <img className='object-contain' srcSet="img/guide/page_4_sidebar/pools@1x.png 480w,
           img/guide/page_4_sidebar/pools@2x.png 800w,
           img/guide/page_4_sidebar/pools@3x.png 2000w"
        sizes="(max-width: 600px) 480px,
          (max-width: 1000px) 800px,
          1000px"
        src="img/guide/page_4_sidebar/pools@1x.png"
      />
    </div>
  </div>
}



export const UserGuide = () => (
  // <div className="relative top-44 left-96 rounded">
  <DismissModal storageKey="fixed-term-guide" title="Fixed Rate Debt Markets" className="w-3/4" open={false}>
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
            sidebar={<Page1Sidebar />}
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
            sidebar={<Page2Sidebar />}
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
            sidebar={<Page3Sidebar />}
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
            sidebar={<Page4Sidebar />}
          />
        ]}
      />
    )}
  </DismissModal>
);
