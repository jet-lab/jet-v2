import { useEffect, useState } from 'react';
import { CSVDownload } from 'react-csv';
import { useRecoilState, useRecoilValue } from 'recoil';
import { AccountTransaction } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { PreferDayMonthYear, PreferredTimeDisplay } from '@state/settings/settings';
import { AccountsViewOrder } from '@state/views/views';
import { Accounts, CurrentAccountHistory } from '@state/user/accounts';
import { ActionRefresh } from '@state/actions/actions';
import { localDayMonthYear, unixToLocalTime, unixToUtcTime, utcDayMonthYear } from '@utils/time';
import { Tabs, Table, Typography, Input, Dropdown, Menu } from 'antd';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import {
  DownloadOutlined,
  SearchOutlined,
  PrinterOutlined,
  CloseOutlined,
  DownOutlined,
  RightOutlined
} from '@ant-design/icons';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import type { ColumnsType } from 'antd/es/table';
import debounce from 'lodash.debounce';

// Table to show margin account's transaction history
export function DebtTable(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const preferredTimeDisplay = useRecoilValue(PreferredTimeDisplay);
  const preferDayMonthYear = useRecoilValue(PreferDayMonthYear);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const currentAccountHistory = useRecoilValue(CurrentAccountHistory);
  const [filteredTxHistory, setFilteredTxHistory] = useState<AccountTransaction[] | undefined>(
    currentAccountHistory?.transactions
  );
  const accounts = useRecoilValue(Accounts);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [currentTable] = useState('transactions');
  const [pageSize, setPageSize] = useState(5);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const { Paragraph, Text } = Typography;

  // Dummy table data
  const postOrderColumns: ColumnsType<PostDataType> = [
    {
      title: dictionary.fixedView.debtTable.postOrder.id,
      dataIndex: 'id',
      key: 'id'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.placedTime,
      dataIndex: 'placedTime',
      key: 'placedTime'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.orderSize,
      dataIndex: 'orderSize',
      key: 'orderSize'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.filledSize,
      dataIndex: 'filledSize',
      key: 'filledSize'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.tenor,
      dataIndex: 'tenor',
      key: 'tenor'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.rate,
      dataIndex: 'rate',
      key: 'rate'
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.autoRoll,
      dataIndex: 'autoRoll',
      key: 'autoRoll',
      render: () => <>{true ? 'YES' : 'No'}</>
    },
    {
      title: dictionary.fixedView.debtTable.postOrder.cancel,
      dataIndex: 'cancel',
      key: 'cancel',
      render: () => <CloseOutlined style={{ color: '#e36868' }} onClick={() => {}} /> // color: --dt-danger
    }
  ];

  interface PostDataType {
    key: string;
    id: string;
    placedTime: string;
    orderSize: string;
    filledSize: string;
    tenor: string;
    rate: string;
    autoRoll: boolean;
    cancel: boolean;
  }

  const postDataDummy: PostDataType[] = [
    {
      key: '1234',
      id: '1234',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    },
    {
      key: '12345',
      id: '12345',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    },
    {
      key: '123456',
      id: '123456',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    },
    {
      key: '1234567',
      id: '1234567',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    },
    {
      key: '12345678',
      id: '12345678',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    },
    {
      key: '123456789',
      id: '123456789',
      placedTime: '12pm',
      orderSize: '100 USDC',
      filledSize: '10 USDC',
      tenor: '1 day',
      rate: '.99',
      autoRoll: true,
      cancel: false
    }
  ];

  interface FillDataType {
    key: string;
    id: string;
    startTime: string;
    matureTime: string;
    fillSize: string;
    quoteValue: string;
    status: string;
  }

  const fillOrderColumns = [
    {
      title: dictionary.fixedView.debtTable.fillOrder.id,
      dataIndex: 'id',
      key: 'id'
    },
    {
      title: dictionary.fixedView.debtTable.fillOrder.startTime,
      dataIndex: 'startTime',
      key: 'startTime'
    },
    {
      title: dictionary.fixedView.debtTable.fillOrder.matureTime,
      dataIndex: 'matureTime',
      key: 'matureTime'
    },
    {
      title: dictionary.fixedView.debtTable.fillOrder.fillSize,
      dataIndex: 'fillSize',
      key: 'fillSize'
    },
    {
      title: dictionary.fixedView.debtTable.fillOrder.quoteValue,
      dataIndex: 'quoteValue',
      key: 'quoteValue'
    },
    {
      title: dictionary.fixedView.debtTable.fillOrder.status,
      dataIndex: 'status',
      key: 'status'
    }
  ];

  const fillDataDummy = [
    {
      key: 'a123',
      id: 'a123',
      startTime: '12pm',
      matureTime: '6pm',
      fillSize: '11 USDC',
      quoteValue: '10 USDC',
      status: 'autoRoll'
    },
    {
      key: 'a1234',
      id: 'a1234',
      startTime: '12pm',
      matureTime: '6pm',
      fillSize: '11 USDC',
      quoteValue: '10 USDC',
      status: 'autoRoll'
    },
    {
      key: 'a12345',
      id: 'a12345',
      startTime: '12pm',
      matureTime: '6pm',
      fillSize: '11 USDC',
      quoteValue: '10 USDC',
      status: 'repaid'
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
    <div className="debt-detail account-table view-element flex-centered">
      <ConnectionFeedback />
      <Tabs
        defaultActiveKey={dictionary.fixedView.debtTable.title}
        items={[
          {
            label: dictionary.fixedView.debtTable.title,
            key: 'debtDetail',
            children: (
              // todo: add row onclick to expand the inner table
              <Table
                className={'debt-table'}
                columns={postOrderColumns}
                dataSource={postDataDummy}
                expandable={{
                  expandedRowRender: row => <Table columns={fillOrderColumns} dataSource={fillDataDummy} />,
                  expandIcon: ({ expanded, onExpand, record }) =>
                    expanded ? (
                      <DownOutlined onClick={e => onExpand(record, e)} />
                    ) : (
                      <RightOutlined onClick={e => onExpand(record, e)} />
                    )
                }}
                locale={{ emptyText: 'No Data' }}
                rowClassName={(_, index) => ((index + 1) % 2 === 0 ? 'dark-bg' : '')}
                pagination={{ pageSize }}
              />
            )
          }
        ]}
      />

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
        <div className="download-btns">
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
        <PrinterOutlined
          onClick={() => {
            setDownloadCsv(true);
            setTimeout(() => setDownloadCsv(false), 1000);
          }}
        />
        <SearchOutlined />
        {/* todo - fixme when data is ready*/}
        <Input
          type="text"
          placeholder={getFilterInputPlaceholder()}
          onChange={debounce(e => filterTxHistory(e.target.value), 300)}
        />
      </div>
      <ReorderArrows component="debtTable" order={accountsViewOrder} setOrder={setAccountsViewOrder} vertical />
    </div>
  );
}
