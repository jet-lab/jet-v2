import { useEffect, useState } from 'react';
import { useRecoilState, useSetRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import { OrderSide as MarginOrderSide, TokenAmount } from '@jet-lab/margin';
import { Dictionary } from '../../../state/settings/localization/localization';
import { BlockExplorer, Cluster } from '../../../state/settings/settings';
import { TradeRowOrder } from '../../../state/views/views';
import { CurrentPool } from '../../../state/borrow/pools';
import {
  OrderSide,
  orderTypes,
  OrderType,
  OrderPrice,
  OrderAmount,
  OrderSize,
  OrderPriceString,
  OrderAmountString,
  OrderSizeString
} from '../../../state/trade/order';
import { MarketsInit, CurrentMarket, CurrentMarketPair } from '../../../state/trade/market';
import { AccountsInit, CurrentAccount } from '../../../state/user/accounts';
import { ActionResponse, useMarginActions } from '../../../utils/jet/marginActions';
import { getTokenAmountFromNumber, useCurrencyFormatting } from '../../../utils/currency';
import { useOrderReviewMessage } from '../../../utils/trade/order';
import { getExplorerUrl } from '../../../utils/ui';
import { notify } from '../../../utils/notify';
import { formatRiskIndicator } from '../../../utils/format';
import { useRiskStyle } from '../../../utils/risk';
import { Button, Select, Slider, Tabs, Typography } from 'antd';
import { ReorderArrows } from '../../misc/ReorderArrows';
import { OrderInput } from './OrderInput';
import { UpOutlined, DownOutlined } from '@ant-design/icons';
import { ConnectionFeedback } from '../../misc/ConnectionFeedback';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';

export function OrderEntry(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const [tradeRowOrder, setTradeRowOrder] = useRecoilState(TradeRowOrder);
  const { currencyAbbrev } = useCurrencyFormatting();
  const currentPool = useRecoilValue(CurrentPool);
  const { placeOrder } = useMarginActions();
  const marketsInit = useRecoilValue(MarketsInit);
  const currentMarketPair = useRecoilValue(CurrentMarketPair);
  const currentMarket = useRecoilValue(CurrentMarket);
  const accountsInit = useRecoilValue(AccountsInit);
  const currentAccount = useRecoilValue(CurrentAccount);
  const baseAssetPoolPosition = currentMarket && currentAccount?.poolPositions[currentMarket.baseSymbol];
  const quoteAssetPoolPosition = currentMarket && currentAccount?.poolPositions[currentMarket.quoteSymbol];
  const [orderSide, setOrderSide] = useRecoilState(OrderSide);
  const [orderType, setOrderType] = useRecoilState(OrderType);
  const orderPrice = useRecoilValue(OrderPrice);
  const setOrderPriceString = useSetRecoilState(OrderPriceString);
  const resetOrderPriceString = useResetRecoilState(OrderPriceString);
  const [orderAmount, setOrderAmount] = useRecoilState(OrderAmount);
  const setOrderAmountString = useSetRecoilState(OrderAmountString);
  const resetOrderAmountString = useResetRecoilState(OrderAmountString);
  const [orderSize, setOrderSize] = useRecoilState(OrderSize);
  const setOrderSizeString = useSetRecoilState(OrderSizeString);
  const resetOrderSizeString = useResetRecoilState(OrderSizeString);
  const zeroTokenAmount = TokenAmount.zero(currentMarket?.quoteDecimals ?? 6);
  const riskStyle = useRiskStyle();
  const orderReviewMessage = useOrderReviewMessage();
  const [sendingOrder, setSendingOrder] = useState(false);
  const [disabled, setDisabled] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');
  const { Paragraph, Text } = Typography;
  const { TabPane } = Tabs;
  const { Option } = Select;

  // Buying
  const maxMarginBuy =
    currentAccount && currentMarket && currentPool
      ? currentAccount.poolPositions[currentPool.symbol].maxTradeAmounts.borrow
      : zeroTokenAmount;
  const maxOrderSize = getTokenAmountFromNumber(
    maxMarginBuy.tokens + (quoteAssetPoolPosition?.depositBalance ?? zeroTokenAmount).tokens,
    maxMarginBuy.decimals
  );
  const marginDifferenceLong = getTokenAmountFromNumber(
    orderSize.tokens - (quoteAssetPoolPosition?.depositBalance ?? zeroTokenAmount).tokens,
    currentMarket?.quoteDecimals ?? 6
  );
  const projectedRiskIndicatorLong =
    currentPool && currentAccount && !marginDifferenceLong.isZero()
      ? currentPool.projectAfterBorrow(currentAccount, marginDifferenceLong.tokens).riskIndicator
      : currentAccount?.riskIndicator ?? 0;
  const projectedRiskStyleLong = useRiskStyle(projectedRiskIndicatorLong);

  // Selling
  const maxShortSale =
    currentAccount && currentMarket && currentPool?.symbol
      ? currentAccount.poolPositions[currentPool.symbol].maxTradeAmounts.borrow
      : zeroTokenAmount;
  const maxOrderAmount = getTokenAmountFromNumber(
    maxShortSale.tokens + (baseAssetPoolPosition?.depositBalance ?? zeroTokenAmount).tokens,
    maxShortSale.decimals
  );
  const marginDifferenceShort = baseAssetPoolPosition
    ? getTokenAmountFromNumber(
        orderAmount.tokens - baseAssetPoolPosition.depositBalance.tokens,
        currentMarket.baseDecimals
      )
    : zeroTokenAmount;
  const projectedRiskIndicatorShort =
    currentPool && currentAccount && !marginDifferenceShort.isZero()
      ? currentPool.projectAfterBorrow(currentAccount, marginDifferenceShort.tokens).riskIndicator
      : currentAccount?.riskIndicator ?? 0;
  const projectedRiskStyleShort = useRiskStyle(projectedRiskIndicatorShort);

  // Submit trade order
  async function sumbitOrder() {
    if (disabled || !orderPrice || !orderAmount) {
      return;
    }

    setSendingOrder(true);
    const [txId, resp] = await placeOrder();
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.orders.successTitle,
        dictionary.notifications.orders.successDescription
          .replaceAll('{{TYPE}}', orderType)
          .replaceAll('{{SIDE}}', orderSide)
          .replaceAll('{{AMOUNT}}', orderAmount?.uiTokens ?? '')
          .replaceAll('{{BASE}}', currentMarket?.baseSymbol ?? '')
          .replaceAll('{{PRICE}}', orderPrice?.toString() ?? '')
          .replaceAll('{{QUOTE}}', currentMarket?.quoteSymbol ?? ''),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
      resetOrderPriceString();
      resetOrderAmountString();
      resetOrderSizeString();
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.orders.cancelledTitle,
        dictionary.notifications.orders.cancelledDescription
          .replaceAll('{{TYPE}}', orderType)
          .replaceAll('{{AMOUNT}}', orderAmount?.uiTokens ?? '')
          .replaceAll('{{BASE}}', currentMarket?.baseSymbol ?? ''),
        'warning'
      );
      setSendingOrder(false);
    } else {
      notify(
        dictionary.notifications.orders.failedTitle,
        dictionary.notifications.orders.failedDescription
          .replaceAll('{{TYPE}}', orderType)
          .replaceAll('{{AMOUNT}}', orderAmount?.uiTokens ?? '')
          .replaceAll('{{BASE}}', currentMarket?.baseSymbol ?? ''),
        'error'
      );
      setSendingOrder(false);
    }
  }

  // Check for error message on input changes
  useEffect(() => {
    setErrorMessage('');
    if ((orderSide === 'sell' && orderAmount.gt(maxOrderAmount)) || orderSize.gt(maxOrderSize)) {
      setErrorMessage(
        dictionary.actions.trade.errorMessages.overMaxMargin
          .replaceAll('{{MARGIN_AMOUNT}}', orderSide === 'sell' ? maxOrderAmount.uiTokens : maxOrderSize.uiTokens)
          .replaceAll(
            '{{MARGIN_ASSET}}',
            (orderSide === 'sell' ? currentMarket?.baseSymbol : currentMarket?.quoteSymbol) ?? ''
          )
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [orderSide, orderAmount, maxOrderAmount, orderSize, maxOrderSize]);

  // Disable order entry components
  useEffect(() => {
    if (sendingOrder || !marketsInit || !accountsInit) {
      setDisabled(true);
    } else {
      setDisabled(false);
    }
  }, [sendingOrder, marketsInit, accountsInit]);

  // Reset when switching currentMarketPair or orderSide
  useEffect(() => {
    resetOrderAmountString();
    resetOrderSizeString();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentMarketPair, orderSide]);

  return (
    <div className="order-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="orderEntry" order={tradeRowOrder} setOrder={setTradeRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.tradeView.orderEntry.title}</Paragraph>
        </div>
        <div className="order-entry-head-bottom flex-centered">
          <Tabs
            className={orderSide === 'sell' ? 'danger' : ''}
            defaultActiveKey={orderSide}
            onChange={(side: string) => setOrderSide(side as MarginOrderSide)}>
            {['buy', 'sell'].map(side => (
              <TabPane key={side} tab={`${side} ${currentMarket?.baseSymbol ?? '—'}`}></TabPane>
            ))}
          </Tabs>
        </div>
      </div>
      <div className="order-entry-body view-element-item view-element-item-hidden">
        <ConnectionFeedback />
        <div className="order-entry-body-section flex align-start justify-between">
          <div className="order-entry-body-section-price flex column">
            <Text className="small-accent-text">{dictionary.common.price}</Text>
            <div className="flex-centered">
              <OrderInput
                inputType="price"
                maxOrderAmount={maxOrderAmount}
                maxOrderSize={maxOrderSize}
                disabled={disabled || orderType === 'ioc'}
                onPressEnter={sumbitOrder}
              />
              <div className={`tick-arrows flex-centered column ${disabled || orderType === 'ioc' ? 'disabled' : ''}`}>
                <UpOutlined
                  onClick={() => {
                    if (currentMarket && !(disabled || orderType === 'ioc')) {
                      setOrderPriceString(
                        getTokenAmountFromNumber(
                          (orderPrice ?? 0) + currentMarket.serum.tickSize,
                          currentMarket.quoteDecimals
                        ).uiTokens
                      );
                    }
                  }}
                />
                <DownOutlined
                  onClick={() => {
                    if (currentMarket && !(disabled || orderType === 'ioc') && orderPrice > 0) {
                      setOrderPriceString(
                        getTokenAmountFromNumber(
                          (orderPrice ?? 0) - currentMarket.serum.tickSize,
                          currentMarket.quoteDecimals
                        ).uiTokens
                      );
                    }
                  }}
                />
              </div>
            </div>
          </div>
          <div className="order-entry-body-section-type flex align-start column">
            <Text className="small-accent-text">{dictionary.tradeView.orderEntry.type}</Text>
            <Select
              value={orderType}
              disabled={disabled}
              suffixIcon={<AngleDown className="jet-icon" />}
              onChange={type => setOrderType(type)}>
              {orderTypes.map(type => (
                <Option key={dictionary.tradeView.orderEntry[type]} value={type}>
                  {dictionary.tradeView.orderEntry[type]}
                </Option>
              ))}
            </Select>
          </div>
        </div>
        <div className="order-entry-body-section flex column">
          <div className="flex align-start justify-between">
            <div className="order-entry-body-section-amount flex column">
              <Text className="small-accent-text">{dictionary.common.amount}</Text>
              <OrderInput
                inputType="amount"
                maxOrderAmount={maxOrderAmount}
                maxOrderSize={maxOrderSize}
                disabled={disabled}
                onPressEnter={sumbitOrder}
              />
            </div>
            <div className="order-entry-body-section-amount flex align-start column">
              <Text className="small-accent-text">{dictionary.common.size}</Text>
              <OrderInput
                inputType="size"
                maxOrderAmount={maxOrderAmount}
                maxOrderSize={maxOrderSize}
                disabled={disabled}
                onPressEnter={sumbitOrder}
              />
            </div>
          </div>
          <Slider
            value={
              (orderSide === 'sell'
                ? orderAmount.tokens / maxOrderAmount.tokens
                : orderSize.tokens / maxOrderSize.tokens) * 100
            }
            min={0}
            max={100}
            step={1}
            disabled={disabled || (orderSide === 'sell' ? maxOrderAmount.isZero() : maxOrderSize.isZero())}
            onChange={percent => {
              if (!currentMarket) {
                return;
              }

              const maxInput = orderSide === 'sell' ? maxOrderAmount : maxOrderSize;
              const percentageAmount = maxInput.tokens * (percent / 100);
              const percentTokenAmount = getTokenAmountFromNumber(percentageAmount, currentMarket.baseDecimals);
              if (orderSide === 'sell') {
                setOrderAmount(percentTokenAmount);
                setOrderAmountString(percentTokenAmount.uiTokens);
                if (orderPrice > 0) {
                  const orderSize = percentTokenAmount.muln(orderPrice);
                  setOrderSizeString(orderSize.uiTokens);
                  setOrderSize(orderSize);
                }
              } else {
                setOrderSize(percentTokenAmount);
                setOrderSizeString(percentTokenAmount.uiTokens);
                if (orderPrice > 0) {
                  const orderAmount = percentTokenAmount.divn(orderPrice);
                  setOrderAmount(orderAmount);
                  setOrderAmountString(orderAmount.uiTokens);
                }
              }
            }}
            tipFormatter={value => value + '%'}
            tooltipPlacement="bottom"
            marks={{
              0: '0%',
              25: '25%',
              50: '50%',
              75: '75%',
              100: '100%'
            }}
          />
        </div>
        <div className="order-entry-body-section flex-centered column">
          <div className="order-entry-body-section-info flex-centered column">
            <div className="order-entry-body-section-info-item flex align-center justify-between">
              <Paragraph type="danger">{dictionary.common.loanBalance}</Paragraph>
              <Paragraph type="danger">
                {currencyAbbrev(
                  orderSide === 'sell'
                    ? baseAssetPoolPosition?.loanBalance.tokens ?? 0
                    : quoteAssetPoolPosition?.loanBalance.tokens ?? 0,
                  false,
                  undefined,
                  ((orderSide === 'sell' ? currentMarket?.baseDecimals : currentMarket?.quoteDecimals) ?? 6) / 2
                )}
                {((orderSide === 'sell' && !orderAmount.isZero()) || !orderSize.isZero()) && (
                  <>
                    &nbsp;&#8594;&nbsp;
                    {currencyAbbrev(
                      orderSide === 'sell'
                        ? (baseAssetPoolPosition?.loanBalance.tokens ?? 0) + marginDifferenceShort.tokens
                        : (quoteAssetPoolPosition?.loanBalance.tokens ?? 0) + marginDifferenceLong.tokens,
                      false,
                      undefined,
                      ((orderSide === 'sell' ? currentMarket?.baseDecimals : currentMarket?.quoteDecimals) ?? 6) / 2
                    )}
                  </>
                )}
                {' ' + (orderSide === 'sell' ? currentMarket?.baseSymbol : currentMarket?.quoteSymbol)}
              </Paragraph>
            </div>
            <div className="order-entry-body-section-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.common.riskLevel}</Paragraph>
              <div className="flex-centered">
                <Paragraph type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
                {((orderSide === 'sell' && !orderAmount.isZero()) || !orderSize.isZero()) && (
                  <Paragraph type={orderSide === 'sell' ? projectedRiskStyleShort : projectedRiskStyleLong}>
                    &nbsp;&#8594;&nbsp;
                    {formatRiskIndicator(
                      orderSide === 'sell' ? projectedRiskIndicatorShort : projectedRiskIndicatorLong
                    )}
                  </Paragraph>
                )}
              </div>
            </div>
            <div className="order-entry-body-section-info-item flex align-center justify-between">
              <Paragraph type="secondary">{dictionary.tradeView.orderEntry.estimatedLiquidationPrice}</Paragraph>
              <Paragraph type="secondary">{`— ${currentMarket?.quoteSymbol ?? ''}`}</Paragraph>
            </div>
          </div>
          <Text italic type="danger" className={`order-error ${errorMessage.length ? '' : 'no-opacity'}`}>
            {errorMessage}
          </Text>
          <Paragraph
            italic
            className={`order-review ${
              !errorMessage && orderPrice > 0 && !orderAmount.isZero() && !orderSize.isZero() ? '' : 'no-opacity'
            }`}>
            {orderReviewMessage}
          </Paragraph>
        </div>
      </div>
      <div className="order-entry-footer view-element-item view-element-item-hidden flex-centered">
        <Button
          block
          loading={sendingOrder}
          disabled={
            disabled ||
            !orderPrice ||
            orderAmount.isZero() ||
            orderSize.isZero() ||
            orderAmount.gt(maxOrderAmount) ||
            orderSize.gt(maxOrderSize)
          }
          danger={orderSide === 'sell'}
          onClick={sumbitOrder}>
          {sendingOrder
            ? // @ts-ignore
              dictionary.tradeView.orderEntry[`${orderSide}ing`] + '...'
            : `${orderSide} ${currentMarket?.baseSymbol ?? '—'}`}
        </Button>
      </div>
    </div>
  );
}
