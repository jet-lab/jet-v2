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
import { useRiskStyle } from '../../../utils/risk';
import { ActionResponse } from '../../../utils/jet/marginActions';
import { useMarginActions } from '../../../utils/jet/marginActions';
import { Button, Modal, Tabs, Typography } from 'antd';
import { TokenInput } from '../../misc/TokenInput';

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
  const accountPoolPosition = currentPool?.symbol ? currentAccount?.poolPositions[currentPool.symbol] : undefined;
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const disabledMessage = useTokenInputDisabledMessage();
  const riskStyle = useRiskStyle();
  const projectedRiskIndicator =
    currentPool && currentAccount && currentAction && !tokenInputAmount.isZero()
      ? currentPool.projectAfterAction(currentAccount, tokenInputAmount.tokens, currentAction).riskIndicator
      : currentAccount?.riskIndicator ?? 0;
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useState(false);
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

  if (currentAction === 'deposit' || currentAction === 'withdraw') {
    return (
      <Modal
        visible
        className="action-modal"
        footer={null}
        onCancel={() => {
          resetCurrentAction();
          resetTokenInputString();
        }}>
        <Tabs
          activeKey={currentAction ?? 'deposit'}
          onChange={(action: string) => setCurrentAction(action as PoolAction)}>
          {['deposit', 'withdraw'].map(action => (
            <TabPane tab={action} key={action}></TabPane>
          ))}
        </Tabs>
        <div className="wallet-balance flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {walletTokens && currentPool?.symbol && (
            <Paragraph type="secondary" italic>{`${walletTokens.map[currentPool.symbol].amount.tokens} ${
              currentPool.symbol
            }`}</Paragraph>
          )}
        </div>
        <TokenInput
          account={currentAction === 'withdraw' ? currentAccount : undefined}
          onChangeToken={(tokenSymbol: string) => {
            setCurrentPoolSymbol(tokenSymbol);
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
              {currencyAbbrev(accountPoolPosition?.depositBalance.tokens ?? 0, false, undefined, currentPool?.decimals)}
              {!tokenInputAmount.isZero() && accountPoolPosition && (
                <>
                  &nbsp;&#8594;&nbsp;
                  {currencyAbbrev(
                    currentAction === 'deposit'
                      ? accountPoolPosition.depositBalance.tokens + tokenInputAmount.tokens
                      : accountPoolPosition.depositBalance.tokens - tokenInputAmount.tokens,
                    false,
                    undefined,
                    currentPool?.decimals
                  )}
                </>
              )}
              {' ' + (currentPool?.symbol ?? '—')}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.poolDepositRate}</Paragraph>
            <Paragraph type="secondary">
              {formatRate(currentPool?.depositApy ?? 0)}
              {!tokenInputAmount.isZero() && (
                <>
                  &nbsp;&#8594;&nbsp;
                  {formatRate(
                    currentPool && currentAccount
                      ? currentPool.projectAfterAction(currentAccount, tokenInputAmount.tokens, currentAction)
                          .depositRate
                      : 0
                  )}
                </>
              )}
            </Paragraph>
          </div>
          <div className="action-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
              {!tokenInputAmount.isZero() && (
                <Paragraph type={projectedRiskStyle}>
                  &nbsp;&#8594;&nbsp;{formatRiskIndicator(projectedRiskIndicator)}
                </Paragraph>
              )}
            </div>
          </div>
        </div>
        <Button
          block
          disabled={sendingTransaction || disabledMessage.length < 0 || tokenInputAmount.isZero()}
          loading={sendingTransaction}
          onClick={depositWithdraw}>
          {sendingTransaction ? dictionary.common.sending + '..' : dictionary.actions[currentAction ?? 'deposit'].title}
        </Button>
      </Modal>
    );
  } else {
    return <></>;
  }
}
