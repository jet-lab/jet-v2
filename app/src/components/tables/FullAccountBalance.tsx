import { useEffect, useRef, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { CSVDownload } from 'react-csv';
import { SetterOrUpdater, useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { FiatCurrency } from '../../state/settings/settings';
import { AccountsViewOrder, SwapsViewOrder } from '../../state/views/views';
import { WalletTokens } from '../../state/user/walletTokens';
import { CurrentPoolSymbol, Pools } from '../../state/pools/pools';
import { AccountBalance, Accounts, CurrentAccount } from '../../state/user/accounts';
import { ActionRefresh, CurrentSwapOutput } from '../../state/actions/actions';
import { useCurrencyFormatting } from '../../utils/currency';
import { createDummyArray } from '../../utils/ui';
import { formatRate } from '../../utils/format';
import { Table, Skeleton, Typography, Input } from 'antd';
import { TokenLogo } from '../misc/TokenLogo';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Info } from '../misc/Info';
import { ConnectionFeedback } from '../misc/ConnectionFeedback/ConnectionFeedback';
import { DownloadOutlined, SearchOutlined } from '@ant-design/icons';

// Table to show margin account's balances for each token
export function FullAccountBalance(): JSX.Element {
  const { pathname } = useLocation();
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const fiatCurrency = useRecoilValue(FiatCurrency);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const [swapsViewOrder, setSwapsViewOrder] = useRecoilState(SwapsViewOrder);
  const walletTokens = useRecoilValue(WalletTokens);
  const currentPoolSymbol = useRecoilValue(CurrentPoolSymbol);
  const pools = useRecoilValue(Pools);
  const currentSwapOutput = useRecoilValue(CurrentSwapOutput);
  const currentAccount = useRecoilValue(CurrentAccount);
  const [currentAccountBalances, setCurrentAccountBalances] = useState<AccountBalance[] | undefined>(undefined);
  const [filteredAccountBalances, setFilteredAccountBalances] = useState(currentAccountBalances);
  const accounts = useRecoilValue(Accounts);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const accountBalancesRef = useRef<any>();
  const loadingAccounts = walletTokens && !accounts.length;
  const { Paragraph, Text } = Typography;

  // Determine which component ordering state/reordering method to utilize
  function getOrderContext(): {
    order: string[];
    setOrder: SetterOrUpdater<string[]>;
  } {
    switch (pathname) {
      case '/swaps':
        return {
          order: swapsViewOrder,
          setOrder: setSwapsViewOrder
        };
      default:
        return {
          order: accountsViewOrder,
          setOrder: setAccountsViewOrder
        };
    }
  }

  // Tell if token is a swap token, and highlight in table
  function isSwapToken(symbol: string) {
    if (pathname === '/swaps' && (currentPoolSymbol === symbol || currentSwapOutput?.symbol === symbol)) {
      return true;
    }
  }

  // Renders the tokens price info
  function renderTokenPrice(balance: AccountBalance) {
    let render = <Skeleton paragraph={false} active={loadingAccounts} />;
    if (accounts && pools) {
      render = (
        <div className="flex-centered">
          <Text className="price-name">{`${balance.tokenSymbol} ≈ ${currencyFormatter(
            pools.tokenPools[balance.tokenSymbol]?.tokenPrice ?? 0,
            true,
            balance.tokenSymbol === 'USDC' ? 0 : undefined,
            balance.tokenSymbol === 'USDC'
          )}`}</Text>
          <Text className="price-abbrev">{`≈ ${currencyFormatter(
            pools.tokenPools[balance.tokenSymbol]?.tokenPrice ?? 0,
            true,
            balance.tokenSymbol === 'USDC' ? 0 : undefined,
            balance.tokenSymbol === 'USDC'
          )}`}</Text>
        </div>
      );
    }

    return render;
  }

  // Renders the token column for table
  function renderTokenColumn(balance: AccountBalance) {
    let render = <Skeleton avatar paragraph={false} active={loadingAccounts} />;
    if (accounts && balance?.tokenSymbol) {
      render = (
        <div className="table-token">
          <TokenLogo height={22} symbol={balance.tokenSymbol} />
          <Text className="table-token-name" strong>
            {balance.tokenName}
          </Text>
          <Text className="table-token-abbrev" strong>
            {balance.tokenSymbol}
          </Text>
          {renderTokenPrice(balance)}
        </div>
      );
    }

    return render;
  }

  // Renders the deposit balance column for table
  function renderDepositBalanceColumn(balance: AccountBalance) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && balance?.tokenSymbol) {
      render = (
        <Text type={balance.depositBalance.isZero() ? undefined : 'success'}>
          {currencyAbbrev(balance.depositBalance.tokens, false, undefined, balance.depositBalance.decimals / 2)}
        </Text>
      );
    }

    return render;
  }

  // Renders the loan balance column for table
  function renderLoanBalanceColumn(balance: AccountBalance) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && balance?.tokenSymbol) {
      render = (
        <Text type={balance.loanBalance.isZero() ? undefined : 'warning'}>
          {currencyAbbrev(balance.loanBalance.tokens, false, undefined, balance.loanBalance.decimals / 2)}
        </Text>
      );
    }

    return render;
  }

  // Renders the fiat value column for table
  function renderFiatValue(balance: AccountBalance) {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && balance?.tokenSymbol) {
      render = <Text>{balance.fiatValue}</Text>;
    }

    return render;
  }

  // Renders the pool rates columns for table
  function renderPoolRate(balance: AccountBalance, side: 'deposit' | 'borrow') {
    let render = <Skeleton className="align-right" paragraph={false} active={loadingAccounts} />;
    if (accounts && balance?.tokenSymbol) {
      render = (
        <Text type={side === 'borrow' ? 'danger' : 'success'}>
          {formatRate(balance[side === 'borrow' ? 'borrowRate' : 'depositRate'])}
        </Text>
      );
    }

    return render;
  }

  // Table data
  const balanceTableColumns = [
    {
      title: dictionary.common.token,
      key: 'tokenSymbol',
      align: 'left' as any,
      width: 175,
      render: (value: any, balance: AccountBalance) => renderTokenColumn(balance)
    },
    {
      title: dictionary.accountsView.deposits,
      key: 'deposits',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => renderDepositBalanceColumn(balance)
    },
    {
      title: dictionary.accountsView.borrows,
      key: 'borrows',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => renderLoanBalanceColumn(balance)
    },
    {
      title: dictionary.accountsView.fiatValue.replace('{{FIAT_CURRENCY}}', fiatCurrency),
      key: 'fiatValue',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => renderFiatValue(balance)
    },
    {
      title: (
        <Info term="depositRate">
          <Text className="info-element">{dictionary.accountsView.depositRate}</Text>
        </Info>
      ),
      key: 'depositRate',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => renderPoolRate(balance, 'deposit')
    },
    {
      title: (
        <Info term="borrowRate">
          <Text className="info-element">{dictionary.accountsView.borrowRate}</Text>
        </Info>
      ),
      key: 'borrowRate',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => renderPoolRate(balance, 'borrow')
    }
  ];

  // Filters the balance table from a query
  function filterBalanceTable(queryString: string) {
    const query = queryString.toLowerCase();
    if (currentAccountBalances) {
      const filteredAccountBalances: AccountBalance[] = [];
      for (const balance of currentAccountBalances) {
        if (balance.tokenName.toLowerCase().includes(query) || balance.tokenSymbol.toLowerCase().includes(query)) {
          filteredAccountBalances.push(balance);
        }
      }
      setFilteredAccountBalances(filteredAccountBalances);
    }
  }

  // Create array of account balances for current account
  useEffect(() => {
    if (currentAccount && pools) {
      const accountBalances: AccountBalance[] = [];
      for (const token of Object.values(pools.tokenPools)) {
        const poolPosition = currentAccount.poolPositions[token.symbol];
        if (poolPosition) {
          const netBalance = poolPosition.depositBalance.sub(poolPosition.loanBalance);
          const fiatValue = currencyAbbrev(poolPosition.depositValue - poolPosition.loanValue, true);
          const percent =
            (poolPosition.depositBalance.tokens + poolPosition.loanBalance.tokens) /
            (currentAccount.summary.depositedValue + currentAccount.summary.borrowedValue);

          accountBalances.push({
            tokenName: token.name ?? dictionary.common.notAvailable,
            tokenSymbol: token.symbol,
            depositBalance: poolPosition.depositBalance,
            loanBalance: poolPosition.loanBalance,
            netBalance,
            fiatValue,
            percentageOfPortfolio: !isNaN(percent) ? percent : 0,
            depositRate: token.depositApy,
            borrowRate: token.borrowApr
          });
        }
      }
      setCurrentAccountBalances(accountBalances);
      setFilteredAccountBalances(accountBalances);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pools, accounts, actionRefresh]);

  return (
    <div className="full-account-balance account-table view-element flex-centered">
      <ConnectionFeedback />
      <div className="full-account-balance-header flex align-center justify-between">
        <Paragraph strong>{dictionary.accountsView.allBalances}</Paragraph>
        <div className="account-table-search">
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
          </div>
          <SearchOutlined />
          <Input
            type="text"
            placeholder={dictionary.accountsView.balancesFilterPlaceholder}
            onChange={e => filterBalanceTable(e.target.value)}
          />
        </div>
      </div>
      <Table
        ref={accountBalancesRef}
        dataSource={
          filteredAccountBalances ?? createDummyArray(Object.keys(pools?.tokenPools ?? {}).length, 'signature')
        }
        columns={balanceTableColumns}
        className="no-row-interaction balance-table "
        rowKey={row => `${row.tokenSymbol}-${Math.random()}`}
        rowClassName={row => (isSwapToken(row.tokenSymbol) ? 'dark-bg' : '')}
        locale={{ emptyText: dictionary.accountsView.noBalances }}
        pagination={false}
      />
      <ReorderArrows
        component="fullAccountBalance"
        order={getOrderContext().order}
        setOrder={getOrderContext().setOrder}
        vertical
      />
    </div>
  );
}
