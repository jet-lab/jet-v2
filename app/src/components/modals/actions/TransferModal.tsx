import { useEffect, useState } from 'react';
import { useSetRecoilState, useResetRecoilState, useRecoilValue, useRecoilState } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { SendingTransaction } from '../../../state/actions/actions';
import { BlockExplorer, Cluster } from '../../../state/settings/settings';
import { AccountNames, Accounts, useAccountFromName } from '../../../state/user/accounts';
import { CurrentPool } from '../../../state/pools/pools';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { ActionResponse } from '../../../utils/jet/marginActions';
import { useTokenInputDisabledMessage } from '../../../utils/actions/tokenInput';
import { DEFAULT_DECIMALS, useCurrencyFormatting } from '../../../utils/currency';
import { formatRiskIndicator } from '../../../utils/format';
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { useProjectedRisk, useRiskStyle } from '../../../utils/risk';
import { useMarginActions } from '../../../utils/jet/marginActions';
import { Button, Divider, Modal, Select, Typography } from 'antd';
import { TokenInput } from '../../misc/TokenInput/TokenInput';
import AngleDown from '../../../styles/icons/arrow-angle-down.svg';
import ArrowDown from '../../../styles/icons/arrow-down.svg';
import { ArrowRight } from './ArrowRight';

