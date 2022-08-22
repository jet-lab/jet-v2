import { useEffect, useRef, useState } from 'react';
import { CSVDownload } from 'react-csv';
import { useReactToPrint } from 'react-to-print';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Order } from '@project-serum/serum/lib/market';
import { AccountTransaction } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { BlockExplorer, Cluster, PreferDayMonthYear, PreferredTimeDisplay } from '../../state/settings/settings';
import { AccountsViewOrder } from '../../state/views/views';
import { WalletInit } from '../../state/user/walletTokens';
import { AccountsInit, AccountOrder, CurrentAccountHistory, AccountNames } from '../../state/user/accounts';
import { ActionRefresh } from '../../state/actions/actions';
import { createDummyArray, getExplorerUrl } from '../../utils/ui';
import { formatMarketPair } from '../../utils/format';
import { notify } from '../../utils/notify';
import { localDayMonthYear, unixToLocalTime, unixToUtcTime, utcDayMonthYear } from '../../utils/time';
import { ActionResponse, useMarginActions } from '../../utils/jet/marginActions';
import { Tabs, Table, Skeleton, Typography, Input, Dropdown, Menu, Popover } from 'antd';
import { ReorderArrows } from '../misc/ReorderArrows';
import { ConnectionFeedback } from '../misc/ConnectionFeedback';
import { DownloadOutlined, PrinterFilled, SearchOutlined } from '@ant-design/icons';
import { ReactComponent as AngleDown } from '../../styles/icons/arrow-angle-down.svg';
import { ReactComponent as CancelIcon } from '../../styles/icons/cancel-icon.svg';
import { ActionIcon } from '../misc/ActionIcon';

