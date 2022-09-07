import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import reactStringReplace from 'react-string-replace';
import { CSVDownload } from 'react-csv';
import { useReactToPrint } from 'react-to-print';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Order } from '@project-serum/serum/lib/market';
import { Dictionary } from '../../state/settings/localization/localization';
import {
  BlockExplorer,
  Cluster,
  FiatCurrency,
  PreferDayMonthYear,
  PreferredTimeDisplay
} from '../../state/settings/settings';
import { TradeViewOrder } from '../../state/views/views';
import { WalletInit } from '../../state/user/walletTokens';
import { Pools } from '../../state/borrow/pools';
import { CurrentMarketPair } from '../../state/trade/market';
import {
  AccountBalance,
  AccountsInit,
  AccountOrder,
  CurrentAccount,
  CurrentAccountHistory
} from '../../state/user/accounts';
import { ActionRefresh } from '../../state/actions/actions';
import { useCurrencyFormatting } from '../../utils/currency';
import { animateViewOut, APP_TRANSITION_TIMEOUT, createDummyArray, getExplorerUrl } from '../../utils/ui';
import { formatRate } from '../../utils/format';
import { localDayMonthYear, unixToLocalTime, unixToUtcTime, utcDayMonthYear } from '../../utils/time';
import { notify } from '../../utils/notify';
import { ActionResponse, useMarginActions } from '../../utils/jet/marginActions';
import { Tabs, Table, Skeleton, Typography, Input, Dropdown, Menu, Popover } from 'antd';
import { TokenLogo } from '../misc/TokenLogo';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Info } from '../misc/Info';
import { ConnectionFeedback } from '../misc/ConnectionFeedback';
import { DownloadOutlined, PrinterFilled, SearchOutlined } from '@ant-design/icons';
import { ReactComponent as AngleDown } from '../../styles/icons/arrow-angle-down.svg';
import { ReactComponent as CancelIcon } from '../../styles/icons/cancel-icon.svg';

