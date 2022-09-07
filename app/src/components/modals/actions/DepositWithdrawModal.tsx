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
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { useProjectedRisk, useRiskStyle } from '../../../utils/risk';
import { ActionResponse } from '../../../utils/jet/marginActions';
import { useMarginActions } from '../../../utils/jet/marginActions';
import { ArrowRight } from './ArrowRight';
import { TokenInput } from '../../misc/TokenInput/TokenInput';
import { Button, Modal, Tabs, Typography } from 'antd';

export function DepositWithdrawModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { deposit, withdraw } = useMarginActions();
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

  // Deposit / Withdraw
  async function depositWithdraw() {
    setSendingTransaction(true);
    const [txId, resp] = currentAction === 'deposit' ? await deposit() : await withdraw();
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

  // Returns the margin account's deposit balance
  function getDepositBalance(balance?: number) {
    let depositBalance = 0;
    if (accountPoolPosition) {
      depositBalance = accountPoolPosition.depositBalance.tokens;
    }

    let decimals = 2;
    if (currentPool) {
      decimals = currentPool.decimals;
    }

    const abbreviatedDepositBalance = currencyAbbrev(depositBalance, false, undefined, decimals);
    return abbreviatedDepositBalance;
  }

  // Renders account's affected deposit balance
  function renderAffectedDepositBalance() {
    let render = <></>;
    if (!tokenInputAmount.isZero() && accountPoolPosition) {
      const newBalance =
        currentAction === 'deposit'
          ? accountPoolPosition.depositBalance.tokens + tokenInputAmount.tokens
          : accountPoolPosition.depositBalance.tokens - tokenInputAmount.tokens;
      const affectedBalance = getDepositBalance(newBalance);

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
  function renderAffectedDepositRate() {
    let render = <></>;
    if (!tokenInputAmount.isZero() && currentPool && currentAccount && currentAction) {
      const newRate = currentPool.projectAfterAction(
        currentAccount,
        tokenInputAmount.tokens,
        currentAction
      ).depositRate;
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
    let text = dictionary.actions[currentAction ?? 'deposit'].title;
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

  if (currentAccount && (currentAction === 'deposit' || currentAction === 'withdraw')) {
    return (
      <Modal visible className="action-modal" footer={null} onCancel={handleCancel}>
        <Tabs
          activeKey={currentAction ?? 'deposit'}
          onChange={(action: string) => setCurrentAction(action as PoolAction)}>
          {['deposit', 'withdraw'].map(action => (
            <TabPane tab={action} key={action}></TabPane>
          ))}
        </Tabs>
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {renderWalletBalance()}
        </div>
        <TokenInput
          account={currentAction === 'withdraw' ? currentAccount : undefined}
          onChangeToken={(tokenSymbol: string) => {
            setCurrentPoolSymbol(tokenSymbol);
            // If we're not switching to USDC, also update the currentMarketPair
            if (tokenSymbol !== 'USDC') {
              setCurrentMarketPair(`${tokenSymbol}/USDC`);
            }
          }}
          loading={sendingTransaction}
          onPressEnter={depositWithdraw}
        />
        <div className="action-info flex-centered column">
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="success">{dictionary.common.accountBalance}</Paragraph>
            <Paragraph type="success">
              {getDepositBalance()}
              {renderAffectedDepositBalance()}
              {' ' + (currentPool ? currentPool.symbol : '—')}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.poolDepositRate}</Paragraph>
            <Paragraph type="secondary">
              {formatRate(currentPool ? currentPool.depositApy : 0)}
              {renderAffectedDepositRate()}
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
        <Button block disabled={disabled} loading={sendingTransaction} onClick={depositWithdraw}>
          {getSubmitText()}
        </Button>
        <div className={`action-modal-overlay ${sendingTransaction ? 'showing' : ''}`}></div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