export function FullAccountHistory(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { cancelOrder } = useMarginActions();
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const preferDayMonthYear = useRecoilValue(PreferDayMonthYear);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const walletInit = useRecoilValue(WalletInit);
  const currentAccountHistory = CurrentAccountHistory();
  const [filteredOrderHistory, setFilteredOrderHistory] = useState<AccountOrder[] | undefined>(
    currentAccountHistory?.orders
  );
  const [filteredOpenOrderHistory, setFilteredOpenOrderHistory] = useState<AccountOrder[] | undefined>(undefined);
  const [filteredFillHistory, setFilteredFillHistory] = useState<AccountOrder[] | undefined>(undefined);
  const [filteredTransactionHistory, setFilteredTransactionHistory] = useState<AccountTransaction[] | undefined>(
    currentAccountHistory?.transactions
  );
  const accountNames = useRecoilValue(AccountNames);
  const accountsInit = useRecoilValue(AccountsInit);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [currentTable, setCurrentTable] = useState('openOrders');
  const [pageSize, setPageSize] = useState(5);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const openOrdersRef = useRef<any>();
  const printOpenOrdersPdf = useReactToPrint({
    content: () => openOrdersRef.current
  });
  const ordersRef = useRef<any>();
  const printOrdersPdf = useReactToPrint({
    content: () => ordersRef.current
  });
  const fillsRef = useRef<any>();
  const printFillsPdf = useReactToPrint({
    content: () => fillsRef.current
  });
  const transactionsRef = useRef<any>();
  const printTransactionsPdf = useReactToPrint({
    content: () => transactionsRef.current
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

  // Table data
  const openOrderTableColumns = [
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
      title: dictionary.common.market,
      key: 'market',
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          formatMarketPair(order.pair)
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
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
      title: dictionary.accountsView.filled,
      key: 'filled',
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
      title: dictionary.tradeView.orderEntry.type,
      key: 'type',
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.type ? (
          accountsInit && order?.type ? (
            // @ts-ignore
            <Text className="order-type">{dictionary.tradeView.orderEntry[type]}</Text>
          ) : (
            <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
          )
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: '',
      key: 'cancel',
      align: 'right' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit ? (
          <div className="cancel-order flex align-center justify-end">
            <CancelIcon className="jet-icon cancel-icon" onClick={() => cancelOpenOrder(order.serumOrder)} />
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    }
  ];
  const orderTableColumns = [
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
      title: dictionary.common.market,
      key: 'market',
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          formatMarketPair(order.pair)
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
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
          <Text className="order-type">{dictionary.tradeView.orderEntry[type]}</Text>
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
              <div className="cancel-order flex align-center justify-start">
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
  const fillTableColumns = [
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
      title: dictionary.common.market,
      key: 'market',
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.pair ? (
          formatMarketPair(order.pair)
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
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
      align: 'left' as any,
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
      align: 'left' as any,
      width: 50,
      render: (value: any, order: AccountOrder) =>
        accountsInit && order?.type ? (
          // @ts-ignore
          <Text className="order-type">{dictionary.tradeView.orderEntry[type]}</Text>
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
  const transactionHistoryColumns = [
    {
      title: dictionary.accountsView.timePlaced,
      key: 'time',
      align: 'left' as any,
      width: 200,
      render: (value: any, transaction: AccountTransaction) =>
        accountsInit && transaction?.timestamp ? (
          preferredTimeDisplay === 'local' ? (
            `${localDayMonthYear(transaction.timestamp, preferDayMonthYear)}, ${unixToLocalTime(transaction.timestamp)}`
          ) : (
            `${utcDayMonthYear(transaction.timestamp, preferDayMonthYear)}, ${unixToUtcTime(transaction.timestamp)}`
          )
        ) : (
          <Skeleton className="align-left" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.activity,
      key: 'action',
      align: 'center' as any,
      render: (value: any, transaction: AccountTransaction) =>
        accountsInit && transaction?.tradeAction ? (
          <div className={`account-table-action-${transaction.tradeAction} flex-centered`}>
            <ActionIcon action={transaction.tradeAction} />
            &nbsp;
            {transaction.tradeAction === 'transfer' && transaction.fromAccount
              ? dictionary.accountsView.transferFrom.replace(
                  '{{TRANSFER_ACCOUNT_NAME}}',
                  accountNames[transaction.fromAccount.toString()] ?? ''
                )
              : transaction.toAccount
              ? dictionary.accountsView.transferTo.replace(
                  '{{TRANSFER_ACCOUNT_NAME}}',
                  accountNames[transaction.toAccount.toString()] ?? ''
                )
              : dictionary.actions[transaction.tradeAction].title}
          </div>
        ) : (
          <Skeleton className="align-center" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.common.token,
      key: 'token',
      align: 'center' as any,
      render: (value: any, transaction: AccountTransaction) =>
        accountsInit && transaction?.tokenSymbol ? (
          transaction.tokenSymbol
        ) : (
          <Skeleton className="align-center" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.common.amount,
      key: 'amount',
      align: 'right' as any,
      render: (value: any, transaction: AccountTransaction) =>
        accountsInit && transaction?.tradeAmount ? (
          <Text>{transaction.tradeAmount.tokens}</Text>
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    }
  ];

  // Update filteredOrderHistory and filteredTransactionHistory on currentAccountHistory init/change
  useEffect(() => {
    if (currentAccountHistory) {
      setFilteredOrderHistory(currentAccountHistory.orders);
      setFilteredTransactionHistory(currentAccountHistory.transactions);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [accountsInit, currentAccountHistory, actionRefresh]);

  // Any time filteredOrderHistory updates, update fills and open orders tables
  useEffect(() => {
    if (filteredOrderHistory) {
      const filteredOpenOrderHistory: AccountOrder[] = [];
      const filteredFillHistory: AccountOrder[] = [];
      for (const order of filteredOrderHistory) {
        if (order.status === 'open') {
          filteredOpenOrderHistory.push(order);
        } else if (order.status === 'filled') {
          filteredFillHistory.push(order);
        }
      }
      setFilteredOpenOrderHistory(filteredOpenOrderHistory);
      setFilteredFillHistory(filteredFillHistory);
    }
  }, [filteredOrderHistory]);

  return (
    <div className="full-account-history account-table view-element view-element-hidden flex-centered">
      <ConnectionFeedback />
      <Tabs
        className="view-element-item view-element-item-hidden"
        activeKey={currentTable}
        onChange={table => setCurrentTable(table)}>
        <TabPane key="openOrders" tab={dictionary.accountsView.openOrders}>
          <Table
            ref={openOrdersRef}
            dataSource={accountsInit ? filteredOpenOrderHistory : createDummyArray(10, 'address')}
            columns={openOrderTableColumns}
            pagination={{ pageSize }}
            className="no-row-interaction"
            rowKey={row => `${row.pair}-${Math.random()}`}
            rowClassName={(order, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
            locale={{ emptyText: dictionary.accountsView.noOpenOrders }}
          />
          <div className="download-btns flex-centered">
            <DownloadOutlined
              onClick={() => {
                setDownloadCsv(true);
                setTimeout(() => setDownloadCsv(false), 1000);
              }}
            />
            {downloadCsv && filteredOpenOrderHistory && (
              // @ts-ignore
              <CSVDownload
                filename={'Jet_OPEN_ORDERS.csv'}
                data={filteredOpenOrderHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredOpenOrderHistory ? printOpenOrdersPdf() : null)} />
          </div>
        </TabPane>
        <TabPane key="orders" tab={dictionary.accountsView.orderHistory}>
          <Table
            ref={ordersRef}
            dataSource={accountsInit ? filteredOrderHistory : createDummyArray(pageSize, 'signature')}
            columns={orderTableColumns}
            pagination={{ pageSize }}
            className={accountsInit && filteredOrderHistory?.length ? '' : 'no-row-interaction'}
            rowKey={row => `${row.pair}-${Math.random()}`}
            rowClassName={(order, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
            onRow={(order: AccountOrder) => {
              return {
                onClick: () =>
                  order.status !== 'open'
                    ? window.open(getExplorerUrl(order.signature, cluster, blockExplorer), '_blank', 'noopener')
                    : null
              };
            }}
            locale={{ emptyText: dictionary.accountsView.noOrderHistory }}
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
                filename={`Jet_ORDER_HISTORY.csv`}
                data={filteredOrderHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredOrderHistory ? printOrdersPdf() : null)} />
          </div>
        </TabPane>
        <TabPane key="fills" tab={dictionary.accountsView.fillHistory}>
          <Table
            ref={fillsRef}
            dataSource={accountsInit ? filteredFillHistory : createDummyArray(pageSize, 'signature')}
            columns={fillTableColumns}
            pagination={{ pageSize }}
            className={accountsInit && filteredFillHistory?.length ? '' : 'no-row-interaction'}
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
            {downloadCsv && filteredFillHistory && (
              // @ts-ignore
              <CSVDownload
                filename={`Jet_FILLS_HISTORY.csv`}
                data={filteredFillHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled onClick={() => (accountsInit && filteredFillHistory ? printFillsPdf() : null)} />
          </div>
        </TabPane>
        <TabPane key="transactions" tab={dictionary.accountsView.accountHistory}>
          <Table
            ref={transactionsRef}
            dataSource={accountsInit ? filteredTransactionHistory : createDummyArray(pageSize, 'signature')}
            columns={transactionHistoryColumns}
            pagination={{ pageSize }}
            className={accountsInit && filteredTransactionHistory?.length ? '' : 'no-row-interaction'}
            rowKey={row => `${row.tokenSymbol}-${Math.random()}`}
            rowClassName={(transaction, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
            onRow={(transaction: AccountTransaction) => {
              return {
                onClick: () =>
                  window.open(getExplorerUrl(transaction.signature, cluster, blockExplorer), '_blank', 'noopener')
              };
            }}
            locale={{ emptyText: dictionary.accountsView.noAccountHistory }}
          />
          <div className="download-btns flex-centered">
            <DownloadOutlined
              onClick={() => {
                setDownloadCsv(true);
                setTimeout(() => setDownloadCsv(false), 1000);
              }}
            />
            {downloadCsv && filteredTransactionHistory && (
              // @ts-ignore
              <CSVDownload
                filename={`Jet_FILLS_HISTORY.csv`}
                data={filteredTransactionHistory ?? ''}
                target="_blank"></CSVDownload>
            )}
            <PrinterFilled
              onClick={() => (accountsInit && filteredTransactionHistory ? printTransactionsPdf() : null)}
            />
          </div>
        </TabPane>
      </Tabs>
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
            // Filter accountHistory.orders
            if (currentAccountHistory?.orders) {
              const filteredOrderHistory: AccountOrder[] = [];
              for (const order of currentAccountHistory?.orders) {
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
            // Filter accountHistory.transactions
            if (currentAccountHistory?.transactions) {
              const filteredTransactionHistory: AccountTransaction[] = [];
              for (const transaction of currentAccountHistory?.transactions) {
                const orderDate =
                  preferredTimeDisplay === 'local'
                    ? localDayMonthYear(transaction.timestamp, preferDayMonthYear)
                    : utcDayMonthYear(transaction.timestamp, preferDayMonthYear);
                const orderTime =
                  preferredTimeDisplay === 'local'
                    ? unixToLocalTime(transaction.timestamp)
                    : unixToUtcTime(transaction.timestamp);
                if (
                  transaction.signature.toLowerCase().includes(query) ||
                  orderDate.toLowerCase().includes(query) ||
                  orderTime.toLowerCase().includes(query) ||
                  transaction.tokenName.toLowerCase().includes(query) ||
                  transaction.tokenName.toLowerCase().includes(query) ||
                  transaction.tradeAction?.toLowerCase().includes(query)
                ) {
                  filteredTransactionHistory.push(transaction);
                }
              }
              setFilteredTransactionHistory(filteredTransactionHistory);
            }
          }}
        />
      </div>
      <ReorderArrows
        component="fullAccountHistory"
        order={accountsViewOrder}
        setOrder={setAccountsViewOrder}
        vertical
      />
    </div>
  );
}
