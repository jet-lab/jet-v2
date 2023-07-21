import { useEffect, useMemo, useState } from 'react';
import { useRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { SwapsRowOrder } from '@state/views/views';
import { Dictionary } from '@state/settings/localization/localization';
import { CurrentAccount } from '@state/user/accounts';
import { Pools, PoolOptions } from '@state/pools/pools';
import {
  ActionRefresh,
  CurrentAction,
  CurrentSwapOutput,
  SendingTransaction,
  TokenInputAmount,
  TokenInputString
} from '@state/actions/actions';
import { useProjectedRisk, useRiskStyle } from '@utils/risk';
import { formatPriceImpact, formatRiskIndicator } from '@utils/format';
import { notify } from '@utils/notify';
import { getExplorerUrl, getTokenStyleType } from '@utils/ui';
import { DEFAULT_DECIMALS, useCurrencyFormatting } from '@utils/currency';
import { SwapQuote, SwapStep, getSwapRoutes } from '@utils/actions/swap';
import { ActionResponse, useMarginActions } from '@utils/jet/marginActions';
import { Info } from '@components/misc/Info';
import { TokenInput } from '@components/misc/TokenInput/TokenInput';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { ConnectionFeedback } from '@components/misc/ConnectionFeedback/ConnectionFeedback';
import { ArrowRight } from '@components/modals/actions/ArrowRight';
import { Button, Checkbox, Input, Radio, Typography } from 'antd';
import SwapIcon from '@assets/icons/function-swap.svg';
import { useTokenInputDisabledMessage, useTokenInputErrorMessage } from '@utils/actions/tokenInput';
import debounce from 'lodash.debounce';
import { useJetStore } from '@jet-lab/store';
import BN from 'bn.js';

// Component for user to enter and submit a swap action
export function SwapEntry(): JSX.Element {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { routeSwap } = useMarginActions();
  // Margin account
  const currentAction = useRecoilValue(CurrentAction);
  const currentAccount = useRecoilValue(CurrentAccount);
  // Pools
  const pools = useRecoilValue(Pools);
  const poolOptions = useRecoilValue(PoolOptions);
  // Input token pool
  const {
    prices,
    selectedPoolKey,
    selectPool,
    airspaceLookupTables,
    marginAccountLookupTables,
    selectedMarginAccount
  } = useJetStore(state => ({
    prices: state.prices,
    selectedPoolKey: state.selectedPoolKey,
    selectPool: state.selectPool,
    airspaceLookupTables: state.airspaceLookupTables,
    marginAccountLookupTables: state.marginAccountLookupTables,
    selectedMarginAccount: state.selectedMarginAccount
  }));
  const lookupTables = useMemo(() => {
    if (!selectedMarginAccount) {
      return airspaceLookupTables;
    } else {
      return marginAccountLookupTables[selectedMarginAccount]?.length
        ? airspaceLookupTables.concat(marginAccountLookupTables[selectedMarginAccount])
        : airspaceLookupTables;
    }
  }, [selectedMarginAccount, airspaceLookupTables, marginAccountLookupTables]);
  const currentPool = useMemo(
    () =>
      pools?.tokenPools && Object.values(pools?.tokenPools).find(pool => pool.address.toBase58() === selectedPoolKey),
    [selectedPoolKey, pools]
  );

  const swapEndpoint =
    cluster === 'mainnet-beta'
      ? process.env.REACT_APP_SWAP_API
      : cluster === 'devnet'
        ? process.env.REACT_APP_DEV_SWAP_API
        : process.env.REACT_APP_LOCAL_SWAP_API;

  const poolPrecision = currentPool?.precision ?? DEFAULT_DECIMALS;
  const poolPosition = currentAccount && currentPool && currentAccount.poolPositions[currentPool.symbol];
  const overallInputBalance = poolPosition ? poolPosition.depositBalance.tokens - poolPosition.loanBalance.tokens : 0;
  const depositBalanceString = poolPosition ? poolPosition.depositBalance.uiTokens : '0';
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
  const actionRefresh = useRecoilValue(ActionRefresh);
  const [swapQuotes, setSwapQuotes] = useState<SwapQuote[]>([]);
  const [selectedSwapQuote, setSelectedSwapQuote] = useState<SwapQuote | undefined>(undefined);
  const [slippage, setSlippage] = useState(0.001);
  const [slippageInput, setSlippageInput] = useState('');
  const [swapOutputTokens, setSwapOutputTokens] = useState(TokenAmount.zero(0));
  const [minOutAmount, setMinOutAmount] = useState(TokenAmount.zero(0));
  const priceImpact = !selectedSwapQuote ? 0.0 : selectedSwapQuote.price_impact;
  const [swapFeeUsd, setSwapFeeUsd] = useState(0.0);
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
  const errorMessage = useTokenInputErrorMessage(undefined, projectedRiskIndicator);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const [switchingAssets, setSwitchingAssets] = useState(false);
  const disabled =
    sendingTransaction || !currentPool || !outputToken || projectedRiskIndicator >= 1 || disabledMessage.length > 0;
  const { Paragraph, Text } = Typography;

  // Parse slippage input
  function getSlippageInput() {
    const slippage = parseFloat(slippageInput);
    if (!isNaN(slippage) && slippage > 0) {
      setSlippage(slippage / 100);
    }
  }
  useEffect(getSlippageInput, [setSlippage, slippageInput]);

  // Get swap quote when token inputs change or on a timer
  useEffect(() => {
    if (!currentPool || !outputToken) {
      return;
    }
    // If the token input is 0, don't send a request
    if (tokenInputAmount.isZero()) {
      return;
    }
    async function getSwapTokenPrices() {
      if (!currentPool || !outputToken || !pools) {
        return;
      }
      try {
        const routes = await getSwapRoutes(
          swapEndpoint || '',
          currentPool.tokenMint,
          outputToken.tokenMint,
          tokenInputAmount
        );
        if (!routes) {
          return;
        }
        setSwapQuotes(routes);
        if (!routes.length) {
          setSwapOutputTokens(TokenAmount.zero(0));
          setMinOutAmount(TokenAmount.zero(0));
          setSwapFeeUsd(0.0);
          return;
        }
        // TODO: assume that the user will take the cheapest route always
        setSelectedSwapQuote(swapQuotes[0]);
        const selectedQuote = swapQuotes[0];
        console.log((selectedQuote.swaps[0][0] as any)?.program)
        console.log(selectedQuote.tokens_out, outputToken.decimals)
        setSwapOutputTokens(new TokenAmount(new BN(selectedQuote.tokens_out), outputToken.decimals));
        setMinOutAmount(
          new TokenAmount(new BN(Math.round(selectedQuote.tokens_out * (1 - slippage))), outputToken.decimals)
        );
        let totalFee = 0.0;
        for (let fee of Object.keys(selectedQuote.fees)) {
          const price = prices && prices[fee].price;
          const swapFee = selectedQuote.fees[fee];
          const pool = Object.values(pools.tokenPools).find(p => {
            return p.tokenMint.toBase58() === fee;
          });
          let decimals = pool && -pool.decimals;
          if (price) {
            // TODO: hardcoded as a common decimal length
            totalFee += price * swapFee * Math.pow(10, decimals || -6);
          }
        }
        console.log('swap fee', totalFee);
        setSwapFeeUsd(totalFee);
      } catch (err) {
        console.error(err);
      }
    }

    getSwapTokenPrices();

  }, [
    actionRefresh,
    currentPool?.symbol,
    outputToken?.symbol,
    tokenInputAmount,
    swapEndpoint,
    pools,
    slippage,
  ])

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
            {currencyAbbrev(affectedBalance, precision, false, undefined)}
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

  function renderSwapFee() {
    let render = (
      <div className="flex-centered">
        <Paragraph type="success">$0</Paragraph>
      </div>
    );
    if (swapOutputTokens) {
      render = (
        <div className="flex-centered">
          <Paragraph>{currencyAbbrev(swapFeeUsd, 2, true, undefined, true, true)}</Paragraph>
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
    if (!currentPool || !outputToken) {
      return;
    }

    setSendingTransaction(true);
    const swapTitle = dictionary.actions.swap.title.toLowerCase();
    const [txId, resp] = await routeSwap(
      currentPool,
      outputToken,
      swapQuotes[0].swaps.map(step => step[0] as SwapStep), // TODO: must not be empty
      tokenInputAmount,
      minOutAmount,
      hasOutputLoan && repayLoanWithOutput,
      lookupTables
    );
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replaceAll('{{ACTION}}', swapTitle),
        dictionary.notifications.actions.successDescription
          .replaceAll('{{ACTION}}', swapTitle)
          .replaceAll('{{ASSET}}', currentPool.symbol)
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'success',
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
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
          'error',
          txId ? getExplorerUrl(txId, cluster, explorer) : undefined
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
        if (pool.symbol !== currentPool?.symbol) {
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
    <div className="order-entry swap-entry view-element column flex">
      <div className="order-entry-head column flex">
        <ReorderArrows component="swapEntry" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.swapsView.orderEntry.title}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body align-center column flex justify-evenly">
        <ConnectionFeedback />
        <div className="swap-tokens">
          <div className="swap-section-head align-center flex justify-between">
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
            onChange={debounce(e => {
              setTokenInputString(e.target.value);
            }, 300)}
            style={{ marginTop: '-5px' }}>
            <Radio.Button
              className="small-btn"
              key="accountBalance"
              value={depositBalanceString !== '0' && depositBalanceString}
              disabled={depositBalanceString === '0' || sendingTransaction}>
              {dictionary.common.accountBalance}
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
                selectPool(outputToken.address.toBase58());
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
          <div className="swap-section-head align-center flex justify-start">
            <Text className="small-accent-text">{dictionary.actions.swap.receive.toUpperCase()}</Text>
          </div>
          <TokenInput
            poolSymbol={outputToken?.symbol}
            value={swapOutputTokens}
            tokenOptions={poolOptions.filter(pool => {
              if (pool.symbol !== currentPool?.symbol) {
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
        <div className="swap-slippage column flex">
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
              className={`swap-slippage-input flex-centered ${(slippage * 100).toString() === slippageInput ? 'active' : ''
                }`}
              onClick={getSlippageInput}>
              <Input
                type="string"
                placeholder="0.75"
                value={slippageInput}
                disabled={sendingTransaction}
                onChange={debounce(e => {
                  let inputString = e.target.value;
                  if (isNaN(+inputString) || +inputString < 0) {
                    inputString = '0';
                  }
                  setSlippageInput(inputString);
                }, 300)}
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
          <div className="order-entry-body-section-info-item align-center flex justify-between">
            <Paragraph type="secondary">{`${currentPool?.symbol ?? '—'} ${dictionary.common.balance}`}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(overallInputBalance)}>
                {currencyAbbrev(overallInputBalance, poolPrecision, false, undefined)}
              </Paragraph>
              {renderAffectedBalance('input')}
            </div>
          </div>
          <div className="order-entry-body-section-info-item align-center flex justify-between">
            <Paragraph type="secondary">{`${outputToken?.symbol ?? '—'} ${dictionary.common.balance}`}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={getTokenStyleType(overallOutputBalance)}>
                {currencyAbbrev(overallOutputBalance, outputPrecision, false, undefined)}
              </Paragraph>
              {renderAffectedBalance('output')}
            </div>
          </div>
          <div className="order-entry-body-section-info-item align-center flex justify-between">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
              {renderAffectedRiskLevel()}
            </div>
          </div>
          <div className="order-entry-body-section-info-item align-center flex justify-between">
            <Paragraph type="secondary">{dictionary.common.priceImpact}</Paragraph>
            {renderPriceImpact()}
          </div>
          <div className="order-entry-body-section-info-item align-center flex justify-between">
            <Paragraph type="secondary">{dictionary.common.swapFee}</Paragraph>
            {renderSwapFee()}
          </div>
        </div>
        {errorMessage ? (
          <div className="order-entry-body-section flex-centered">
            <Paragraph
              italic
              type={errorMessage.length ? 'danger' : undefined}
              className={`order-review ${errorMessage.length ? '' : 'no-opacity'}`}>
              {errorMessage}
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