// Modal to transfer collateral from one marginAccount to another
export function TransferModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { transfer } = useMarginActions();
  const accounts = useRecoilValue(Accounts);
  const currentPool = useRecoilValue(CurrentPool);
  const precision = currentPool?.precision ?? DEFAULT_DECIMALS;
  const currentAction = useRecoilValue(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const accountNames = useRecoilValue(AccountNames);
  const [fromAccountOptions, setFromAccountOptions] = useState<string[]>([]);
  const [fromAccountName, setFromAccountName] = useState<string | undefined>(undefined);
  const fromAccount = useAccountFromName(fromAccountName);
  const fromAccountPoolPosition =
    fromAccount && currentPool ? fromAccount.poolPositions[currentPool.symbol] : undefined;
  const [toAccountOptions, setToAccountOptions] = useState<string[]>([]);
  const [toAccountName, setToAccountName] = useState<string | undefined>(undefined);
  const toAccount = useAccountFromName(toAccountName);
  const toAccountPoolPosition = currentPool && toAccount ? toAccount.poolPositions[currentPool.symbol] : undefined;
  const fromAdjustedRiskIndicator = useProjectedRisk(currentPool, fromAccount, 'withdraw');
  const fromAdjustedRiskStyle = useRiskStyle(fromAdjustedRiskIndicator);
  const fromRiskStyle = useRiskStyle(fromAccount?.riskIndicator);
  const toAdjustedRiskIndicator = useProjectedRisk(currentPool, toAccount, 'deposit');
  const toAdjustedRiskStyle = useRiskStyle(toAdjustedRiskIndicator);
  const toRiskStyle = useRiskStyle(toAccount?.riskIndicator);
  const disabledMessage = useTokenInputDisabledMessage(fromAccount);
  const setTokenInputString = useSetRecoilState(TokenInputString);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const disabled =
    sendingTransaction || disabledMessage.length > 0 || tokenInputAmount.isZero() || !(fromAccount && toAccount);
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

  // Keep account select options updated with latest choices
  useEffect(() => {
    const fromAccountOptions: string[] = [];
    const toAccountOptions: string[] = [];
    for (const account of accounts) {
      if (account.address.toString() !== fromAccount?.address.toString()) {
        toAccountOptions.push(accountNames[account.address.toString()]);
      }
      if (account.address.toString() !== toAccount?.address.toString()) {
        fromAccountOptions.push(accountNames[account.address.toString()]);
      }
    }
    setFromAccountOptions(fromAccountOptions);
    setToAccountOptions(toAccountOptions);
  }, [accounts, fromAccount, toAccount, accountNames, fromAccountName, toAccountName]);

  // Renders the affected balance of either the from or to transfer account
  function renderAffectedBalance(side: 'from' | 'to') {
    let render = <></>;
    const poolPosition = side === 'from' ? fromAccountPoolPosition : toAccountPoolPosition;
    if (poolPosition && !tokenInputAmount.isZero()) {
      const balanceText = fromAccountPoolPosition
        ? currencyAbbrev(poolPosition.depositBalance.tokens - tokenInputAmount.tokens, false, undefined, precision)
        : '—';
      render = (
        <div className="flex-centered">
          <ArrowRight />
          {balanceText}
        </div>
      );
    }

    return render;
  }

  // Renders the adjusted risk level if user were to transfer
  function renderAdjustedRisk(side: 'from' | 'to') {
    let render = <></>;
    if (!tokenInputAmount.isZero()) {
      const riskIndicator = side === 'from' ? fromAdjustedRiskIndicator : toAdjustedRiskIndicator;
      const riskStyle = side === 'from' ? fromAdjustedRiskStyle : toAdjustedRiskStyle;
      render = (
        <Paragraph type={riskStyle}>
          <ArrowRight />
          {formatRiskIndicator(riskIndicator)}
        </Paragraph>
      );
    }

    return render;
  }

  if (currentAction === 'transfer') {
    return (
      <Modal
        visible
        className="action-modal transfer-modal header-modal"
        maskClosable={false}
        footer={null}
        onCancel={() => {
          if (!sendingTransaction) {
            resetCurrentAction();
            resetTokenInputString();
            resetTokenInputAmount();
          }
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.actions.transfer.title}</Title>
        </div>
        <div className="transfer-accounts-select flex align-start justify-center column">
          <Text className="small-accent-text">{dictionary.actions.transfer.from}</Text>
          <Select
            value={fromAccountName}
            className="dropdown-space-between"
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={option => setFromAccountName(option)}>
            {fromAccountOptions.map(account => (
              <Option key={`from${account}`} value={account}>
                {account}
              </Option>
            ))}
            {fromAccount && (
              <Option key="fromWallet" value="">
                —
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
          <Text className="small-accent-text">{fromAccountName}</Text>
          {fromAccountPoolPosition && currentPool?.symbol && (
            <Paragraph
              onClick={() => setTokenInputString(fromAccountPoolPosition.depositBalance.tokens.toString())}
              className="token-balance">{`${fromAccountPoolPosition.depositBalance.uiTokens} ${currentPool.symbol}`}</Paragraph>
          )}
        </div>
        <TokenInput account={fromAccount} hideSlider onPressEnter={sendTransfer} />
        <Divider />
        <div className="from-account flex column">
          <Paragraph className="from-account-text">{fromAccountName ?? '—'}</Paragraph>
          <div className="action-info from-account-info flex align-between justify-start column">
            <div className="action-info-item flex align-center justify-between">
              <Paragraph className="from-account-text">{dictionary.common.balance}</Paragraph>
              <Paragraph className="from-account-text">
                {fromAccountPoolPosition
                  ? currencyAbbrev(fromAccountPoolPosition.depositBalance.tokens, false, undefined, precision)
                  : '—'}
                {renderAffectedBalance('from')}
                {' ' + currentPool?.symbol}
              </Paragraph>
            </div>
            <div className="action-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
              <div className="flex-centered">
                <Paragraph type={fromAccount ? fromRiskStyle : 'secondary'}>
                  {fromAccount ? formatRiskIndicator(fromAccount.riskIndicator) : '—'}
                </Paragraph>
                {renderAdjustedRisk('from')}
              </div>
            </div>
          </div>
        </div>
        <div className="flex-centered">
          <ArrowDown className="jet-icon arrow-down" />
        </div>
        <div className="to-account flex column">
          <Paragraph className="to-account-text">{toAccountName ?? '—'}</Paragraph>
          <div className="action-info to-account-info flex align-between justify-start column">
            <div className="action-info-item flex align-center justify-between">
              <Paragraph className="to-account-text">{dictionary.common.balance}</Paragraph>
              <Paragraph className="to-account-text">
                {toAccountPoolPosition ? currencyAbbrev(toAccountPoolPosition.depositBalance.tokens) : '—'}
                {renderAffectedBalance('to')}
                {' ' + currentPool?.symbol}
              </Paragraph>
            </div>
            <div className="action-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
              <div className="flex-centered">
                <Paragraph type={toAccount ? toRiskStyle : 'secondary'}>
                  {toAccount ? formatRiskIndicator(toAccount.riskIndicator) : '—'}
                </Paragraph>
                {renderAdjustedRisk('to')}
              </div>
            </div>
          </div>
        </div>
        <Button block disabled={disabled} loading={sendingTransaction} onClick={sendTransfer}>
          {sendingTransaction ? dictionary.common.sending + '..' : dictionary.actions[currentAction ?? 'deposit'].title}
        </Button>
        <div className={`action-modal-overlay ${sendingTransaction ? 'showing' : ''}`}></div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
