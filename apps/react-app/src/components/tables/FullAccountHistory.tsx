import { useEffect, useRef, useState } from 'react';
import { CSVDownload } from 'react-csv';
import { useRecoilState, useRecoilValue } from 'recoil';
import { FlightLog, PoolAction } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { PreferDayMonthYear, PreferredTimeDisplay } from '@state/settings/settings';
import { AccountsViewOrder } from '@state/views/views';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts, CurrentAccountHistory, AccountHistoryLoaded } from '@state/user/accounts';
import { ActionRefresh } from '@state/actions/actions';
import { createDummyArray, getExplorerUrl, openLinkInBrowser } from '@utils/ui';
import { localDayMonthYear, unixToLocalTime, unixToUtcTime, utcDayMonthYear } from '@utils/time';
import { Tabs, Table, Skeleton, Typography, Input, Dropdown } from 'antd';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { DownloadOutlined, SearchOutlined } from '@ant-design/icons';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { ActionIcon, FixedTermAction } from '@components/misc/ActionIcon';
import debounce from 'lodash.debounce';
import { useJetStore } from '@jet-lab/store';
import { Pools } from '@state/pools/pools';

// Table to show margin account's transaction history
export function FullAccountHistory(): JSX.Element {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const preferDayMonthYear = useRecoilValue(PreferDayMonthYear);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const walletTokens = useRecoilValue(WalletTokens);
  const accountHistoryLoaded = useRecoilValue(AccountHistoryLoaded);
  const currentAccountHistory = useRecoilValue(CurrentAccountHistory);
  const [filteredTxHistory, setFilteredTxHistory] = useState<FlightLog[] | undefined>(
    currentAccountHistory?.transactions
  );
  const accounts = useRecoilValue(Accounts);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [currentTable, setCurrentTable] = useState('transactions');
  const [pageSize, setPageSize] = useState(5);
  const pools = useRecoilValue(Pools);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const transactionsRef = useRef<any>();
  const loadingAccounts = walletTokens && !filteredTxHistory?.length;
  const { Paragraph, Text } = Typography;

  // Renders the date/time column for table
  function renderDateColumn(transaction: FlightLog) {
    let render = <Skeleton className="align-left" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.timestamp) {
      const dateTime =
        preferredTimeDisplay === 'local'
          ? `${localDayMonthYear(transaction.timestamp!.valueOf(), preferDayMonthYear)}, ${unixToLocalTime(transaction.timestamp!.valueOf())}`
          : `${utcDayMonthYear(transaction.timestamp!.valueOf(), preferDayMonthYear)}, ${unixToUtcTime(transaction.timestamp!.valueOf())}`;
      render = <Text>{dateTime}</Text>;
    }

    return render;
  }

  // Renders the activity column for table
  function renderActivityColumn(transaction: FlightLog) {
    let render = <Skeleton className="align-center" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.timestamp) {
      // TODO: no longer just a pool action, should include fixed term
      let action: PoolAction | FixedTermAction = "deposit";
      switch (transaction.activity_type) {
        case "Deposit":
          action = "deposit";
          break;
        case "Withdraw":
          action = "withdraw";
          break;
        case "MarginSwap":
          action = "swap";
          break;
        case "RouteSwap":
          action = "swap";
          break;
        case "MarginBorrow":
          action = "borrow";
          break;
        case "MarginRepay":
          action = "repay";
          break;
        case "Repay":
          action = "repay";
          break;
        case "BorrowNow":
          action = "borrow-now"
          break;
        case "LendNow":
          action = "lend-now"
          break;
        case "OfferLoan":
          action = "offer-loan"
          break;
        case "RequestLoan":
          action = "request-loan"
          break;
      }
      render = (
        <div className={`account-table-action-${action} flex-centered`}>
          <ActionIcon action={action} />
          &nbsp;
          {transaction.activity_type}
        </div>
      );
    }

    return render;
  }

  // Renders the token column for table
  function renderTokenColumn(transaction: FlightLog) {
    let render = <Skeleton className="align-center" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.token1_symbol) {
      render = (
        <Text>
          {transaction.token2_symbol
            ? `${transaction.token1_symbol} → ${transaction.token2_symbol}`
            : transaction.token1_symbol}
        </Text>
      );
    }

    return render;
  }

  // Renders the amount column for table
  function renderAmountColumn(transaction: FlightLog) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction.activity_value) {
      let token1_decimals = pools?.tokenPools[transaction.token1_symbol]?.decimals ?? 2;
      let token2_decimals = transaction.token2_symbol ? (pools?.tokenPools[transaction.token2_symbol]?.decimals ?? 2) : 2;
      render = (
        <Text>
          {transaction.token1_amount.toFixed(token1_decimals)}
          {transaction.token2_amount !== 0.0 && ` → ${transaction.token2_amount.toFixed(token2_decimals)}`}
        </Text>
      );
    }

    return render;
  }

  // Renders the value column for table
  function renderValueColumn(transaction: FlightLog) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction.activity_value) {
      const token1_value = transaction.activity_value.toFixed(2);
      const token2_value = (transaction.token2_amount * transaction.token2_price).toFixed(2);
      render = (
        <Text>
          {token1_value}
          {token2_value !== '0.00' && ` → ${token2_value}`}
        </Text>
      );
    }

    return render;
  }

  // Table data
  const transactionHistoryColumns = [
    {
      title: dictionary.accountsView.timePlaced,
      key: 'time',
      align: 'left' as any,
      width: 200,
      render: (_: string, transaction: FlightLog) => renderDateColumn(transaction)
    },
    {
      title: dictionary.accountsView.activity,
      key: 'action',
      align: 'center' as any,
      render: (_: string, transaction: FlightLog) => renderActivityColumn(transaction)
    },
    {
      title: dictionary.common.token,
      key: 'token',
      align: 'center' as any,
      render: (_: string, transaction: FlightLog) => renderTokenColumn(transaction)
    },
    {
      title: dictionary.common.amount,
      key: 'amount',
      align: 'right' as any,
      render: (_: string, transaction: FlightLog) => renderAmountColumn(transaction)
    },
    {
      title: dictionary.common.usdValue,
      key: 'value',
      align: 'right' as any,
      render: (_: string, transaction: FlightLog) => renderValueColumn(transaction)
    }
  ];

  // Returns placeholder text for filter input
  function getFilterInputPlaceholder() {
    let text = dictionary.accountsView.balancesFilterPlaceholder;
    if (currentTable === 'orders') {
      text = dictionary.accountsView.ordersFilterPlaceholder;
    } else if (currentTable === 'fills') {
      text = dictionary.accountsView.fillsFilterPlaceholder;
    }

    return text;
  }

  // Filters transaction history from a query
  function filterTxHistory(queryString: string) {
    const query = queryString.toLowerCase();
    if (currentAccountHistory?.transactions) {
      const filteredTxHistory: FlightLog[] = [];
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
          transaction.token1_name.toLowerCase().includes(query) ||
          transaction.token1_name.toLowerCase().includes(query) ||
          transaction.activity_type.toLowerCase().includes(query)
        ) {
          filteredTxHistory.push(transaction);
        }
      }
      setFilteredTxHistory(filteredTxHistory);
    }
  }

  // Update filteredTxHistory on currentAccountHistory init/change
  useEffect(() => {
    if (currentAccountHistory) {
      setFilteredTxHistory(currentAccountHistory.transactions);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [accounts, currentAccountHistory, actionRefresh]);

  const paginationSizes = [5, 10, 25, 50, 100].map(size => ({
    key: size,
    label: (
      <div onClick={() => setPageSize(size)} className={size == pageSize ? 'active' : ''}>
        {size}
      </div>
    )
  }));

  return (
    <div className="full-account-history account-table view-element flex-centered">
      <ConnectionFeedback />
      <Tabs
        activeKey={currentTable}
        onChange={table => setCurrentTable(table)}
        items={[
          {
            label: dictionary.accountsView.accountHistory,
            key: 'transactions',
            children: (
              <Table
                ref={transactionsRef}
                dataSource={
                  accounts && accountHistoryLoaded ? filteredTxHistory : createDummyArray(pageSize, 'signature')
                }
                columns={transactionHistoryColumns}
                pagination={{ pageSize }}
                className={accounts && filteredTxHistory?.length ? '' : 'no-row-interaction'}
                rowKey={row => `${row.token1_symbol}-${Math.random()}`}
                rowClassName={(_transaction, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
                onRow={(transaction: FlightLog) => ({
                  onClick: () => openLinkInBrowser(getExplorerUrl(transaction.signature, cluster, explorer))
                })}
                locale={{ emptyText: dictionary.accountsView.noAccountHistory }}
              />
            )
          }
        ]}></Tabs>
      <div className="page-size-dropdown flex-centered">
        <Paragraph italic>{dictionary.accountsView.rowsPerPage}:</Paragraph>
        <Dropdown menu={{ items: paginationSizes }}>
          <Text type="secondary">
            {pageSize}
            <AngleDown className="jet-icon" />
          </Text>
        </Dropdown>
      </div>
      <div className="account-table-search">
        <div className="download-btns flex-centered">
          <DownloadOutlined
            onClick={() => {
              setDownloadCsv(true);
              setTimeout(() => setDownloadCsv(false), 1000);
            }}
          />
          {downloadCsv && filteredTxHistory && (
            // @ts-ignore
            <CSVDownload
              filename={`Jet_FILLS_HISTORY.csv`}
              data={filteredTxHistory ?? ''}
              target="_blank"></CSVDownload>
          )}
        </div>
        <SearchOutlined />
        <Input
          type="text"
          placeholder={getFilterInputPlaceholder()}
          onChange={debounce(e => filterTxHistory(e.target.value), 300)}
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
