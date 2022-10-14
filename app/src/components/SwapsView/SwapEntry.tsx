import { useEffect, useState } from 'react';
import { useRecoilState, useResetRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { SwapsRowOrder } from '../../state/views/views';
import { BlockExplorer, Cluster } from '../../state/settings/settings';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentAccount } from '../../state/user/accounts';
import { CurrentPoolSymbol, Pools, CurrentPool, PoolOptions } from '../../state/pools/pools';
import {
  CurrentAction,
  CurrentSwapOutput,
  SendingTransaction,
  TokenInputAmount,
  TokenInputString
} from '../../state/actions/actions';
import { useProjectedRisk, useRiskStyle } from '../../utils/risk';
import { formatPriceImpact, formatRiskIndicator } from '../../utils/format';
import { notify } from '../../utils/notify';
import { getExplorerUrl, getTokenStyleType } from '../../utils/ui';
import { DEFAULT_DECIMALS, useCurrencyFormatting } from '../../utils/currency';
import { getMinOutputAmount, getOutputTokenAmount, useSwapReviewMessage } from '../../utils/actions/swap';
import { ActionResponse, useMarginActions } from '../../utils/jet/marginActions';
import { Info } from '../misc/Info';
import { TokenInput } from '../misc/TokenInput/TokenInput';
import { ReorderArrows } from '../misc/ReorderArrows';
import { ConnectionFeedback } from '../misc/ConnectionFeedback/ConnectionFeedback';
import { ArrowRight } from '../modals/actions/ArrowRight';
import { Button, Checkbox, Input, Radio, Typography } from 'antd';
import SwapIcon from '../../assets/icons/function-swap.svg';
import { CurrentSplSwapPool, hasOrcaPool, SwapFees, SwapPoolTokenAmounts } from '../../state/swap/splSwap';
import { useTokenInputDisabledMessage, useTokenInputErrorMessage } from '../../utils/actions/tokenInput';

