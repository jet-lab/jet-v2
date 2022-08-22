import { useEffect, useState } from 'react';
import { useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { BlockExplorer, Cluster } from '../../../state/settings/settings';
import { WalletTokens } from '../../../state/user/walletTokens';
import { AccountNames, Accounts, useAccountFromName, CurrentAccount } from '../../../state/user/accounts';
import { CurrentMarketPair } from '../../../state/trade/market';
import { CurrentPoolSymbol, CurrentPool } from '../../../state/borrow/pools';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { ActionResponse } from '../../../utils/jet/marginActions';
import { useTokenInputDisabledMessage } from '../../../utils/actions/tokenInput';
import { useCurrencyFormatting } from '../../../utils/currency';
import { formatRiskIndicator } from '../../../utils/format';
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { useRiskStyle } from '../../../utils/risk';
import { useMarginActions } from '../../../utils/jet/marginActions';
import { Button, Divider, Modal, Select, Typography } from 'antd';
import { TokenInput } from '../../misc/TokenInput';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';
import { ReactComponent as ArrowDown } from '../../../styles/icons/arrow-down.svg';
import { getProjectionAfterAction } from '../../../utils/actions/projection';

export function TransferModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { transfer } = useMarginActions();
  const walletTokens = useRecoilValue(WalletTokens);
  const marginAccounts = useRecoilValue(Accounts);
  const currentPool = useRecoilValue(CurrentPool);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const currentAction = useRecoilValue(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const currentAccount = useRecoilValue(CurrentAccount);
  const accountNames = useRecoilValue(AccountNames);
  const [fromAccountOptions, setFromAccountOptions] = useState<string[]>([]);
  const [fromAccountName, setFromAccountName] = useState<string | undefined>(undefined);
  const fromAccount = useAccountFromName(fromAccountName);
  const fromAccountPoolPosition = currentPool?.symbol && fromAccount?.poolPositions[currentPool.symbol];
  const [toAccountOptions, setToAccountOptions] = useState<string[]>([]);
  const [toAccountName, setToAccountName] = useState<string | undefined>(undefined);
  const toAccount = useAccountFromName(toAccountName);
  const toAccountPoolPosition = currentPool?.symbol && toAccount?.poolPositions[currentPool.symbol];
  const fromAdjustedRiskIndicator =
    currentPool && fromAccount && currentAction && !tokenInputAmount.isZero()
      ? getProjectionAfterAction(currentPool, currentAction, fromAccount, tokenInputAmount)
      : fromAccount?.riskIndicator ?? 0;
  const fromAdjustedRiskStyle = useRiskStyle(fromAdjustedRiskIndicator);
  const fromRiskStyle = useRiskStyle(fromAccount?.riskIndicator);
  const toAdjustedRiskIndicator =
    currentPool && toAccount && currentAction && !tokenInputAmount.isZero()
      ? currentPool.projectAfterDeposit(toAccount, tokenInputAmount.tokens).riskIndicator
      : toAccount?.riskIndicator ?? 0;
  const toAdjustedRiskStyle = useRiskStyle(toAdjustedRiskIndicator);
  const toRiskStyle = useRiskStyle(toAccount?.riskIndicator);
  const disabledMessage = useTokenInputDisabledMessage(fromAccount);
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Paragraph, Text } = Typography;
  const { Option } = Select;

  // Transfer
  async function sendTransfer() {
    if (!fromAccount || !toAccount) {
      return;
    }

    setSendingTransaction(true);
    const [txId, resp] = await transfer(fromAccount, toAccount);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replaceAll('{{ACTION}}', currentAction ?? ''),
        dictionary.notifications.actions.successDescription
          .replaceAll('{{ACTION}}', currentAction ?? '')
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
      resetTokenInputString();
      resetCurrentAction();
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.actions.cancelledTitle.replaceAll('{{ACTION}}', currentAction ?? ''),
        dictionary.notifications.actions.cancelledDescription
          .replaceAll('{{ACTION}}', currentAction ?? '')
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.actions.failedTitle.replaceAll('{{ACTION}}', currentAction ?? ''),
        dictionary.notifications.actions.failedDescription
          .replaceAll('{{ACTION}}', currentAction ?? '')
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'error'
      );
    }
    setSendingTransaction(false);
  }

  // Set fromAccount to currentAccount on init
  useEffect(() => {
    for (const accountKey of Object.keys(accountNames)) {
      if (accountKey === currentAccount?.address.toString()) {
        setFromAccountName(accountNames[accountKey]);
      }
    }
  }, [accountNames, currentAccount]);

  // Keep account select options updated with latest choices
  useEffect(() => {
    const fromAccountOptions: string[] = [];
    const toAccountOptions: string[] = [];
    for (const account of marginAccounts) {
      if (account.address.toString() !== fromAccount?.address.toString()) {
        toAccountOptions.push(accountNames[account.address.toString()]);
      }
      if (account.address.toString() !== toAccount?.address.toString()) {
        fromAccountOptions.push(accountNames[account.address.toString()]);
      }
    }
    setFromAccountOptions(fromAccountOptions);
    setToAccountOptions(toAccountOptions);
  }, [marginAccounts, fromAccount, toAccount, accountNames, fromAccountName, toAccountName]);

  if (currentAction === 'transfer') {
    return (
      <Modal
        visible
        className="action-modal transfer-modal header-modal"
        footer={null}
        onCancel={() => {
          resetCurrentAction();
          resetTokenInputString();
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.actions.transfer.title}</Title>
        </div>
        <div className="transfer-accounts-select flex align-start justify-center column">
          <Text className="small-accent-text">{dictionary.actions.transfer.from}</Text>
          <Select
            value={fromAccountName}
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={option => setFromAccountName(option)}>
            {fromAccountOptions.map(account => (
              <Option key={`from${account}`} value={account}>
                {account}
              </Option>
            ))}
            {fromAccount && (
              <Option key="fromWallet" value="">
                {''}
              </Option>
            )}
          </Select>
          <Text className="small-accent-text">{dictionary.actions.transfer.to}</Text>
          <Select
            value={toAccountName}
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={option => setToAccountName(option)}>
            {toAccountOptions.map(account => (
              <Option key={`to${account}`} value={account}>
                {account}
              </Option>
            ))}
            {toAccount && (
              <Option key="toWallet" value="">
                {''}
              </Option>
            )}
          </Select>
        </div>
        <Divider />
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {walletTokens && currentPool?.symbol && (
            <Paragraph type="secondary" italic>{`${walletTokens.map[currentPool.symbol].amount.tokens} ${
              currentPool.symbol
            }`}</Paragraph>
          )}
        </div>
        <TokenInput
          account={fromAccount}
          onChangeToken={(tokenSymbol: string) => {
            setCurrentPoolSymbol(tokenSymbol);
            if (tokenSymbol !== 'USDC') {
              setCurrentMarketPair(`${tokenSymbol}/USDC`);
            }
          }}
          loading={sendingTransaction}
          onPressEnter={sendTransfer}
        />
        <Divider />
        <div className="from-account flex column">
          <Paragraph className="from-account-text">{fromAccountName ?? '—'}</Paragraph>
          <div className="action-info from-account-info flex align-between justify-center column">
            <div className="action-info-item flex align-center justify-between">
              <Paragraph className="from-account-text">{dictionary.common.accountBalance}</Paragraph>
              <Paragraph className="from-account-text">
                {fromAccountPoolPosition
                  ? currencyAbbrev(
                      fromAccountPoolPosition.depositBalance.tokens,
                      false,
                      undefined,
                      currentPool?.decimals
                    )
                  : '—'}
                {!tokenInputAmount.isZero() && (
                  <>
                    &nbsp;&#8594;&nbsp;
                    {fromAccountPoolPosition
                      ? currencyAbbrev(
                          fromAccountPoolPosition.depositBalance.tokens - tokenInputAmount.tokens,
                          false,
                          undefined,
                          currentPool?.decimals
                        )
                      : '—'}
                  </>
                )}
                {' ' + currentPool?.symbol}
              </Paragraph>
            </div>
            <div className="action-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
              <div className="flex-centered">
                <Paragraph type={fromAccount ? fromRiskStyle : 'secondary'}>
                  {fromAccount ? formatRiskIndicator(fromAccount.riskIndicator) : '—'}
                </Paragraph>
                {!tokenInputAmount.isZero() && (
                  <Paragraph type={fromAdjustedRiskStyle}>
                    &nbsp;&#8594;&nbsp;{formatRiskIndicator(fromAdjustedRiskIndicator)}
                  </Paragraph>
                )}
              </div>
            </div>
          </div>
        </div>
        <div className="flex-centered">
          <ArrowDown className="jet-icon arrow-down" />
        </div>
        <div className="to-account flex column">
          <Paragraph className="to-account-text">{toAccountName ?? '—'}</Paragraph>
          <div className="action-info to-account-info flex align-between justify-center column">
            <div className="action-info-item flex align-center justify-between">
              <Paragraph className="to-account-text">{dictionary.common.accountBalance}</Paragraph>
              <Paragraph className="to-account-text">
                {toAccountPoolPosition ? currencyAbbrev(toAccountPoolPosition.depositBalance.tokens) : '—'}
                {!tokenInputAmount.isZero() && (
                  <>
                    &nbsp;&#8594;&nbsp;
                    {toAccountPoolPosition
                      ? currencyAbbrev(toAccountPoolPosition.depositBalance.tokens + tokenInputAmount.tokens)
                      : '—'}
                  </>
                )}
                {' ' + currentPool?.symbol}
              </Paragraph>
            </div>
            <div className="action-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
              <div className="flex-centered">
                <Paragraph type={toAccount ? toRiskStyle : 'secondary'}>
                  {toAccount ? formatRiskIndicator(toAccount.riskIndicator) : '—'}
                </Paragraph>
                {!tokenInputAmount.isZero() && (
                  <Paragraph type={toAdjustedRiskStyle}>
                    &nbsp;&#8594;&nbsp;{formatRiskIndicator(toAdjustedRiskIndicator)}
                  </Paragraph>
                )}
              </div>
            </div>
          </div>
        </div>
        <Button
          block
          disabled={
            sendingTransaction || disabledMessage.length < 0 || tokenInputAmount.isZero() || !(fromAccount && toAccount)
          }
          loading={sendingTransaction}
          onClick={sendTransfer}>
          {sendingTransaction ? dictionary.common.sending + '..' : dictionary.actions[currentAction ?? 'deposit'].title}
        </Button>
      </Modal>
    );
  } else {
    return <></>;
  }
}
