import { useEffect, useState } from 'react';
import { useRecoilState, useResetRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { SwapsRowOrder } from '../../state/views/views';
import { BlockExplorer, Cluster } from '../../state/settings/settings';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentAccount } from '../../state/user/accounts';
import { CurrentPoolSymbol, Pools, CurrentPool } from '../../state/borrow/pools';
import { CurrentMarketPair } from '../../state/trade/market';
import { CurrentSwapOutput, TokenInputAmount, TokenInputString } from '../../state/actions/actions';
import { useProjectedRisk, useRiskStyle } from '../../utils/risk';
import { formatRiskIndicator } from '../../utils/format';
import { notify } from '../../utils/notify';
import { getExplorerUrl } from '../../utils/ui';
import { getTokenAmountFromNumber, useCurrencyFormatting } from '../../utils/currency';
import { getMinOutputAmount, getOutputTokenAmount, useSwapReviewMessage } from '../../utils/actions/swap';
import { ActionResponse, useMarginActions } from '../../utils/jet/marginActions';
import { Button, Divider, Input, Radio, Typography } from 'antd';
import { Info } from '../misc/Info';
import { TokenInput } from '../misc/TokenInput/TokenInput';
import { ReorderArrows } from '../misc/ReorderArrows';
import { ConnectionFeedback } from '../misc/ConnectionFeedback';
import { ReactComponent as SwapIcon } from '../../styles/icons/function-swap.svg';