// Component for user to enter and submit a swap action
export function SwapEntry(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { splTokenSwap } = useMarginActions();
  // Margin account
  const currentAction = useRecoilValue(CurrentAction);
  const currentAccount = useRecoilValue(CurrentAccount);
  // Pools
  const pools = useRecoilValue(Pools);
  const poolOptions = useRecoilValue(PoolOptions);
  // Input token pool
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const currentPool = useRecoilValue(CurrentPool);
  const poolPrecision = currentPool?.precision ?? DEFAULT_DECIMALS;
  const poolPosition = currentAccount && currentPool && currentAccount.poolPositions[currentPool.symbol];
  const overallInputBalance = poolPosition ? poolPosition.depositBalance.tokens - poolPosition.loanBalance.tokens : 0;
  const depositBalanceString = poolPosition ? poolPosition.depositBalance.uiTokens : '0';
  const maxSwapString = poolPosition ? poolPosition.maxTradeAmounts.swap.uiTokens : '0';
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const disabledMessage = useTokenInputDisabledMessage();
  // Output token pool
  const [outputToken, setOutputToken] = useRecoilState(CurrentSwapOutput);
  const outputPrecision = outputToken?.precision ?? DEFAULT_DECIMALS;
  const outputPoolPosition = currentAccount && outputToken && currentAccount?.poolPositions[outputToken.symbol];
  const overallOutputBalance = outputPoolPosition
    ? outputPoolPosition.depositBalance.tokens - outputPoolPosition.loanBalance.tokens
    : 0;
  const hasOutputLoan = outputPoolPosition ? !outputPoolPosition.loanBalance.isZero() : false;
  // Orca pools
  const swapPool = useRecoilValue(CurrentSplSwapPool);
  const swapPoolTokenAmounts = useRecoilValue(SwapPoolTokenAmounts);
  const noOrcaPool = currentPool && outputToken && !swapPool;
  const swapFees = useRecoilValue(SwapFees);
  const [slippage, setSlippage] = useState(0.001);
  const [slippageInput, setSlippageInput] = useState('');
  const swapOutputTokens = getOutputTokenAmount(
    tokenInputAmount,
    swapPoolTokenAmounts?.source,
    swapPoolTokenAmounts?.destination,
    swapPool?.pool.swapType,
    swapFees,
    swapPool?.pool.amp ?? 1
  );
  const minOutAmount = getMinOutputAmount(
    tokenInputAmount,
    swapPoolTokenAmounts?.source,
    swapPoolTokenAmounts?.destination,
    swapPool?.pool.swapType,
    swapFees,
    slippage,
    0
  );
  // Exponents
  const expoSource = swapPoolTokenAmounts ? Math.pow(10, swapPoolTokenAmounts.source.decimals) : 0;
  const expoDestination = swapPoolTokenAmounts ? Math.pow(10, swapPoolTokenAmounts.destination.decimals) : 0;
  // Get the swap pool account balances
  const balanceSourceToken = swapPoolTokenAmounts ? swapPoolTokenAmounts.source.lamports.toNumber() : 0;
  const balanceDestinationToken = swapPoolTokenAmounts ? swapPoolTokenAmounts.destination.lamports.toNumber() : 0;
  const poolPrice = !swapPool
    ? 0.0
    : swapPool.pool.swapType === 'stable'
    ? !swapPool.inverted
      ? currentPool.tokenPrice / outputToken.tokenPrice
      : outputToken.tokenPrice / currentPool.tokenPrice
    : !swapPool.inverted
    ? balanceDestinationToken / expoDestination / (balanceSourceToken / expoSource)
    : balanceSourceToken / expoSource / (balanceDestinationToken / expoDestination);
  const swapPrice =
    !swapPool || !minOutAmount || minOutAmount.isZero() || !tokenInputAmount || tokenInputAmount.isZero()
      ? 0.0
      : !swapPool.inverted
      ? minOutAmount.lamports.toNumber() / expoDestination / (tokenInputAmount.lamports.toNumber() / expoSource)
      : tokenInputAmount.lamports.toNumber() / expoSource / (minOutAmount.lamports.toNumber() / expoDestination);
  const priceImpact = !swapPool
    ? 0.0
    : !swapPool.inverted
    ? (poolPrice - swapPrice) / poolPrice
    : (swapPrice - poolPrice) / poolPrice;
  const priceImpactStyle = priceImpact <= 0.01 ? 'success' : priceImpact <= 0.03 ? 'warning' : 'danger';
  const [repayLoanWithOutput, setRepayLoanWithOutput] = useState(false);
  // Swap / health feedback
  const riskStyle = useRiskStyle();
  const projectedRiskIndicator = useProjectedRisk(
    undefined,
    currentAccount,
    'swap',
    tokenInputAmount,
    swapOutputTokens,
    outputToken,
    repayLoanWithOutput
  );
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const swapReviewMessage = useSwapReviewMessage(
    currentAccount,
    currentPool,
    outputToken,
    swapPoolTokenAmounts?.source,
    swapPoolTokenAmounts?.destination,
    swapPool?.pool.swapType,
    swapFees,
    swapPool?.pool.amp ?? 1
  );
  const errorMessage = useTokenInputErrorMessage(undefined, projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const [switchingAssets, setSwitchingAssets] = useState(false);
  const disabled =
    sendingTransaction ||
    !currentPool ||
    !outputToken ||
    noOrcaPool ||
    projectedRiskIndicator >= 1 ||
    disabledMessage.length > 0;
  const { Paragraph, Text } = Typography;

  // Parse slippage input
  function getSlippageInput() {
    const slippage = parseFloat(slippageInput);
    if (!isNaN(slippage) && slippage > 0) {
      setSlippage(slippage / 100);
    }
  }
  useEffect(getSlippageInput, [setSlippage, slippageInput]);

  // Renders the user's collateral balance for input token
  function renderInputCollateralBalance() {
    let render = <></>;
    if (currentPool) {
      render = (
        <Paragraph
          onClick={() => {
            if (!disabled) {
              setTokenInputString(depositBalanceString);
            }
          }}
          className={
            !disabled ? 'token-balance' : 'secondary-text'
          }>{`${depositBalanceString} ${currentPool.symbol}`}</Paragraph>
      );
    }

    return render;
  }

  // Renders the user's overall balance for output token after current swap
  function renderAffectedBalance(side: 'input' | 'output') {
    let render = <></>;
    const amount = side === 'input' ? tokenInputAmount : swapOutputTokens;
    const overallBalance = side === 'input' ? overallInputBalance : overallOutputBalance;
    const precision = side === 'input' ? poolPrecision : outputPrecision;
    if (amount && !amount.isZero() && !currentAction) {
      const affectedBalance = side === 'input' ? overallBalance - amount.tokens : overallBalance + amount.tokens;
      render = (
        <div className="flex-centered">
          <ArrowRight />
          <Paragraph type={getTokenStyleType(affectedBalance)}>
            {currencyAbbrev(affectedBalance, false, undefined, precision)}
          </Paragraph>
        </div>
      );
    }

    return render;
  }

  // Render the user's risk level projection after the current swap
  function renderAffectedRiskLevel() {
    let render = <></>;
    if (swapOutputTokens && projectedRiskIndicator) {
      render = (
        <div className="flex-centered">
          <ArrowRight />
          <Paragraph type={projectedRiskStyle}>{formatRiskIndicator(projectedRiskIndicator)}</Paragraph>
        </div>
      );
    }

    return render;
  }

  // Render the user's price impact from the swap
  function renderPriceImpact() {
    let render = (
      <div className="flex-centered">
        <Paragraph type="success">0</Paragraph>
      </div>
    );
    if (swapOutputTokens) {
      render = (
        <div className="flex-centered">
          <Paragraph type={priceImpactStyle}>{formatPriceImpact(priceImpact)}</Paragraph>
        </div>
      );
    }

    return render;
  }

  // Returns text for the swap submit button
  function getSubmitText() {
    const inputText = currentPool?.symbol ?? '';
    const outputText = outputToken ? `${dictionary.actions.swap.for} ${outputToken.symbol}` : '';
    let text = `${dictionary.actions.swap.title} ${inputText} ${outputText}`;
    if (sendingTransaction) {
      text = dictionary.common.sending + '..';
    }

    return text;
  }

  // Swap
  async function sendSwap() {
    if (!currentPool || !outputToken || noOrcaPool || !swapPool) {
      return;
    }

    setSendingTransaction(true);
    const swapTitle = dictionary.actions.swap.title.toLowerCase();
    const [txId, resp] = await splTokenSwap(
      currentPool,
      outputToken,
      swapPool.pool,
      tokenInputAmount,
      minOutAmount,
      hasOutputLoan && repayLoanWithOutput
    );
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replaceAll('{{ACTION}}', swapTitle),
        dictionary.notifications.actions.successDescription
          .replaceAll('{{ACTION}}', swapTitle)
          .replaceAll('{{ASSET}}', currentPool.symbol)
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
      resetTokenInputString();
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.actions.cancelledTitle.replaceAll('{{ACTION}}', swapTitle),
        dictionary.notifications.actions.cancelledDescription
          .replaceAll('{{ACTION}}', swapTitle)
          .replaceAll('{{ASSET}}', currentPool.symbol)
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'warning'
      );
    } else if (resp === ActionResponse.Failed)
      if (resp === ActionResponse.Failed) {
        notify(
          dictionary.notifications.actions.failedTitle.replaceAll('{{ACTION}}', swapTitle),
          dictionary.notifications.actions.failedDescription
            .replaceAll('{{ACTION}}', swapTitle)
            .replaceAll('{{ASSET}}', currentPool.symbol)
            .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
          'error'
        );
      }
    setSendingTransaction(false);
  }

  // Disable repayLoanWithOutput if user has no loan to repay
  useEffect(() => {
    if (!hasOutputLoan) {
      setRepayLoanWithOutput(false);
    }
  }, [hasOutputLoan]);

  // Set initial outputToken, update selected swap pool
  useEffect(() => {
    if (!currentPool) {
      return;
    }

    const canFindOutput =
      !outputToken || currentPool.symbol === outputToken.symbol || currentPool.symbol === outputToken.symbol;
    if (pools && canFindOutput) {
      let output = Object.values(pools.tokenPools).filter(pool => {
        if (pool.symbol !== currentPool?.symbol && hasOrcaPool(cluster, currentPool.symbol, pool.symbol)) {
          return true;
        } else {
          return false;
        }
      })[0];
      if (!output) {
        output = Object.values(pools.tokenPools).filter(pool => pool.symbol !== currentPool?.symbol)[0];
      }
      setOutputToken(output);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool?.symbol, outputToken?.symbol]);

  return (
    <div className="order-entry swap-entry view-element flex column">
      <div className="order-entry-head flex column">
        <ReorderArrows component="swapEntry" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.swapsView.orderEntry.title}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body flex align-center justify-evenly column">
        <ConnectionFeedback />
        <div className="swap-tokens">
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.common.account.toUpperCase()}</Text>
            {renderInputCollateralBalance()}
          </div>
          <TokenInput
            action="swap"
            onPressEnter={sendSwap}
            hideSlider
            dropdownStyle={{ minWidth: 308, maxWidth: 308 }}
          />
          <Radio.Group
            className="flex-centered quick-fill-btns"
            value={tokenInputString}
            onChange={e => setTokenInputString(e.target.value)}
            style={{ marginTop: '-5px' }}>
            <Radio.Button
              className="small-btn"
              key="accountBalance"
              value={depositBalanceString !== '0' && depositBalanceString}
              disabled={depositBalanceString === '0' || sendingTransaction}>
              {dictionary.common.accountBalance}
            </Radio.Button>
            <Radio.Button
              className="small-btn"
              key="maxLeverage"
              value={maxSwapString !== '0' && maxSwapString}
              disabled={maxSwapString === '0' || sendingTransaction}>
              {dictionary.actions.swap.maxLeverage}
            </Radio.Button>
          </Radio.Group>
        </div>
        <div className="flex-centered" style={{ marginBottom: '-25px', marginTop: '-10px' }}>
          <Button
            className="function-btn swap-assets"
            shape="round"
            icon={<SwapIcon className="jet-icon" />}
            disabled={sendingTransaction || !outputToken}
            onClick={() => {
              if (outputToken) {
                const outputString = swapOutputTokens?.uiTokens ?? '0';
                setCurrentPoolSymbol(outputToken.symbol);
                setOutputToken(currentPool);
                // Allow UI to update and then adjust amounts
                setSwitchingAssets(true);
                setTimeout(() => {
                  setTokenInputString(outputString);
                  setSwitchingAssets(false);
                }, 500);
              }
            }}
          />
        </div>
        <div className="swap-tokens">
          <div className="swap-section-head flex align-center justify-start">
            <Text className="small-accent-text">{dictionary.actions.swap.receive.toUpperCase()}</Text>
          </div>
          <TokenInput
            poolSymbol={outputToken?.symbol}
            value={
              getOutputTokenAmount(
                tokenInputAmount,
                swapPoolTokenAmounts?.source,
                swapPoolTokenAmounts?.destination,
                swapPool?.pool.swapType,
                swapFees,
                swapPool?.pool.amp ?? 1
              ) ?? TokenAmount.zero(0)
            }
            tokenOptions={poolOptions.filter(pool => {
              if (
                pool.symbol !== currentPool?.symbol &&
                hasOrcaPool(cluster, currentPool?.symbol ?? '', pool.symbol ?? '')
              ) {
                return true;
              } else {
                return false;
              }
            })}
            onChangeToken={(tokenSymbol: string) => {
              // Set outputToken on token select
              if (pools) {
                const poolMatch = Object.values(pools.tokenPools).filter(pool => pool.symbol === tokenSymbol)[0];
                if (poolMatch) {
                  setOutputToken(poolMatch);
                }
              }
            }}
            onPressEnter={sendSwap}
            dropdownStyle={{ minWidth: 308, maxWidth: 308 }}
          />
        </div>
        <div className="swap-slippage flex column">
          <Info term="slippage">
            <Text className="small-accent-text info-element">{dictionary.actions.swap.slippage.toUpperCase()}</Text>
          </Info>
          <Radio.Group
            className="flex-centered slippage-btns"
            value={slippage}
            onChange={e => setSlippage(e.target.value)}>
            {[0.001, 0.005, 0.01].map(percentage => (
              <Radio.Button key={percentage} value={percentage} disabled={sendingTransaction}>
                {percentage * 100}%
              </Radio.Button>
            ))}
            <div
              className={`swap-slippage-input flex-centered ${
                (slippage * 100).toString() === slippageInput ? 'active' : ''
              }`}
              onClick={getSlippageInput}>
              <Input
                type="string"
                placeholder="0.75"
                value={slippageInput}
                disabled={sendingTransaction}
                onChange={e => {
                  let inputString = e.target.value;
                  if (isNaN(+inputString) || +inputString < 0) {
                    inputString = '0';
                  }
                  setSlippageInput(inputString);
                }}
                onPressEnter={sendSwap}
              />
              <Text type="secondary" strong>
                %
              </Text>
            </div>
          </Radio.Group>
        </div>
        {hasOutputLoan && (
          <div className="flex-centered repay-with-output" style={{ width: '100%' }}>
            <Checkbox
              onClick={() => setRepayLoanWithOutput(!repayLoanWithOutput)}
              disabled={!hasOutputLoan}
              checked={repayLoanWithOutput}>
              {dictionary.actions.swap.repayWithOutput
                .replace('{{ASSET}}', outputToken?.symbol ?? '')
                .replace('{{BALANCE}}', outputPoolPosition?.loanBalance.uiTokens ?? '')}
            </Checkbox>
          </div>
        )}
        <div className="order-entry-body-section-info flex-centered column">
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{`${currentPool?.symbol ?? '—'} ${dictionary.common.balance}`}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(overallInputBalance)}>
                {currencyAbbrev(overallInputBalance, false, undefined, poolPrecision)}
              </Paragraph>
              {renderAffectedBalance('input')}
            </div>
          </div>
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{`${outputToken?.symbol ?? '—'} ${dictionary.common.balance}`}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(overallOutputBalance)}>
                {currencyAbbrev(overallOutputBalance, false, undefined, outputPrecision)}
              </Paragraph>
              {renderAffectedBalance('output')}
            </div>
          </div>
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
              {renderAffectedRiskLevel()}
            </div>
          </div>
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.priceImpact}</Paragraph>
            {renderPriceImpact()}
          </div>
        </div>
        {noOrcaPool || errorMessage || swapReviewMessage.length ? (
          <div className="order-entry-body-section flex-centered">
            <Paragraph
              italic
              type={noOrcaPool || errorMessage.length ? 'danger' : undefined}
              className={`order-review ${
                noOrcaPool || errorMessage.length || swapReviewMessage.length ? '' : 'no-opacity'
              }`}>
              {noOrcaPool
                ? dictionary.actions.swap.errorMessages.noPools
                : errorMessage.length
                ? errorMessage
                : swapReviewMessage}
            </Paragraph>
          </div>
        ) : (
          <></>
        )}
        {!tokenInputAmount.isZero() && priceImpact && priceImpact >= 0.05 ? (
          <div className="order-entry-body-section flex-centered">
            <Paragraph italic type={'danger'} className={'order-review'}>
              {dictionary.actions.swap.warningMessages.largePriceImpact}
            </Paragraph>
          </div>
        ) : (
          <></>
        )}
      </div>
      <div className="order-entry-footer flex-centered">
        <Button
          block
          disabled={disabled || tokenInputAmount.isZero() || priceImpact >= 0.05}
          loading={sendingTransaction}
          onClick={sendSwap}
          style={sendingTransaction ? { zIndex: 1002 } : undefined}>
          {getSubmitText()}
        </Button>
      </div>
      <div className={`action-modal-overlay ${sendingTransaction || switchingAssets ? 'showing' : ''}`}></div>
    </div>
  );
}