export function PairRelatedAccount(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { cancelOrder } = useMarginActions();
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const fiatCurrency = useRecoilValue(FiatCurrency);
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const preferDayMonthYear = useRecoilValue(PreferDayMonthYear);
  const [tradeViewOrder, setTradeViewOrder] = useRecoilState(TradeViewOrder);
  const walletInit = useRecoilValue(WalletInit);
  const pools = useRecoilValue(Pools);
  const currentMarketPair = useRecoilValue(CurrentMarketPair);
  const currentAccount = useRecoilValue(CurrentAccount);
  const [currentAccountBalances, setCurrentAccountBalances] = useState<AccountBalance[] | undefined>(undefined);
  const [filteredAccountBalances, setFilteredAccountBalances] = useState(currentAccountBalances);
  const currentAccountHistory = CurrentAccountHistory();
  const [pairOrderHistory, setPairOrderHistory] = useState<AccountOrder[] | undefined>(undefined);
  const [filteredOrderHistory, setFilteredOrderHistory] = useState<AccountOrder[] | undefined>(undefined);
  const [filteredFillHistory, setFilteredFillHistory] = useState<AccountOrder[] | undefined>(undefined);
  const accountsInit = useRecoilValue(AccountsInit);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [currentTable, setCurrentTable] = useState('balances');
  const [pageSize, setPageSize] = useState(5);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const accountBalancesRef = useRef<any>();
  const printBalancesPdf = useReactToPrint({
    content: () => accountBalancesRef.current
  });
  const ordersRef = useRef<any>();
  const printOrdersPdf = useReactToPrint({
    content: () => ordersRef.current
  });
  const fillsRef = useRef<any>();
  const printFillsPdf = useReactToPrint({
    content: () => fillsRef.current
  });
  const { Paragraph, Text } = Typography;
  const { TabPane } = Tabs;

  // Cancel a Serum order
  async function cancelOpenOrder(serumOrder: Order) {
    const [txId, resp] = await cancelOrder(serumOrder);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.cancelOrder.successTitle,
        dictionary.notifications.cancelOrder.successDescription.replaceAll(
          '{{ORDER_ID}}',
          serumOrder.orderId.toString()
        ),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.cancelOrder.cancelledTitle,
        dictionary.notifications.cancelOrder.cancelledDescription.replaceAll(
          '{{ORDER_ID}}',
          serumOrder.orderId.toString()
        ),
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.cancelOrder.failedTitle,
        dictionary.notifications.cancelOrder.failedDescription,
        'error'
      );
    }
  }

  // Set up full accounts text with link
  const navigate = useNavigate();
  let fullAccountsText = reactStringReplace(dictionary.accountsView.fullAccountsLink, '{{ACCOUNTS_LINK}}', () => (
    <Paragraph
      key="accounts-link"
      className="link-btn"
      onClick={() => {
        setTimeout(() => {
          navigate('/accounts', { replace: true });
        }, APP_TRANSITION_TIMEOUT);
        animateViewOut();
      }}>
      {dictionary.accountsView.title}
    </Paragraph>
  ));
  fullAccountsText = reactStringReplace(fullAccountsText, '{{CURRENT_PAIR}}', () => currentMarketPair);

  // Table data
  const balanceTableColumns = [
    {
      title: dictionary.common.token,
      key: 'token',
      align: 'left' as any,
      width: 175,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          <div className="table-token">
            <TokenLogo height={22} symbol={balance.tokenSymbol} />
            <Text className="table-token-name" strong>
              {balance.tokenName}
            </Text>
            <Text className="table-token-abbrev" strong>
              {balance.tokenSymbol}
            </Text>
            {accountsInit && pools ? (
              <Text>{`${balance.tokenSymbol} ≈ ${currencyFormatter(
                pools.tokenPools[balance.tokenSymbol]?.tokenPrice ?? 0,
                true,
                balance.tokenSymbol === 'USDC' ? 0 : undefined,
                balance.tokenSymbol === 'USDC'
              )}`}</Text>
            ) : (
              <Skeleton paragraph={false} active={walletInit && !accountsInit} />
            )}
          </div>
        ) : (
          <Skeleton avatar paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.currentBalance,
      key: 'currentBalance',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          balance.netBalance.uiTokens
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.fiatValue.replace('{{FIAT_CURRENCY}}', fiatCurrency),
      key: 'value',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.fiatValue ? (
          balance.fiatValue
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.percentOfPortfolio,
      key: 'percent',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          formatRate(balance?.percentageOfPortfolio ?? 0)
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: (
        <Info term="depositBorrowRate">
          <Text className="info-element">{dictionary.accountsView.depositBorrowRates}</Text>
        </Info>
      ),
      key: 'depositBorrowRate',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          <div className="flex align-center justify-end">
            <Text type="success">{formatRate(balance.depositRate)}</Text>&nbsp;/&nbsp;
            <Text type="danger">{formatRate(balance.borrowRate)}</Text>
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    }
  ];
  const ordersTableColumns = [
    {
      title: dictionary.accountsView.timePlaced,
      key: 'time',
      align: 'left' as any,
      width: 200,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.dateString ? (
          order.dateString
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.side,
      key: 'side',
      align: 'center' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.side ? (
          <Text type={order.side === 'buy' ? 'success' : 'danger'}>{order.side.toUpperCase()}</Text>
        ) : (
          <Skeleton className="align-center" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.totalSize,
      key: 'size',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.size} ${order.pair.split('/')[0]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.filledSize,
      key: 'filledSize',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.filledSize} ${order.pair.split('/')[0]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.common.price,
      key: 'price',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.price} ${order.pair.split('/')[1]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.averageFillPrice,
      key: 'averageFillPrice',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.aveFillPrice} ${order.pair.split('/')[1]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.tradeView.orderEntry.type,
      key: 'type',
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.type ? (
          // @ts-ignore
          <Text className="order-type">{dictionary.tradeView.orderEntry[option]}</Text>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.status,
      key: 'status',
      align: 'left' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.status ? (
          <div className="flex align-center justify-start">
            <Text>
              {order.status === 'partialFilled'
                ? dictionary.accountsView.filled.toUpperCase()
                : order.status.toUpperCase()}
            </Text>
            {order.status === 'partialFilled' ? (
              <Text className="partial-fill-indicator">{dictionary.accountsView.partial.toUpperCase()}</Text>
            ) : (
              ''
            )}
            {accountsInit && order && order.status === 'open' ? (
              <div className="cancel-order flex align-center justify-end">
                <CancelIcon className="jet-icon cancel-icon" onClick={() => cancelOpenOrder(order.serumOrder)} />
              </div>
            ) : (
              ''
            )}
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.fees,
      key: 'fees',
      align: 'left' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.totalFees ? (
          <div className="flex align-center justify-end">
            {`${order.totalFees} SOL`}
            <Popover
              placement="right"
              content={
                <div className="flex column">
                  <div
                    className={`popover-fee-tier popover-fee-tier-full popover-fee-tier-${order.serumFeeTier} flex-centered`}>
                    {`${dictionary.accountsView.feeTier} ${order.serumFeeTier}`}
                  </div>
                  <div className="flex align-center justify-between">
                    <Text className="popover-fee-tier-text">Serum</Text>
                    <Text className="popover-fee-tier-text">{order.serumFees}</Text>
                  </div>
                  <div className="flex align-center justify-between">
                    <Text className="popover-fee-tier-text">Solana</Text>
                    <Text className="popover-fee-tier-text">{order.solanaFees}</Text>
                  </div>
                </div>
              }>
              <div
                className={`popover-fee-tier popover-fee-tier-trigger popover-fee-tier-${order.serumFeeTier} flex-centered`}>
                {order.serumFeeTier}
              </div>
            </Popover>
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    }
  ];
  const fillsTableColumns = [
    {
      title: dictionary.accountsView.timePlaced,
      key: 'time',
      align: 'left' as any,
      width: 200,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.timestamp ? (
          preferredTimeDisplay === 'local' ? (
            `${localDayMonthYear(order.timestamp, preferDayMonthYear)}, ${unixToLocalTime(order.timestamp)}`
          ) : (
            `${utcDayMonthYear(order.timestamp, preferDayMonthYear)}, ${unixToUtcTime(order.timestamp)}`
          )
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.side,
      key: 'side',
      align: 'center' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.side ? (
          <Text type={order.side === 'buy' ? 'success' : 'danger'}>{order.side.toUpperCase()}</Text>
        ) : (
          <Skeleton className="align-center" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.common.size,
      key: 'size',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.size} ${order.pair.split('/')[0]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.common.price,
      key: 'price',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          `${order.price} ${order.pair.split('/')[1]}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.tradeView.orderEntry.type,
      key: 'type',
      align: 'right' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.type ? (
          // @ts-ignore
          <Text className="order-type">{dictionary.tradeView.orderEntry[option]}</Text>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.fees,
      key: 'fees',
      align: 'right' as any,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.totalFees ? (
          <div className="flex align-center justify-end">
            {`${order.totalFees} SOL`}
            <Popover
              placement="right"
              content={
                <div className="flex column">
                  <div
                    className={`popover-fee-tier popover-fee-tier-full popover-fee-tier-${order.serumFeeTier} flex-centered`}>
                    {`${dictionary.accountsView.feeTier} ${order.serumFeeTier}`}
                  </div>
                  <div className="flex align-center justify-between">
                    <Text className="popover-fee-tier-text">Serum</Text>
                    <Text className="popover-fee-tier-text">{order.serumFees}</Text>
                  </div>
                  <div className="flex align-center justify-between">
                    <Text className="popover-fee-tier-text">Solana</Text>
                    <Text className="popover-fee-tier-text">{order.solanaFees}</Text>
                  </div>
                </div>
              }>
              <div
                className={`popover-fee-tier popover-fee-tier-trigger popover-fee-tier-${order.serumFeeTier} flex-centered`}>
                {order.serumFeeTier}
              </div>
            </Popover>
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    }
  ];

  // Any time filteredOrderHistory updates, update fills only table
  useEffect(() => {
    if (filteredOrderHistory) {
      const filteredFillHistory: AccountOrder[] = filteredOrderHistory.filter(order => order.status === 'filled');
      setFilteredFillHistory(filteredFillHistory);
    }
  }, [currentMarketPair, filteredOrderHistory]);

  // Update filteredOrderHistory on currentAccountHistory or currentMarketPair init/change
  useEffect(() => {
    if (currentAccountHistory) {
      const pairOrderHistory: AccountOrder[] = [];
      const filteredOrderHistory: AccountOrder[] = [];
      for (const order of currentAccountHistory?.orders) {
        if (order.pair === currentMarketPair) {
          const dateString =
            preferredTimeDisplay === 'local'
              ? `${localDayMonthYear(order.timestamp, preferDayMonthYear)}, ${unixToLocalTime(order.timestamp)}`
              : `${utcDayMonthYear(order.timestamp, preferDayMonthYear)}, ${unixToUtcTime(order.timestamp)}`;
          pairOrderHistory.push({ ...order, dateString });
          filteredOrderHistory.push({ ...order, dateString });
        }
      }
      setPairOrderHistory(filteredOrderHistory);
      setFilteredOrderHistory(filteredOrderHistory);
    }
  }, [currentMarketPair, currentAccountHistory, preferredTimeDisplay, preferDayMonthYear]);

  // Create array of account balances for current account
  useEffect(() => {
    if (currentAccount && pools) {
      const accountBalances: AccountBalance[] = [];
      for (const token of Object.values(pools.tokenPools)) {
        const poolPosition = currentAccount.poolPositions[token.symbol];
        if (poolPosition) {
          const netBalance = poolPosition.depositBalance.sub(poolPosition.loanBalance);
          const fiatValue = currencyAbbrev(poolPosition.depositValue - poolPosition.loanValue, true);
          const percentage =
            (poolPosition.depositBalance.tokens + poolPosition.loanBalance.tokens) /
            (currentAccount.summary.depositedValue + currentAccount.summary.borrowedValue);
          const orders = currentAccountHistory?.orders;
          const inOrders: Record<string, number> = {};
          if (orders) {
            for (const order of orders) {
              if (order.status !== 'cancelled' && order.status !== 'filled') {
                if (order.status === 'partialFilled') {
                  const remainingAmount = order.size - order.filledSize;
                  inOrders[order.pair.split('/')[0]] = remainingAmount;
                } else {
                  inOrders[order.pair.split('/')[0]] = order.size;
                }
              }
            }
          }
          accountBalances.push({
            tokenName: token.name ?? dictionary.common.notAvailable,
            tokenSymbol: token.symbol,
            depositBalance: poolPosition.depositBalance,
            loanBalance: poolPosition.loanBalance,
            netBalance,
            inOrders,
            fiatValue,
            percentageOfPortfolio: !isNaN(percentage) ? percentage : 0,
            depositRate: token.depositApy,
            borrowRate: token.borrowApr
          });
        }
      }
      setCurrentAccountBalances(accountBalances);
      setFilteredAccountBalances(accountBalances);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pools, accountsInit, actionRefresh]);

  return (
    <div className="pair-related-account account-table view-element view-element-hidden flex-centered">
      <ConnectionFeedback />
      <Tabs
        className="view-element-item view-element-item-hidden"
        activeKey={currentTable}
        onChange={table => setCurrentTable(table)}>
        <TabPane key="balances" tab={dictionary.accountsView.balances}>
          <Table
            ref={accountBalancesRef}
            dataSource={filteredAccountBalances ?? createDummyArray(Object.keys(pools?.tokenPools ?? {}).length, 'key')}
            columns={balanceTableColumns}
            pagination={{ pageSize }}
            className="no-row-interaction balance-table"
            rowKey={row => `${row.tokenSymbol}-${Math.random()}`}
            locale={{ emptyText: dictionary.accountsView.noBalances }}
          />
          <div className="download-btns flex-centered">
            <DownloadOutlined
              onClick={() => {
                setDownloadCsv(true);
                setTimeout(() => setDownloadCsv(false), 1000);
              }}
            />
            {downloadCsv && filteredAccountBalances && (
              // @ts-ignore
              <CSVDownload
                filename={'Jet_Pool_Balances.csv'}
                data={filteredAccountBalances ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredAccountBalances ? printBalancesPdf() : null)} />
          </div>
        </TabPane>
        <TabPane
          key="orders"
          tab={
            <div className="flex-centered">
              <Paragraph>{dictionary.accountsView.orders}</Paragraph>
              <Text>{currentMarketPair}</Text>
            </div>
          }>
          <Table
            ref={ordersRef}
            dataSource={accountsInit ? filteredOrderHistory : createDummyArray(10, 'address')}
            columns={ordersTableColumns}
            pagination={{ pageSize }}
            className={`small-table ${accountsInit && filteredOrderHistory?.length ? '' : 'no-row-interaction'}`}
            rowKey={row => `${row.pair}-${Math.random()}`}
            rowClassName={(order, index) =>
              `${(index + 1) % 2 === 0 ? 'dark-bg' : ''} ${order.status === 'open' ? 'no-interaction' : ''}`
            }
            onRow={(order: AccountOrder) => {
              return {
                onClick: () =>
                  order.status !== 'open'
                    ? window.open(getExplorerUrl(order.signature, cluster, blockExplorer), '_blank', 'noopener')
                    : null
              };
            }}
            locale={{ emptyText: dictionary.accountsView.noOrders }}
          />
          <div className="download-btns flex-centered">
            <DownloadOutlined
              onClick={() => {
                setDownloadCsv(true);
                setTimeout(() => setDownloadCsv(false), 1000);
              }}
            />
            {downloadCsv && filteredOrderHistory && (
              // @ts-ignore
              <CSVDownload
                filename={`Jet_${currentMarketPair}_ORDERS.csv`}
                data={filteredOrderHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredOrderHistory ? printOrdersPdf() : null)} />
          </div>
        </TabPane>
        <TabPane
          key="fills"
          tab={
            <div className="flex-centered">
              <Paragraph>{dictionary.accountsView.fills}</Paragraph>
              <Text>{currentMarketPair}</Text>
            </div>
          }>
          <Table
            ref={fillsRef}
            dataSource={accountsInit ? filteredFillHistory : createDummyArray(10, 'address')}
            columns={fillsTableColumns}
            pagination={{ pageSize }}
            className={`small-table ${accountsInit && filteredFillHistory?.length ? '' : 'no-row-interaction'}`}
            rowKey={row => `${row.pair}-${Math.random()}`}
            rowClassName={(order, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
            onRow={(order: AccountOrder) => {
              return {
                onClick: () =>
                  window.open(getExplorerUrl(order.signature, cluster, blockExplorer), '_blank', 'noopener')
              };
            }}
            locale={{ emptyText: dictionary.accountsView.noFills }}
          />
          <div className="download-btns flex-centered">
            <DownloadOutlined
              onClick={() => {
                setDownloadCsv(true);
                setTimeout(() => setDownloadCsv(false), 1000);
              }}
            />
            {downloadCsv && currentAccountHistory && (
              // @ts-ignore
              <CSVDownload
                filename={`Jet_FILLS_HISTORY.csv`}
                data={filteredFillHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredFillHistory ? printFillsPdf() : null)} />
          </div>
        </TabPane>
      </Tabs>
      <Paragraph className="full-accounts-text view-element-item view-element-item-hidden flex-centered">
        {fullAccountsText}
      </Paragraph>
      <div className="page-size-dropdown view-element-item view-element-item-hidden flex-centered">
        <Paragraph italic>{dictionary.accountsView.rowsPerPage}:</Paragraph>
        <Dropdown
          overlay={
            <Menu className="min-width-menu-sm">
              {[5, 10, 25, 50, 100].map(size => (
                <Menu.Item key={size} onClick={() => setPageSize(size)} className={size === pageSize ? 'active' : ''}>
                  {size}
                </Menu.Item>
              ))}
            </Menu>
          }>
          <Text type="secondary">
            {pageSize}
            <AngleDown className="jet-icon" />
          </Text>
        </Dropdown>
      </div>
      <div className="account-table-search view-element-item view-element-item-hidden">
        <SearchOutlined />
        <Input
          type="text"
          placeholder={
            currentTable === 'balances'
              ? dictionary.accountsView.balancesFilterPlaceholder
              : currentTable === 'orders'
              ? dictionary.accountsView.ordersFilterPlaceholder
              : dictionary.accountsView.fillsFilterPlaceholder
          }
          onChange={e => {
            const query = e.target.value.toLowerCase();
            // Filter accountBalances
            if (currentAccountBalances) {
              const filteredAccountBalances: AccountBalance[] = [];
              for (const balance of currentAccountBalances) {
                if (
                  balance.tokenName.toLowerCase().includes(query) ||
                  balance.tokenSymbol.toLowerCase().includes(query)
                ) {
                  filteredAccountBalances.push(balance);
                }
              }
              setFilteredAccountBalances(filteredAccountBalances);
            }
            // Filter accountHistory.orders
            if (pairOrderHistory) {
              const filteredOrderHistory: AccountOrder[] = [];
              for (const order of pairOrderHistory) {
                const orderDate =
                  preferredTimeDisplay === 'local'
                    ? localDayMonthYear(order.timestamp, preferDayMonthYear)
                    : utcDayMonthYear(order.timestamp, preferDayMonthYear);
                const orderTime =
                  preferredTimeDisplay === 'local' ? unixToLocalTime(order.timestamp) : unixToUtcTime(order.timestamp);
                if (
                  order.signature.toLowerCase().includes(query) ||
                  orderDate.toLowerCase().includes(query) ||
                  orderTime.toLowerCase().includes(query) ||
                  order.pair.toLowerCase().includes(query) ||
                  order.side.toLowerCase().includes(query) ||
                  order.status.toLowerCase().includes(query)
                ) {
                  filteredOrderHistory.push(order);
                }
              }
              setFilteredOrderHistory(filteredOrderHistory);
            }
          }}
        />
      </div>
      <ReorderArrows component="pairRelatedAccount" order={tradeViewOrder} setOrder={setTradeViewOrder} vertical />
    </div>
  );
}
