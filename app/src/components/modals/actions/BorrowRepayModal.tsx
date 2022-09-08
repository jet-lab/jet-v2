import { useState } from 'react';
import { useRecoilState, useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '../../../state/settings/localization/localization';
import { BlockExplorer, Cluster } from '../../../state/settings/settings';
import { WalletTokens } from '../../../state/user/walletTokens';
import { CurrentAccount } from '../../../state/user/accounts';
import { CurrentMarketPair } from '../../../state/trade/market';
import { CurrentPoolSymbol, CurrentPool } from '../../../state/borrow/pools';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import { useTokenInputDisabledMessage } from '../../../utils/actions/tokenInput';
import { useCurrencyFormatting } from '../../../utils/currency';
import { formatRiskIndicator, formatRate } from '../../../utils/format';
import { useMarginActions, ActionResponse } from '../../../utils/jet/marginActions';
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { useProjectedRisk, useRiskStyle } from '../../../utils/risk';
import { ArrowRight } from './ArrowRight';
import { Button, Modal, Tabs, Typography } from 'antd';
import { TokenInput } from '../../misc/TokenInput/TokenInput';

// Modal to Borrow / Repay using the current Pool
export function BorrowRepayModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { borrow, repay } = useMarginActions();
  const walletTokens = useRecoilValue(WalletTokens);
  const currentPool = useRecoilValue(CurrentPool);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const currentAccount = useRecoilValue(CurrentAccount);
  const accountPoolPosition = currentPool ? currentAccount?.poolPositions[currentPool.symbol] : undefined;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const riskStyle = useRiskStyle();
  const projectedRiskIndicator = useProjectedRisk();
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const disabledMessage = useTokenInputDisabledMessage();
  const disabled = sendingTransaction || disabledMessage.length < 0 || tokenInputAmount.isZero();
  const { Paragraph, Text } = Typography;
  const { TabPane } = Tabs;

  // Borrow / Repay
  async function borrowRepay() {
    setSendingTransaction(true);
    const [txId, resp] = currentAction === 'borrow' ? await borrow() : await repay();
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

  // Returns the margin account's loan balance
  function getLoanBalance(balance?: number) {
    let loanBalance = balance ?? 0;
    if (!balance && accountPoolPosition) {
      loanBalance = accountPoolPosition.loanBalance.tokens;
    }

    let decimals = 2;
    if (currentPool) {
      decimals = currentPool.decimals;
    }

    const abbreviatedLoanBalance = currencyAbbrev(loanBalance, false, undefined, decimals);
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
          {affectedBalance}
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
        <Paragraph type={projectedRiskStyle}>
          <ArrowRight />
          {affectedRiskLevel}
        </Paragraph>
      );
    }

    return render;
  }

  // Renders the wallet balance for current pool token
  function renderWalletBalance() {
    let render = <></>;
    if (walletTokens && currentPool) {
      const walletBalance = walletTokens.map[currentPool.symbol].amount.tokens + ' ' + currentPool.symbol;
      render = (
        <Paragraph type="secondary" italic>
          {walletBalance}
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

  // Only return the modal if we're borrowing or repaying
  if (currentAccount && (currentAction === 'borrow' || currentAction === 'repay')) {
    return (
      <Modal visible className="action-modal" footer={null} onCancel={handleCancel}>
        <Tabs activeKey={currentAction} onChange={(action: string) => setCurrentAction(action as PoolAction)}>
          {['borrow', 'repay'].map(action => (
            <TabPane tab={action} key={action}></TabPane>
          ))}
        </Tabs>
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {renderWalletBalance()}
        </div>
        <TokenInput
          account={currentAccount}
          onChangeToken={(tokenSymbol: string) => {
            setCurrentPoolSymbol(tokenSymbol);
            // If we're not switching to USDC, also update the currentMarketPair
            if (tokenSymbol !== 'USDC') {
              setCurrentMarketPair(`${tokenSymbol}/USDC`);
            }
          }}
          loading={sendingTransaction}
          onPressEnter={borrowRepay}
        />
        <div className="action-info flex-centered column">
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="danger">{dictionary.common.loanBalance}</Paragraph>
            <Paragraph type="danger">
              {getLoanBalance()}
              {renderAffectedLoanBalance()}
              {' ' + (currentPool ? currentPool.symbol : '—')}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.poolBorrowRate}</Paragraph>
            <Paragraph type="secondary">
              {formatRate(currentPool ? currentPool.borrowApr : 0)}
              {renderAffectedBorrowRate()}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount.riskIndicator)}</Paragraph>
              {renderAffectedRiskLevel()}
            </div>
          </div>
        </div>
        <Button block disabled={disabled} loading={sendingTransaction} onClick={borrowRepay}>
          {getSubmitText()}
        </Button>
        <div className={`action-modal-overlay ${sendingTransaction ? 'showing' : ''}`}></div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
