import { useRecoilState, useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { SendingTransaction } from '@state/actions/actions';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { CurrentAccount } from '@state/user/accounts';
import { CurrentPool } from '@state/pools/pools';
import { CurrentAction, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { useTokenInputDisabledMessage } from '@utils/actions/tokenInput';
import { useCurrencyFormatting } from '@utils/currency';
import { formatRiskIndicator, formatRate } from '@utils/format';
import { useMarginActions, ActionResponse } from '@utils/jet/marginActions';
import { getExplorerUrl, getTokenStyleType } from '@utils/ui';
import { notify } from '@utils/notify';
import { useProjectedRisk, useRiskStyle } from '@utils/risk';
import { ArrowRight } from './ArrowRight';
import { Button, Modal, Tabs, Typography } from 'antd';
import { TokenInput } from '@components/misc/TokenInput/TokenInput';

// Modal to Borrow / Repay using the current Pool
export function BorrowRepayModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { borrow, repay } = useMarginActions();
  const currentPool = useRecoilValue(CurrentPool);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const currentAccount = useRecoilValue(CurrentAccount);
  const accountPoolPosition = currentPool ? currentAccount?.poolPositions[currentPool.symbol] : undefined;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const setTokenInputString = useSetRecoilState(TokenInputString);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const riskStyle = useRiskStyle();
  const projectedRiskIndicator = useProjectedRisk();
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const disabledMessage = useTokenInputDisabledMessage();
  const disabled = sendingTransaction || disabledMessage.length > 0;
  const { Paragraph, Text } = Typography;
  const tabItems = ['borrow', 'repay'].map((action: string) => {
    return {
      label: action,
      key: action
    };
  });

  function displayRepayFromDepositAsRepay() {
    return !currentAction ? '' : currentAction === 'repayFromDeposit' ? 'repay' : currentAction;
  }

  // Borrow / Repay
  async function borrowRepay() {
    setSendingTransaction(true);
    const [txId, resp] = currentAction === 'borrow' ? await borrow() : await repay(true);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay()),
        dictionary.notifications.actions.successDescription
          .replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay())
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
      resetTokenInputString();
      resetCurrentAction();
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.actions.cancelledTitle.replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay()),
        dictionary.notifications.actions.cancelledDescription
          .replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay())
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.actions.failedTitle.replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay()),
        dictionary.notifications.actions.failedDescription
          .replaceAll('{{ACTION}}', displayRepayFromDepositAsRepay())
          .replaceAll('{{ASSET}}', currentPool?.symbol ?? '')
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'error',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
    }
    setSendingTransaction(false);
  }

  // Returns the margin account's loan balance
  function getLoanBalance(balance?: number) {
    let loanBalance = 0;
    if (accountPoolPosition) {
      loanBalance = accountPoolPosition.loanBalance.tokens;
    }

    const decimals = currentPool?.precision ?? 2;

    const abbreviatedLoanBalance = currencyAbbrev(balance ?? loanBalance, decimals, false, undefined);
    return abbreviatedLoanBalance;
  }

  // Renders account's affected loan balance
  function renderAffectedLoanBalance() {
    let render = <></>;
    if (!tokenInputAmount.isZero() && accountPoolPosition) {
      const newBalance =
        currentAction === 'borrow'
          ? accountPoolPosition.loanBalance.tokens + tokenInputAmount.tokens
          : accountPoolPosition.loanBalance.tokens - tokenInputAmount.tokens;
      const affectedBalance = getLoanBalance(newBalance);

      render = (
        <>
          <ArrowRight />
          <Paragraph type={getTokenStyleType(affectedBalance, true)}>{affectedBalance}</Paragraph>
        </>
      );
    }

    return render;
  }

  // Renders pool's affected borrow rate
  function renderAffectedBorrowRate() {
    let render = <></>;
    if (!tokenInputAmount.isZero() && currentPool && currentAccount && currentAction) {
      const newRate = currentPool.projectAfterAction(currentAccount, tokenInputAmount.tokens, currentAction).borrowRate;
      const affectedRate = formatRate(newRate);

      render = (
        <>
          <ArrowRight />
          {affectedRate}
        </>
      );
    }

    return render;
  }

  // Renders account's affected borrow rate
  function renderAffectedRiskLevel() {
    let render = <></>;
    if (!tokenInputAmount.isZero()) {
      const affectedRiskLevel = formatRiskIndicator(projectedRiskIndicator);

      render = (
        <>
          <ArrowRight />
          <Paragraph type={projectedRiskStyle}>{affectedRiskLevel}</Paragraph>
        </>
      );
    }

    return render;
  }

  // Renders the account balance for current pool token
  function renderAccountBalance() {
    let render = <></>;
    if (currentAccount && currentPool) {
      const accountBalance = currentAccount.poolPositions[currentPool.symbol].depositBalance.uiTokens;
      render = (
        <Paragraph
          onClick={() => (!disabled && currentAction === 'repay' ? setTokenInputString(accountBalance) : null)}
          className={!disabled && currentAction === 'repay' ? 'token-balance' : 'secondary-text'}>
          {accountBalance + ' ' + currentPool.symbol}
        </Paragraph>
      );
    }

    return render;
  }

  // Returns the inner text for the submit button
  function getSubmitText() {
    let text = dictionary.actions[currentAction ?? 'borrow'].title;
    if (sendingTransaction) {
      text = dictionary.common.sending + '..';
    }

    return text;
  }

  // Handle user closing the modal
  function handleCancel() {
    // Don't close if we're sending a tx
    if (sendingTransaction) {
      return;
    }

    // Close modal and reset tokenInput
    resetCurrentAction();
    resetTokenInputString();
    resetTokenInputAmount();
  }

  currentAccount?.refresh();
  // Only return the modal if we're borrowing or repaying
  if (
    currentAccount &&
    (currentAction === 'borrow' || currentAction === 'repay' || currentAction === 'repayFromDeposit')
  ) {
    return (
      <Modal open className="action-modal" maskClosable={false} footer={null} onCancel={handleCancel}>
        <Tabs
          activeKey={currentAction === 'borrow' ? 'borrow' : 'repay'}
          onChange={(action: string) => setCurrentAction(action as PoolAction)}
          items={tabItems}
        />
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common['accountBalance'].toUpperCase()}</Text>
          {renderAccountBalance()}
        </div>
        <TokenInput account={currentAccount} onPressEnter={borrowRepay} />
        <div className="action-info flex-centered column">
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.loanBalance}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(getLoanBalance(), true)}>{getLoanBalance()}</Paragraph>
              {renderAffectedLoanBalance()}
            </div>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.poolBorrowRate}</Paragraph>
            <Paragraph type="secondary">
              {formatRate(currentPool ? currentPool.borrowApr : 0)}
              {renderAffectedBorrowRate()}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between" data-testid="predicted-risk">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount.riskIndicator)}</Paragraph>
              {renderAffectedRiskLevel()}
            </div>
          </div>
        </div>
        <Button
          block
          disabled={disabled || tokenInputAmount.isZero() || projectedRiskIndicator > 1}
          loading={sendingTransaction}
          onClick={borrowRepay}>
          {getSubmitText()}
        </Button>
        <div className={`action-modal-overlay ${sendingTransaction ? 'showing' : ''}`}></div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
