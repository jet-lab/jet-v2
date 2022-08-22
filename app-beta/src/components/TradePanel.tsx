import { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { MarginAccount, PoolAction, PoolProjection, PoolTokenChange, TokenAmount } from '@jet-lab/margin';
import { useMargin } from '../contexts/marginContext';
import { useTradeContext } from '../contexts/tradeContext';
import { useLanguage } from '../contexts/localization/localization';
import { useTransactionLogs } from '../contexts/transactionLogs';
import { TransactionResponse, TxResponseType, useMarginActions } from '../hooks/useMarginActions';
import { currencyFormatter } from '../utils/currency';
import { notification, Select, Slider, Tooltip } from 'antd';
import { JetInput } from './JetInput';
import { ConnectMessage } from './ConnectMessage';
import { InfoCircleOutlined } from '@ant-design/icons';
import { useBlockExplorer } from '../contexts/blockExplorer';

export function TradePanel(): JSX.Element {
  const { dictionary } = useLanguage();
  const { pools, marginAccount, walletBalances, userFetched } = useMargin();
  const { refreshLogs } = useTransactionLogs();
  const { connected } = useWallet();
  const { getExplorerUrl } = useBlockExplorer();
  const {
    currentPool,
    currentAction,
    setCurrentAction,
    currentAmount,
    setCurrentAmount,
    sendingTrade,
    setSendingTrade
  } = useTradeContext();
  const accountPoolPosition =
    marginAccount && currentPool?.symbol ? marginAccount.poolPositions[currentPool.symbol] : undefined;
  const accountSummary = marginAccount && marginAccount.summary;
  const maxInput = accountPoolPosition?.maxTradeAmounts[currentAction].tokens ?? 0;
  const [disabledInput, setDisabledInput] = useState<boolean>(false);
  const [disabledMessage, setDisabledMessage] = useState<string>('');
  const [disabledButton, setDisabledButton] = useState<boolean>(false);
  const [inputError, setInputError] = useState<string>('');
  const [inputWarning, setInputWarning] = useState<string>('');
  const tradeActions: PoolAction[] = ['deposit', 'withdraw', 'borrow', 'repay'];
  const { deposit, withdraw, borrow, repay } = useMarginActions();
  const { Option } = Select;

  let poolProjection: PoolProjection | undefined;
  if (currentPool && marginAccount) {
    const projectWith = Math.min(currentAmount ?? 0, maxInput);
    poolProjection = currentPool.projectAfterAction(marginAccount, projectWith, currentAction);
  }
  const predictedRiskIndicator = poolProjection?.riskIndicator ?? marginAccount?.riskIndicator.valueOf() ?? 0;

  // Check if user input should be disabled
  // depending on wallet balance and position
  function checkDisabledInput() {
    // Initially set to true and reset message
    setDisabledMessage('');
    setDisabledInput(true);
    if (!currentPool || currentPool === undefined || !currentPool.symbol || !walletBalances) {
      return;
    }

    // Depositing
    if (currentAction === 'deposit') {
      // No wallet balance to deposit
      if (!walletBalances[currentPool.symbol].amount.tokens) {
        setDisabledMessage(dictionary.cockpit.noBalanceForDeposit.replaceAll('{{ASSET}}', currentPool.symbol));
      } else {
        setDisabledInput(false);
      }
      // Withdrawing
    } else if (currentAction === 'withdraw') {
      // No collateral to withdraw
      if (!accountPoolPosition?.depositBalance.tokens) {
        setDisabledMessage(dictionary.cockpit.noDepositsForWithdraw.replaceAll('{{ASSET}}', currentPool.symbol));
        // User is above max risk
      } else if (marginAccount && marginAccount.riskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL) {
        setDisabledMessage(dictionary.cockpit.aboveMaxRiskLevel);
      } else {
        setDisabledInput(false);
      }
      // Borrowing
    } else if (currentAction === 'borrow') {
      // User has not deposited any collateral
      if (!accountSummary?.depositedValue) {
        setDisabledMessage(dictionary.cockpit.noDepositsForBorrow);
        // User is above max risk
      } else if (marginAccount && marginAccount.riskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL) {
        setDisabledMessage(dictionary.cockpit.aboveMaxRiskLevel);
        // No liquidity in market to borrow from
      } else if (!currentPool.vault.tokens) {
        setDisabledMessage(dictionary.cockpit.noLiquidity);
      } else {
        setDisabledInput(false);
      }
      // Repaying
    } else if (currentAction === 'repay') {
      // User has no loan balance to repay
      if (!accountPoolPosition?.loanBalance.tokens) {
        setDisabledMessage(dictionary.cockpit.noDebtForRepay.replaceAll('{{ASSET}}', currentPool.symbol));
      } else {
        setDisabledInput(false);
      }
    }
  }

  // Check user input and for Copilot warning
  // Then submit trade RPC call
  async function submitTrade() {
    if (!currentPool?.symbol || !accountPoolPosition || !accountSummary || !currentAmount || !walletBalances) {
      return;
    }

    const tradeAction = currentAction;
    const tradeAmount = TokenAmount.tokens(currentAmount.toString(), currentPool.decimals);
    let res: TransactionResponse = { txid: '', response: TxResponseType.Cancelled };
    let tradeError = '';
    setSendingTrade(true);

    // Depositing
    if (tradeAction === 'deposit') {
      // User is depositing more than they have in their wallet
      if (tradeAmount.gt(accountPoolPosition.maxTradeAmounts.deposit)) {
        tradeError = dictionary.cockpit.notEnoughAsset.replaceAll('{{ASSET}}', currentPool.symbol);
        // Otherwise, send deposit
      } else {
        const depositAmount = PoolTokenChange.shiftBy(tradeAmount);
        res = await deposit(currentPool.symbol, depositAmount);
      }
      // Withdrawing sollet ETH
    } else if (tradeAction === 'withdraw') {
      // User is withdrawing more than liquidity in market
      if (tradeAmount.gt(currentPool.vault)) {
        tradeError = dictionary.cockpit.noLiquidity;
        // User is withdrawing more than they've deposited
      } else if (tradeAmount.gt(accountPoolPosition.maxTradeAmounts.withdraw)) {
        tradeError = dictionary.cockpit.lessFunds;
        // Otherwise, send withdraw
      } else {
        // If user is withdrawing all, set to 0 to withdraw dust
        const withdrawAmount =
          tradeAmount.tokens === accountPoolPosition.depositBalance.tokens
            ? PoolTokenChange.setTo(0)
            : PoolTokenChange.shiftBy(tradeAmount);
        res = await withdraw(currentPool.symbol, withdrawAmount);
      }
      // Borrowing
    } else if (tradeAction === 'borrow') {
      // User is borrowing more than liquidity in market
      if (tradeAmount.gt(currentPool.vault)) {
        tradeError = dictionary.cockpit.noLiquidity;
        // User is above max risk
      } else if (marginAccount && marginAccount.riskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL) {
        tradeError = dictionary.cockpit.aboveMaxRiskLevel;
        // User is borring more than max
      } else if (tradeAmount.gt(accountPoolPosition.maxTradeAmounts.borrow)) {
        tradeError = dictionary.cockpit.moreThanMaxBorrow
          .replaceAll('{{AMOUNT}}', accountPoolPosition.maxTradeAmounts.borrow.uiTokens)
          .replaceAll('{{ASSET}}', currentPool.symbol);
        // Otherwise, send borrow
      } else {
        const borrowAmount = PoolTokenChange.shiftBy(tradeAmount);
        res = await borrow(currentPool.symbol, borrowAmount);
      }
      // Repaying
    } else if (tradeAction === 'repay') {
      // User is repaying more than they owe
      if (tradeAmount.gt(accountPoolPosition.loanBalance)) {
        tradeError = dictionary.cockpit.oweLess;
        // User input amount is larger than wallet balance
      } else if (tradeAmount.gt(walletBalances[currentPool.symbol].amount)) {
        tradeError = dictionary.cockpit.notEnoughAsset.replaceAll('{{ASSET}}', currentPool.symbol);
        // Otherwise, send repay
      } else {
        // If user is repaying all, set to 0 to repay dust
        const repayAmount =
          tradeAmount.tokens === accountPoolPosition.loanBalance.tokens
            ? PoolTokenChange.setTo(0)
            : PoolTokenChange.shiftBy(tradeAmount);
        res = await repay(currentPool.symbol, repayAmount);
      }
    }

    // If input error, remove trade amount and return`
    if (tradeError) {
      setInputError(tradeError);
      setSendingTrade(false);
      return;
    }

    // Notify user of successful/unsuccessful trade
    if (res.response === TxResponseType.Success) {
      notification.success({
        message: dictionary.cockpit.txSuccessShort.replaceAll(
          '{{TRADE ACTION}}',
          tradeAction[0].toUpperCase() + tradeAction.substring(1)
        ),
        description: dictionary.cockpit.txSuccess
          .replaceAll('{{TRADE ACTION}}', currentAction)
          .replaceAll('{{AMOUNT AND ASSET}}', `${currentAmount} ${currentPool.symbol}`),
        placement: 'bottomLeft',
        onClick: () => {
          res.txid && window.open(getExplorerUrl(res.txid), '_blank');
        }
      });

      setCurrentAmount(null);
    } else if (res.response === TxResponseType.Failed) {
      notification.error({
        message: dictionary.cockpit.txFailedShort,
        description: dictionary.cockpit.txFailed,
        placement: 'bottomLeft',
        onClick: () => {
          res.txid && window.open(getExplorerUrl(res.txid), '_blank');
        }
      });
    } else if (res.response === TxResponseType.Cancelled) {
      notification.error({
        message: dictionary.cockpit.txFailedShort,
        description: dictionary.cockpit.txCancelled,
        placement: 'bottomLeft'
      });
    }

    // Readjust interface
    checkDisabledInput();
    // End trade submit
    setSendingTrade(false);
    // Add Tx Log
    refreshLogs();
  }

  // Readjust interface onmount
  // and current reserve change
  useEffect(() => {
    checkDisabledInput();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool, accountPoolPosition, accountSummary, currentAction]);

  // If user disconnects wallet, reset inputs
  useEffect(() => {
    setCurrentAmount(null);
  }, [setCurrentAmount, userFetched]);

  // On user input, check for error / warning
  useEffect(() => {
    setDisabledButton(false);
    setInputError('');
    setInputWarning('');
    if (!currentPool || !currentAmount || !walletBalances) {
      return;
    }

    // Withdrawing
    if (currentAction === 'withdraw') {
      if (accountPoolPosition && currentAmount > accountPoolPosition.maxTradeAmounts.withdraw.tokens) {
        setInputError(dictionary.cockpit.lessFunds);
      } else if (
        predictedRiskIndicator >= MarginAccount.RISK_WARNING_LEVEL &&
        predictedRiskIndicator <= MarginAccount.RISK_LIQUIDATION_LEVEL
      ) {
        setInputWarning(
          dictionary.cockpit.subjectToLiquidation.replaceAll(
            '{{NEW-RISK}}',
            currencyFormatter(predictedRiskIndicator, false, 2)
          )
        );
      } else if (predictedRiskIndicator > MarginAccount.RISK_LIQUIDATION_LEVEL) {
        setInputError(
          dictionary.cockpit.subjectToLiquidation.replaceAll(
            '{{NEW-RISK}}',
            currencyFormatter(predictedRiskIndicator, false, 2)
          )
        );
        setSendingTrade(false);
      }
      // Borrowing
    } else if (currentAction === 'borrow') {
      if (accountPoolPosition && currentAmount > accountPoolPosition.maxTradeAmounts.borrow.tokens) {
        setInputError(
          dictionary.cockpit.moreThanMaxBorrow
            .replaceAll('{{AMOUNT}}', accountPoolPosition.maxTradeAmounts.borrow.uiTokens)
            .replaceAll('{{ASSET}}', currentPool.symbol)
        );
      } else if (
        predictedRiskIndicator >= MarginAccount.RISK_WARNING_LEVEL &&
        predictedRiskIndicator < MarginAccount.RISK_LIQUIDATION_LEVEL
      ) {
        setInputWarning(
          dictionary.cockpit.subjectToLiquidation.replaceAll(
            '{{NEW-RISK}}',
            currencyFormatter(predictedRiskIndicator, false, 2)
          )
        );
      } else if (predictedRiskIndicator >= MarginAccount.RISK_LIQUIDATION_LEVEL) {
        setInputError(
          dictionary.cockpit.rejectTrade
            .replaceAll('{{NEW_RISK}}', currencyFormatter(predictedRiskIndicator, false, 2))
            .replaceAll('{{MAX_RISK}}', currencyFormatter(1 / MarginAccount.RISK_LIQUIDATION_LEVEL, false, 1))
        );
        setDisabledButton(true);
      }
    } else if (currentAction === 'repay') {
      if (currentPool.symbol && currentAmount > walletBalances[currentPool.symbol].amount.tokens) {
        setInputError(dictionary.cockpit.notEnoughAsset.replaceAll('{{ASSET}}', currentPool.symbol));
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentAmount, predictedRiskIndicator, currentAction]);

  const availBalance = () => {
    if (currentAction === 'deposit') {
      if (currentPool?.symbol === 'SOL') {
        return 'Max deposit'.toUpperCase();
      } else {
        return dictionary.cockpit.walletBalance.toUpperCase();
      }
    } else if (currentAction === 'withdraw') {
      return dictionary.cockpit.availableFunds.toUpperCase();
    } else if (currentAction === 'borrow') {
      if (currentPool && maxInput <= currentPool.vault.tokens) {
        return dictionary.cockpit.maxBorrowAmount.toUpperCase();
      } else {
        return dictionary.cockpit.availableLiquidity.toUpperCase();
      }
    } else {
      return dictionary.cockpit.amountOwed.toUpperCase();
    }
  };

  return (
    <div className="trade-panel flex align-center justify-start">
      <div className="trade-select-container flex align-center justify-between">
        {tradeActions.map(action => (
          <div
            key={action}
            onClick={() => {
              if (!sendingTrade) {
                setCurrentAction(action);
                checkDisabledInput();
              }
            }}
            className={`trade-select flex justify-center align-center ${currentAction === action ? 'active' : ''}`}>
            <p className="semi-bold-text">{dictionary.cockpit[action].toUpperCase()}</p>
          </div>
        ))}
        <div className="mobile-trade-select flex-centered">
          <Select
            value={currentAction}
            onChange={action => {
              if (!sendingTrade) {
                setCurrentAction(action);
                checkDisabledInput();
              }
            }}>
            {tradeActions.map(action => (
              <Option key={action} value={action}>
                {action.toUpperCase()}
              </Option>
            ))}
          </Select>
        </div>
      </div>
      {!connected || !pools ? <ConnectMessage /> : <></>}
      {disabledMessage.length || inputError.length || inputWarning.length ? (
        <div className="trade-section trade-section-disabled-message flex-centered column">
          <span className={`center-text ${inputError ? 'danger-text' : inputWarning ? 'warning-text' : ''}`}>
            {disabledMessage || inputError || inputWarning}
          </span>
        </div>
      ) : (
        <>
          <div className={`trade-section flex-centered column ${disabledInput ? 'disabled' : ''}`}>
            <span className="center-text bold-text">
              {availBalance()}{' '}
              {currentAction === 'deposit' && currentPool?.symbol === 'SOL' && (
                <Tooltip title={dictionary.cockpit.minSol}>
                  <InfoCircleOutlined />
                </Tooltip>
              )}
            </span>
            <div className="flex-centered">
              <p className="center-text max-amount" onClick={() => setCurrentAmount(maxInput)}>
                {userFetched && currentPool
                  ? currencyFormatter(maxInput, false, currentPool.decimals) + ' ' + currentPool.symbol
                  : '--'}
              </p>
            </div>
          </div>
          <div className={`trade-section flex-centered column ${disabledInput ? 'disabled' : ''}`}>
            <div className="flex-centered">
              <span className="center-text bold-text">{dictionary.cockpit.predictedRiskLevel.toUpperCase()}</span>
            </div>
            <p>{userFetched && marginAccount ? currencyFormatter(predictedRiskIndicator, false, 2) : '--'}</p>
          </div>
        </>
      )}
      <div className="trade-section flex-centered column">
        <JetInput
          type="number"
          currency
          value={currentAmount}
          maxInput={maxInput}
          disabled={!userFetched || disabledInput}
          disabledButton={disabledButton}
          loading={sendingTrade}
          error={inputError}
          warning={inputWarning}
          onChange={(value: number) => {
            const newAmount = value;
            if (newAmount < 0) {
              setCurrentAmount(0);
            } else {
              setCurrentAmount(newAmount);
            }
          }}
          submit={submitTrade}
        />
        <Slider
          dots
          value={((currentAmount ?? 0) / maxInput) * 100}
          min={0}
          max={100}
          step={1}
          disabled={!userFetched || disabledInput}
          onChange={percent => {
            if (!currentPool) {
              return;
            }

            const value = maxInput * ((percent ?? 0) / 100);
            const newAmount = (value * 10 ** currentPool.decimals) / 10 ** currentPool.decimals;
            setCurrentAmount(parseFloat(newAmount.toFixed(currentPool.decimals)));
          }}
          tipFormatter={value => value + '%'}
          tooltipPlacement="bottom"
          marks={{
            0: '0%',
            25: '25%',
            50: '50%',
            75: '75%',
            100: dictionary.cockpit.max
          }}
        />
      </div>
    </div>
  );
}