export function SwapEntry(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(SwapsRowOrder);
  const { currencyAbbrev } = useCurrencyFormatting();
  const { swap } = useMarginActions();
  const currentAccount = useRecoilValue(CurrentAccount);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const currentPool = useRecoilValue(CurrentPool);
  const pools = useRecoilValue(Pools);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  const [outputToken, setOutputToken] = useRecoilState(CurrentSwapOutput);
  const [slippage, setSlippage] = useState(0.005);
  const [slippageInput, setSlippageInput] = useState('');
  const riskStyle = useRiskStyle();
  const currentAccountPoolPosition =
    currentPool && currentAccount ? currentAccount.poolPositions[currentPool.symbol] : undefined;
  const projectedRiskIndicator = useProjectedRisk(undefined, currentAccount, 'borrow', getMarginInputAmount());
  const projectedRiskStyle = useRiskStyle(projectedRiskIndicator);
  const swapReviewMessage = useSwapReviewMessage(currentAccount, currentPool, outputToken);
  const [sendingSwap, setSendingSwap] = useState(false);
  const { Paragraph, Text } = Typography;

  // Calculate marginInputAmount
  function getMarginInputAmount() {
    let marginInputAmount = TokenAmount.zero(currentPool?.decimals ?? 6);
    if (currentAccountPoolPosition && currentAccountPoolPosition.depositBalance.tokens < tokenInputAmount.tokens) {
      marginInputAmount = getTokenAmountFromNumber(
        tokenInputAmount.tokens - currentAccountPoolPosition.depositBalance.tokens,
        currentPool?.decimals ?? 6
      );
    }

    return marginInputAmount;
  }

  // Parse slippage input
  function getSlippageInput() {
    const slippage = parseFloat(slippageInput);
    if (!isNaN(slippage) && slippage > 0) {
      setSlippage(slippage / 100);
    }
  }
  useEffect(getSlippageInput, [slippageInput]);

  // Swap
  async function sendSwap() {
    if (!currentPool || !outputToken) {
      return;
    }

    setSendingSwap(true);
    const swapTitle = dictionary.actions.swap.title.toLowerCase();
    const minOutAmount = getMinOutputAmount(tokenInputAmount, currentPool, outputToken, slippage);
    const [txId, resp] = await swap(currentPool, outputToken, tokenInputAmount, minOutAmount);
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
    } else {
      notify(
        dictionary.notifications.actions.failedTitle.replaceAll('{{ACTION}}', swapTitle),
        dictionary.notifications.actions.failedDescription
          .replaceAll('{{ACTION}}', swapTitle)
          .replaceAll('{{ASSET}}', currentPool.symbol)
          .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens),
        'error'
      );
    }
    setSendingSwap(false);
  }

  // Set initial outputToken
  useEffect(() => {
    if (pools && !outputToken) {
      const output = Object.values(pools.tokenPools).filter(pool => pool.symbol !== currentPool?.symbol)[0];
      setOutputToken(output);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPool?.symbol, outputToken]);

  return (
    <div className="order-entry swap-panel view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="swapEntry" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.swapsView.orderEntry.title}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body view-element-item view-element-item-hidden">
        <ConnectionFeedback />
        <div className="swap-tokens">
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.title.toUpperCase()}</Text>
            {currentPool && (
              <Paragraph type="secondary" italic>{`${
                currentPool && currentAccount
                  ? currentAccount.poolPositions[currentPool.symbol as string]?.depositBalance.tokens
                  : 0
              } ${currentPool ? currentPool.symbol : '—'}`}</Paragraph>
            )}
          </div>
          <TokenInput
            account={currentAccount}
            onChangeToken={(tokenSymbol: string) => {
              setCurrentPoolSymbol(tokenSymbol);
              if (tokenSymbol !== 'USDC') {
                setCurrentMarketPair(`${tokenSymbol}/USDC`);
              }
            }}
            dropdownStyle={{ minWidth: 308 }}
            action="swap"
            onPressEnter={sendSwap}
            loading={sendingSwap}
          />
          <div className="flex-centered">
            <Button
              className="function-btn swap-assets"
              shape="round"
              icon={<SwapIcon className="jet-icon" />}
              disabled={sendingSwap || !outputToken}
              onClick={() => {
                if (outputToken) {
                  resetTokenInputString();
                  setCurrentPoolSymbol(outputToken.symbol);
                  setOutputToken(currentPool);
                }
              }}
            />
          </div>
          <div className="swap-section-head flex align-center justify-between">
            <Text className="small-accent-text">{dictionary.actions.swap.for.toUpperCase()}</Text>
            {currentPool && (
              <Paragraph type="secondary" italic>{`${
                outputToken && currentAccount
                  ? currentAccount.poolPositions[outputToken.symbol as string]?.depositBalance.tokens ?? 0
                  : 0
              } ${outputToken ? outputToken.symbol : '—'}`}</Paragraph>
            )}
          </div>
          <TokenInput
            account={currentAccount}
            tokenSymbol={outputToken?.symbol}
            onChangeToken={(tokenSymbol: string) => {
              if (!pools) {
                return;
              }

              const poolMatch = Object.values(pools.tokenPools).filter(pool => pool.symbol === tokenSymbol)[0];
              if (poolMatch) {
                setOutputToken(poolMatch);
              }
            }}
            dropdownStyle={{ minWidth: 308 }}
            tokenValue={getOutputTokenAmount(tokenInputAmount, currentPool, outputToken) ?? TokenAmount.zero(0)}
            onPressEnter={sendSwap}
            loading={sendingSwap}
          />
        </div>
        <Divider />
        <div className="swap-slippage flex column">
          <Info term="slippage">
            <Text className="small-accent-text info-element">{dictionary.actions.swap.slippage.toUpperCase()}</Text>
          </Info>
          <Radio.Group className="flex-centered" value={slippage} onChange={e => setSlippage(e.target.value)}>
            {[0.001, 0.005, 0.01].map(percentage => (
              <Radio.Button key={percentage} value={percentage} disabled={sendingSwap}>
                {percentage * 100}%
              </Radio.Button>
            ))}
            <div
              className={`swap-slippage-input flex-centered ${slippage.toString() === slippageInput ? 'active' : ''}`}
              onClick={getSlippageInput}>
              <Input
                type="string"
                placeholder="0.10"
                value={slippageInput}
                disabled={sendingSwap}
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
        <div className="order-entry-body-section-info flex-centered column">
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="danger">{dictionary.common.loanBalance}</Paragraph>
            <Paragraph type="danger">
              {currencyAbbrev(
                currentAccountPoolPosition?.loanBalance.tokens ?? 0,
                false,
                undefined,
                (currentPool?.decimals ?? 6) / 2
              )}
              {!getMarginInputAmount().isZero() && currentAccountPoolPosition ? (
                <>
                  &nbsp;&#8594;&nbsp;
                  {currencyAbbrev(
                    currentAccountPoolPosition.loanBalance.tokens + getMarginInputAmount().tokens,
                    false,
                    undefined,
                    (currentPool?.decimals ?? 6) / 2
                  )}
                </>
              ) : (
                <></>
              )}
              {' ' + (currentPool?.symbol ?? '—')}
            </Paragraph>
          </div>
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
            <div className="flex-centered">
              <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
              {!getMarginInputAmount().isZero() ? (
                <Paragraph type={projectedRiskStyle}>
                  &nbsp;&#8594;&nbsp;{formatRiskIndicator(projectedRiskIndicator)}
                </Paragraph>
              ) : (
                <></>
              )}
            </div>
          </div>
          <div className="order-entry-body-section-info-item flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.actions.swap.minimumRecieved}</Paragraph>
            <Paragraph>
              {getMinOutputAmount(tokenInputAmount, currentPool, outputToken, slippage).uiTokens} {outputToken?.symbol}
            </Paragraph>
          </div>
        </div>
        <div className="order-entry-body-section flex-centered">
          <Paragraph italic className={`order-review ${swapReviewMessage.length ? '' : 'no-opacity'}`}>
            {swapReviewMessage}
          </Paragraph>
        </div>
      </div>
      <div className="order-entry-footer view-element-item view-element-item-hidden flex-centered">
        <Button
          block
          disabled={sendingSwap || tokenInputAmount.isZero() || !currentPool || !outputToken}
          loading={!tokenInputAmount.isZero() && sendingSwap}
          onClick={sendSwap}>
          {sendingSwap ? dictionary.common.sending + '..' : dictionary.actions.swap.title}
        </Button>
      </div>
    </div>
  );
}
