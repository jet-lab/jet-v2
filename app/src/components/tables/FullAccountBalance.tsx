import { useEffect, useRef, useState } from 'react';
import { CSVDownload } from 'react-csv';
import { useReactToPrint } from 'react-to-print';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { FiatCurrency } from '../../state/settings/settings';
import { AccountsViewOrder } from '../../state/views/views';
import { WalletModal } from '../../state/modals/modals';
import { WalletInit } from '../../state/user/walletTokens';
import { CurrentMarketPair } from '../../state/trade/market';
import { CurrentPoolSymbol, Pools } from '../../state/borrow/pools';
import { AccountBalance, AccountsInit, CurrentAccount, CurrentAccountHistory } from '../../state/user/accounts';
import { actionOptions, ActionRefresh, CurrentAction } from '../../state/actions/actions';
import { useCurrencyFormatting } from '../../utils/currency';
import { createDummyArray } from '../../utils/ui';
import { formatRate } from '../../utils/format';
import { Table, Skeleton, Typography, Input, Dropdown, Menu } from 'antd';
import { TokenLogo } from '../misc/TokenLogo';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Info } from '../misc/Info';
import { ConnectionFeedback } from '../misc/ConnectionFeedback';
import { DownloadOutlined, PrinterFilled, SearchOutlined } from '@ant-design/icons';
import { ReactComponent as AngleDown } from '../../styles/icons/arrow-angle-down.svg';
import { ActionIcon } from '../misc/ActionIcon';

export function FullAccountBalance(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const fiatCurrency = useRecoilValue(FiatCurrency);
  const [accountsViewOrder, setAccountsViewOrder] = useRecoilState(AccountsViewOrder);
  const [currentActionOption, setCurrentActionOption] = useState<PoolAction>('deposit');
  const setCurrentAction = useSetRecoilState(CurrentAction);
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const { connected } = useWallet();
  const walletInit = useRecoilValue(WalletInit);
  const pools = useRecoilValue(Pools);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const currentAccount = useRecoilValue(CurrentAccount);
  const currentAccountHistory = CurrentAccountHistory();
  const [currentAccountBalances, setCurrentAccountBalances] = useState<AccountBalance[] | undefined>(undefined);
  const [filteredAccountBalances, setFilteredAccountBalances] = useState(currentAccountBalances);
  const accountsInit = useRecoilValue(AccountsInit);
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [downloadCsv, setDownloadCsv] = useState(false);
  const accountBalancesRef = useRef<any>();
  const printBalancesPdf = useReactToPrint({
    content: () => accountBalancesRef.current
  });
  const { Paragraph, Text } = Typography;

  // Table data
  const balanceTableColumns = [
    {
      title: dictionary.common.token,
      key: 'tokenSymbol',
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
                pools.tokenPools[balance.tokenSymbol].tokenPrice ?? 0,
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
      title: dictionary.accountsView.deposits,
      key: 'deposits',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          `${currencyAbbrev(balance.depositBalance.tokens, false, undefined, balance.depositBalance.decimals)} ${
            balance.tokenSymbol
          }`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.borrows,
      key: 'borrows',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          `${currencyAbbrev(balance.loanBalance.tokens, false, undefined, balance.loanBalance.decimals)} ${
            balance.tokenSymbol
          }`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.netBalance,
      key: 'netBalance',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          `${currencyAbbrev(balance.overallBalance.tokens, false, undefined, balance.overallBalance.decimals)} ${
            balance.tokenSymbol
          }`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.inOrders,
      key: 'inOrders',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.tokenSymbol ? (
          `${currencyAbbrev(balance.inOrders[balance.tokenSymbol] ?? 0)} ${balance.tokenSymbol}`
        ) : (
          <Skeleton className="align-right" paragraph={false} active={walletInit && !accountsInit} />
        )
    },
    {
      title: dictionary.accountsView.fiatValue.replace('{{FIAT_CURRENCY}}', fiatCurrency),
      key: 'fiatValue',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) =>
        accountsInit && balance?.fiatValue ? (
          balance.fiatValue
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
      key: 'percent',
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
    },
    {
      title: '',
      key: 'action',
      align: 'right' as any,
      render: (value: any, balance: AccountBalance) => (
        <Dropdown.Button
          onClick={() => {
            if (connected) {
              setCurrentAction(currentActionOption);
              setCurrentPoolSymbol(balance.tokenSymbol);
              if (balance.tokenSymbol !== 'USDC') {
                setCurrentMarketPair(`${balance.tokenSymbol}/USDC`);
              }
            } else {
              setWalletModalOpen(true);
            }
          }}
          icon={<AngleDown className="jet-icon" />}
          overlay={
            <Menu className="min-width-menu">
              {actionOptions.map(option => (
                <Menu.Item
                  key={option}
                  onClick={() => setCurrentActionOption(option)}
                  className={option === currentActionOption ? 'active' : ''}>
                  <ActionIcon action={option} style={{ width: 20, height: 20 }} />
                  {dictionary.actions[option].title.toUpperCase()}
                </Menu.Item>
              ))}
            </Menu>
          }>
          <ActionIcon action={currentActionOption} style={{ width: 20, height: 20 }} />
          {dictionary.actions[currentActionOption].title.toUpperCase()}
        </Dropdown.Button>
      )
    }
  ];

  // Create array of account balances for current account
  useEffect(() => {
    if (currentAccount && pools) {
      const accountBalances: AccountBalance[] = [];
      for (const token of Object.values(pools.tokenPools)) {
        if (!token.symbol) {
          return;
        }

        const poolPosition = currentAccount.poolPositions[token.symbol];
        if (poolPosition) {
          const overallBalance = poolPosition.depositBalance.sub(poolPosition.loanBalance);
          const fiatValue = currencyAbbrev(poolPosition.depositValue - poolPosition.loanValue, true);
          const percent =
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
            overallBalance,
            inOrders,
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
  }, [pools, accountsInit, actionRefresh]);

  return (
    <div className="full-account-balance account-table view-element view-element-hidden flex-centered">
      <ConnectionFeedback />
      <div className="full-account-balance-header view-element-item view-element-item-hidden flex align-center justify-between">
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
            <PrinterFilled onClick={() => (accountsInit && filteredAccountBalances ? printBalancesPdf() : null)} />
          </div>
          <SearchOutlined />
          <Input
            type="text"
            placeholder={dictionary.accountsView.balancesFilterPlaceholder}
            onChange={e => {
              const query = e.target.value.toLowerCase();
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
            }}
          />
        </div>
      </div>
      <Table
        ref={accountBalancesRef}
        dataSource={
          filteredAccountBalances ?? createDummyArray(Object.keys(pools?.tokenPools ?? {}).length, 'signature')
        }
        columns={balanceTableColumns}
        className="no-row-interaction balance-table  view-element-item view-element-item-hidden"
        rowKey={row => `${row.tokenSymbol}-${Math.random()}`}
        locale={{ emptyText: dictionary.accountsView.noBalances }}
        pagination={false}
      />
      <ReorderArrows
        component="fullAccountBalance"
        order={accountsViewOrder}
        setOrder={setAccountsViewOrder}
        vertical
      />
    </div>
  );
}
