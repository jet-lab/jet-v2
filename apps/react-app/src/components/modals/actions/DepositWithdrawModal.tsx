import { useRecoilState, useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { PoolAction } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { ActionRefresh, SendingTransaction } from '@state/actions/actions';
import { WalletTokens } from '@state/user/walletTokens';
import { CurrentAccount, AccountsLoading } from '@state/user/accounts';
import { Pools } from '@state/pools/pools';
import { CurrentAction, TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { useTokenInputDisabledMessage } from '@utils/actions/tokenInput';
import { useCurrencyFormatting } from '@utils/currency';
import { formatRiskIndicator, formatRate } from '@utils/format';
import { getExplorerUrl, getTokenStyleType } from '@utils/ui';
import { notify } from '@utils/notify';
import { useProjectedRisk, useRiskStyle } from '@utils/risk';
import { ActionResponse } from '@utils/jet/marginActions';
import { useMarginActions } from '@utils/jet/marginActions';
import { ArrowRight } from './ArrowRight';
import { TokenInput } from '@components/misc/TokenInput/TokenInput';
import { Button, Modal, Tabs, Typography } from 'antd';
import { useEffect, useMemo } from 'react';
import { useJetStore } from '@jet-lab/store';
import { LoadingOutlined } from '@ant-design/icons';

// Modal to Deposit / Withdraw using the current Pool
export function DepositWithdrawModal(): JSX.Element {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const pools = useRecoilValue(Pools);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );

  const dictionary = useRecoilValue(Dictionary);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { deposit, withdraw } = useMarginActions();
  const walletTokens = useRecoilValue(WalletTokens);
  const [currentAction, setCurrentAction] = useRecoilState(CurrentAction);
  const resetCurrentAction = useResetRecoilState(CurrentAction);
  const currentAccount = useRecoilValue(CurrentAccount);
  const accountPoolPosition = currentPool ? currentAccount?.poolPositions[currentPool.symbol] : undefined;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const setTokenInputString = useSetRecoilState(TokenInputString);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  const setActionRefresh = useSetRecoilState(ActionRefresh);
  const accountsLoading = useRecoilValue(AccountsLoading);
  const riskStyle = useRiskStyle();
  const projectedRiskIndicator = useProjectedRisk();
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const disabledMessage = useTokenInputDisabledMessage();
  const disabled = sendingTransaction || disabledMessage.length > 0;
  const { Paragraph, Text } = Typography;
  const tabItems = ['deposit', 'withdraw'].map((action: string) => {
    return {
      label: action,
      key: action
    };
  });

  useEffect(() => {
    if (currentAction === 'deposit' || currentAction === 'withdraw') setActionRefresh(Date.now());
  }, [currentAction]);

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
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
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
        'error',
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
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

    const decimals = currentPool?.precision ?? 2;

    const abbreviatedDepositBalance = currencyAbbrev(balance ?? depositBalance, decimals, false, undefined);
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
          <Paragraph type={getTokenStyleType(affectedBalance)}>{affectedBalance}</Paragraph>
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
        <>
          <ArrowRight />
          <Paragraph type={projectedRiskStyle}>{affectedRiskLevel}</Paragraph>
        </>
      );
    }

    return render;
  }

  // Renders the wallet balance for current pool token
  function renderWalletBalance() {
    let render = <></>;
    if (walletTokens && currentPool) {
      const walletBalance = walletTokens.map[currentPool.symbol].amount.uiTokens;
      render = (
        <Paragraph
          onClick={() => (!disabled ? setTokenInputString(walletBalance) : null)}
          className={!disabled ? 'token-balance' : 'secondary-text'}>
          {walletBalance + ' ' + currentPool.symbol}
        </Paragraph>
      );
    }

    return render;
  }

  // Renders the account balance for current pool token
  function renderAccountBalance() {
    let render = <></>;
    if (currentAccount && currentPool) {
      const accountBalance = currentAccount.poolPositions[currentPool.symbol].depositBalance.uiTokens;
      render = accountsLoading ? (
        <Paragraph>
          <LoadingOutlined />
        </Paragraph>
      ) : (
        <Paragraph
          onClick={() => (!disabled ? setTokenInputString(accountBalance) : null)}
          className={!disabled ? 'token-balance' : 'secondary-text'}>
          {accountBalance + ' ' + currentPool.symbol}
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
      <Modal open className="action-modal" maskClosable={false} footer={null} onCancel={handleCancel}>
        <Tabs
          activeKey={currentAction ?? 'deposit'}
          onChange={(action: string) => setCurrentAction(action as PoolAction)}
          items={tabItems}
        />
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">
            {dictionary.common[currentAction === 'deposit' ? 'walletBalance' : 'accountBalance'].toUpperCase()}
          </Text>
          {currentAction === 'deposit' ? renderWalletBalance() : renderAccountBalance()}
        </div>
        <TokenInput onPressEnter={depositWithdraw} />
        <div className="action-info flex-centered column">
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.accountBalance}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(getDepositBalance())}>{getDepositBalance()}</Paragraph>
              {renderAffectedDepositBalance()}
            </div>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.poolDepositRate}</Paragraph>
            <Paragraph type="secondary">
              {formatRate(currentPool ? currentPool.depositApy : 0)}
              {renderAffectedDepositRate()}
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
          disabled={
            disabled || tokenInputAmount.isZero() || (projectedRiskIndicator > 2 && currentAction !== 'deposit')
          }
          loading={sendingTransaction}
          onClick={depositWithdraw}>
          {getSubmitText()}
        </Button>
        <div className={`action-modal-overlay ${sendingTransaction ? 'showing' : ''}`}></div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
