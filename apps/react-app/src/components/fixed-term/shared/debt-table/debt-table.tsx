import { useRecoilState, useRecoilValue } from 'recoil';
import { AccountsViewOrder } from '@state/views/views';
import { CurrentAccount } from '@state/user/accounts';
import { Tabs } from 'antd';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { LoadingOutlined } from '@ant-design/icons';
import { useJetStore, useOrdersForUser } from '@jet-lab/store';
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { useEffect, useMemo } from 'react';
import { notify } from '@utils/notify';
import { useProvider } from '@utils/jet/provider';
import { PostedOrdersTable } from './posted-order-table';
import { TokenAmount } from '@jet-lab/margin';
import BN from 'bn.js';
import numeral from 'numeral';
import { useOpenPositions } from '@jet-lab/store';
import { OpenBorrowsTable } from './open-borrows-table';
import { OpenDepositsTable } from './open-deposits-table';
import { Pools } from '@state/pools/pools';

interface ITabLink {
  name: string;
  amount: number;
  decimals: number;
}
const TabLink = ({ name, amount, decimals }: ITabLink) => {
  const formatted = useMemo(() => {
    const ta = new TokenAmount(new BN(amount), decimals);
    const num = numeral(ta.tokens);
    return num.format('0.0a');
  }, [amount]);

  return (
    <div className="tab-link">
      {name}
      <span className="badge">{formatted}</span>
    </div>
  );
};

export function DebtTable() {
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const account = useRecoilValue(CurrentAccount);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const selectedMarket = useRecoilValue(SelectedFixedTermMarketAtom);
  const market = markets[selectedMarket];
  const { provider } = useProvider();
  const { cluster, explorer } = useJetStore(state => state.settings);
  const pools = useRecoilValue(Pools);

  const apiEndpoint = useMemo(
    () =>
      cluster === 'mainnet-beta'
        ? process.env.DATA_API
        : cluster === 'devnet'
        ? process.env.DEV_DATA_API
        : cluster === 'localnet'
        ? process.env.LOCAL_DATA_API
        : '',
    [cluster]
  );

  const {
    data: ordersData,
    error: ordersError,
    isLoading: ordersLoading
  } = useOrdersForUser(String(apiEndpoint), market?.market, account);
  const {
    data: positionsData,
    error: positionsError,
    isLoading: positionsLoading
  } = useOpenPositions(String(apiEndpoint), market?.market, account);

  useEffect(() => {
    if (ordersError || positionsError)
      notify(
        'Error fetching data',
        'There was an unexpected error fetching your orders data, please try again soon',
        'warning'
      );
  }, [ordersError, positionsError]);

  if (!pools) {
    return null;
  }

  return (
    <div className="debt-detail account-table view-element flex-centered">
      <ConnectionFeedback />
      {ordersData && positionsData && market && (
        <Tabs
          defaultActiveKey="open-orders"
          destroyInactiveTabPane={true}
          items={[
            {
              label: (
                <TabLink
                  name="Loan Offers"
                  amount={ordersData.unfilled_lend}
                  decimals={markets[selectedMarket].token.decimals}
                />
              ),
              key: 'loan-offers',
              children:
                ordersLoading || !account ? (
                  <LoadingOutlined />
                ) : (
                  <PostedOrdersTable
                    data={ordersData?.open_orders.filter(o => o.is_lend_order) || []}
                    provider={provider}
                    market={markets[selectedMarket]}
                    marginAccount={account}
                    cluster={cluster}
                    blockExplorer={explorer}
                    pools={pools.tokenPools}
                    markets={markets.map(m => m.market)}
                  />
                )
            },
            {
              label: (
                <TabLink
                  name="Open Deposits"
                  amount={positionsData.total_lent}
                  decimals={markets[selectedMarket].token.decimals}
                />
              ),
              key: 'open-deposits',
              children:
                ordersLoading || !account ? (
                  <LoadingOutlined />
                ) : (
                  <OpenDepositsTable
                    data={positionsData.deposits}
                    market={markets[selectedMarket]}
                    provider={provider}
                    marginAccount={account}
                    cluster={cluster}
                    blockExplorer={explorer}
                    pools={pools.tokenPools}
                    markets={markets.map(m => m.market)}
                  />
                )
            },
            {
              label: (
                <TabLink
                  name="Borrow Requests"
                  amount={ordersData.unfilled_borrow}
                  decimals={markets[selectedMarket].token.decimals}
                />
              ),
              key: 'borrow-requests',
              children:
                ordersLoading || !account ? (
                  <LoadingOutlined />
                ) : (
                  <PostedOrdersTable
                    data={ordersData?.open_orders.filter(o => !o.is_lend_order) || []}
                    provider={provider}
                    market={markets[selectedMarket]}
                    marginAccount={account}
                    cluster={cluster}
                    blockExplorer={explorer}
                    pools={pools.tokenPools}
                    markets={markets.map(m => m.market)}
                  />
                )
            },
            {
              label: (
                <TabLink
                  name="Open Borrows"
                  amount={positionsData?.total_borrowed}
                  decimals={markets[selectedMarket].token.decimals}
                />
              ),
              key: 'open-borrows',
              children: positionsLoading ? (
                <LoadingOutlined />
              ) : (
                <OpenBorrowsTable data={positionsData.loans} market={markets[selectedMarket]} />
              )
            }
          ]}
          size="large"
        />
      )}
      <ReorderArrows component="debtTable" order={accountsViewOrder} setOrder={setAccountsViewOrder} vertical />
    </div>
  );
}
