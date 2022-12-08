import { useEffect, useRef, useState } from 'react';
import { CSVDownload } from 'react-csv';
import { useRecoilState, useRecoilValue } from 'recoil';
import { AccountTransaction } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { BlockExplorer, Cluster, PreferDayMonthYear, PreferredTimeDisplay } from '@state/settings/settings';
import { AccountsViewOrder } from '@state/views/views';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts, CurrentAccountHistory, AccountNames, AccountHistoryLoaded } from '@state/user/accounts';
import { ActionRefresh } from '@state/actions/actions';
import { createDummyArray, getExplorerUrl, openLinkInBrowser } from '@utils/ui';
import { localDayMonthYear, unixToLocalTime, unixToUtcTime, utcDayMonthYear } from '@utils/time';
import { Tabs, Table, Skeleton, Typography, Input, Dropdown } from 'antd';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { DownloadOutlined, SearchOutlined } from '@ant-design/icons';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { ActionIcon } from '@components/misc/ActionIcon';
import debounce from 'lodash.debounce';

// Table to show margin account's transaction history
export function FullAccountHistory(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const preferDayMonthYear = useRecoilValue(PreferDayMonthYear);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const walletTokens = useRecoilValue(WalletTokens);
  const accountHistoryLoaded = useRecoilValue(AccountHistoryLoaded);
  const currentAccountHistory = useRecoilValue(CurrentAccountHistory);
  const [filteredTxHistory, setFilteredTxHistory] = useState<AccountTransaction[] | undefined>(
    currentAccountHistory?.transactions
  );
  const accountNames = useRecoilValue(AccountNames);
  const accounts = useRecoilValue(Accounts);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [currentTable, setCurrentTable] = useState('transactions');
  const [pageSize, setPageSize] = useState(5);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const transactionsRef = useRef<any>();
  const loadingAccounts = walletTokens && !filteredTxHistory?.length;
  const { Paragraph, Text } = Typography;

  // Renders the date/time column for table
  function renderDateColumn(transaction: AccountTransaction) {
    let render = <Skeleton className="align-left" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.timestamp) {
      const dateTime =
        preferredTimeDisplay === 'local'
          ? `${localDayMonthYear(transaction.timestamp, preferDayMonthYear)}, ${unixToLocalTime(transaction.timestamp)}`
          : `${utcDayMonthYear(transaction.timestamp, preferDayMonthYear)}, ${unixToUtcTime(transaction.timestamp)}`;
      render = <Text>{dateTime}</Text>;
    }

    return render;
  }

  // Renders the activity column for table
  function renderActivityColumn(transaction: AccountTransaction) {
    let render = <Skeleton className="align-center" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.timestamp) {
      const action = transaction.tradeAction.includes('repay') ? 'repay' : transaction.tradeAction;
      render = (
        <div className={`account-table-action-${action} flex-centered`}>
          <ActionIcon action={action} />
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
            : dictionary.actions[transaction.tradeAction.includes('repay') ? 'repay' : transaction.tradeAction]?.title}
        </div>
      );
    }

    return render;
  }

  // Renders the token column for table
  function renderTokenColumn(transaction: AccountTransaction) {
    let render = <Skeleton className="align-center" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.tokenSymbol) {
      render = (
        <Text>
          {transaction.tokenSymbolInput
            ? `${transaction.tokenSymbolInput} → ${transaction.tokenSymbol}`
            : transaction.tokenSymbol}
        </Text>
      );
    }

    return render;
  }

  // Renders the amount column for table
  function renderAmountColumn(transaction: AccountTransaction) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && transaction?.tradeAmount) {
      render = (
        <Text>
          {transaction.tradeAmountInput && `${transaction.tradeAmountInput?.uiTokens} → `}{' '}
          {transaction.tradeAmount.uiTokens}
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
      render: (_: string, transaction: AccountTransaction) => renderDateColumn(transaction)
    },
    {
      title: dictionary.accountsView.activity,
      key: 'action',
      align: 'center' as any,
      render: (_: string, transaction: AccountTransaction) => renderActivityColumn(transaction)
    },
    {
      title: dictionary.common.token,
      key: 'token',
      align: 'center' as any,
      render: (_: string, transaction: AccountTransaction) => renderTokenColumn(transaction)
    },
    {
      title: dictionary.common.amount,
      key: 'amount',
      align: 'right' as any,
      render: (_: string, transaction: AccountTransaction) => renderAmountColumn(transaction)
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
      const filteredTxHistory: AccountTransaction[] = [];
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
                rowKey={row => `${row.tokenSymbol}-${Math.random()}`}
                rowClassName={(_transaction, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
                onRow={(transaction: AccountTransaction) => ({
                  onClick: () => openLinkInBrowser(getExplorerUrl(transaction.signature, cluster, blockExplorer))
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
