import { useEffect, useRef } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { TokenAmount } from '@jet-lab/margin';
import { CurrentMarket, MarketPrice } from '../../../state/trade/market';
import {
  OrderType,
  OrderPrice,
  OrderAmount,
  OrderSize,
  OrderPriceString,
  OrderAmountString,
  OrderSizeString
} from '../../../state/trade/order';
import { getTokenAmountFromNumber } from '../../../utils/currency';
import { Input, Typography } from 'antd';

type OrderInputType = 'price' | 'amount' | 'size';
export function OrderInput(props: {
  maxOrderAmount: TokenAmount;
  maxOrderSize: TokenAmount;
  inputType: OrderInputType;
  disabled: boolean;
  onPressEnter: () => void;
}): JSX.Element {
  const currentMarket = useRecoilValue(CurrentMarket);
  const baseDecimals = currentMarket?.baseDecimals || 6;
  const quoteDecimals = currentMarket?.quoteDecimals || 6;
  const marketPrice = useRecoilValue(MarketPrice);
  const orderType = useRecoilValue(OrderType);
  const [orderPrice, setOrderPrice] = useRecoilState(OrderPrice);
  const [orderPriceString, setOrderPriceString] = useRecoilState(OrderPriceString);
  const orderPriceRef = useRef(orderPriceString);
  const [orderAmount, setOrderAmount] = useRecoilState(OrderAmount);
  const [orderAmountString, setOrderAmountString] = useRecoilState(OrderAmountString);
  const orderAmountRef = useRef(orderAmountString);
  const [orderSize, setOrderSize] = useRecoilState(OrderSize);
  const [orderSizeString, setOrderSizeString] = useRecoilState(OrderSizeString);
  const orderSizeRef = useRef(orderSizeString);
  const maxInput =
    props.inputType === 'amount' ? props.maxOrderAmount : props.inputType === 'size' ? props.maxOrderSize : undefined;
  const { Paragraph } = Typography;

  // Set current input's string, update tokenAmount and adjust other order inputs
  function updateOrderAmounts(type: OrderInputType, stringValue: string) {
    const amount = parseFloat(stringValue);
    if (type === 'price') {
      setOrderPrice(amount);
      if (!orderAmount.isZero()) {
        const orderSize = orderAmount.tokens * amount;
        const orderSizeTokens = getTokenAmountFromNumber(orderSize, quoteDecimals);
        setOrderSize(orderSizeTokens);
        setOrderSizeString(orderSizeTokens.uiTokens);
        orderSizeRef.current = orderSizeTokens.uiTokens;
      } else if (!orderSize.isZero()) {
        const orderAmount = orderSize.tokens / amount;
        const orderAmountTokens = getTokenAmountFromNumber(orderAmount, baseDecimals);
        setOrderAmount(orderAmountTokens);
        setOrderAmountString(orderAmountTokens.uiTokens);
        orderAmountRef.current = orderAmountTokens.uiTokens;
      }
    } else if (type === 'amount') {
      const orderAmount = Math.min(amount, props.maxOrderAmount.tokens);
      const orderAmountTokens = getTokenAmountFromNumber(orderAmount, baseDecimals);
      setOrderAmount(orderAmountTokens);
      if (orderPrice > 0) {
        const orderSize = orderAmount * orderPrice;
        const orderSizeTokens = getTokenAmountFromNumber(orderSize, quoteDecimals);
        setOrderSize(orderSizeTokens);
        setOrderSizeString(orderSizeTokens.uiTokens);
        orderSizeRef.current = orderSizeTokens.uiTokens;
      }
    } else if (type === 'size') {
      const orderSize = Math.min(amount, props.maxOrderSize.tokens);
      const orderSizeTokens = getTokenAmountFromNumber(orderSize, quoteDecimals);
      setOrderSize(orderSizeTokens);
      if (orderPrice > 0) {
        const orderAmount = orderSize / orderPrice;
        const orderAmountTokens = getTokenAmountFromNumber(orderAmount, baseDecimals);
        setOrderAmount(orderAmountTokens);
        setOrderAmountString(orderAmountTokens.uiTokens);
        orderAmountRef.current = orderAmountTokens.uiTokens;
      }
    }
  }

  // Keep all amounts up to date if a string changes
  useEffect(() => {
    if (orderPriceString && orderPriceString !== orderPriceRef.current) {
      updateOrderAmounts('price', orderPriceString);
      orderPriceRef.current = orderPriceString;
    } else if (orderAmountString && orderAmountString !== orderAmountRef.current) {
      updateOrderAmounts('amount', orderAmountString);
      orderAmountRef.current = orderAmountString;
    } else if (orderSizeString && orderSizeString !== orderSizeRef.current) {
      updateOrderAmounts('size', orderSizeString);
      orderSizeRef.current = orderSizeString;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [orderPriceString, orderAmountString, orderSizeString]);

  // If a market order, lock the price input and update the others
  useEffect(() => {
    if (props.inputType === 'price' && orderType === 'ioc' && marketPrice) {
      setOrderPriceString(marketPrice.toString());
      if (orderAmount) {
        setOrderSizeString(orderAmount.muln(marketPrice).uiTokens);
      } else if (orderSize) {
        setOrderAmountString(orderSize.divn(marketPrice).uiTokens);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [marketPrice, orderType]);

  return (
    <div className="order-input">
      <Input
        className={`secondary-input ${
          (props.inputType === 'price' && orderPriceString) ||
          (props.inputType === 'amount' && orderAmountString) ||
          (props.inputType === 'size' && orderSizeString)
            ? 'has-value'
            : ''
        }`}
        type="string"
        max={maxInput?.tokens}
        disabled={props.disabled}
        value={
          props.inputType === 'price'
            ? orderPriceString
            : props.inputType === 'amount'
            ? orderAmountString
            : orderSizeString
        }
        onChange={e => {
          if (!currentMarket) {
            return;
          }

          // Check input and remove unnecessary 0's
          let inputString = e.target.value;
          while (!inputString.includes('.') && inputString[0] === '0') {
            inputString = inputString.substring(1);
          }
          if (isNaN(+inputString) || +inputString < 0 || +inputString > Number.MAX_SAFE_INTEGER) {
            inputString = '0';
          } else if (inputString.length > 1 && inputString[0] === '0' && inputString[1] !== '.') {
            inputString = inputString.substring(1);
          }

          // Update appropriate inputString
          if (props.inputType === 'price') {
            setOrderPriceString(inputString);
            if (!inputString) {
              setOrderPrice(0);
            }
          } else if (props.inputType === 'amount') {
            setOrderAmountString(inputString);
            if (!inputString) {
              setOrderAmount(TokenAmount.zero(currentMarket?.quoteDecimals ?? 6));
            }
          } else if (props.inputType === 'size') {
            setOrderSizeString(inputString);
            if (!inputString) {
              setOrderSize(TokenAmount.zero(currentMarket?.quoteDecimals ?? 6));
            }
          }
        }}
        onPressEnter={() => {
          if (props.disabled) {
            return;
          }

          let inputAmount = orderAmount;
          if (maxInput) {
            if (props.inputType === 'size') {
              inputAmount = orderSize;
            }
            const withinMaxRange = getTokenAmountFromNumber(
              Math.min(inputAmount.tokens, maxInput.tokens),
              maxInput.decimals
            );
            inputAmount = withinMaxRange;
            if (props.inputType === 'amount') {
              setOrderAmount(inputAmount);
              setOrderAmountString(inputAmount.uiTokens);
            } else if (props.inputType === 'size') {
              setOrderSize(inputAmount);
              setOrderSizeString(inputAmount.uiTokens);
            }
          }

          if (!inputAmount.isZero()) {
            props.onPressEnter();
          }
        }}
      />
      <Paragraph>
        {currentMarket ? (props.inputType === 'amount' ? currentMarket?.baseSymbol : currentMarket?.quoteSymbol) : '—'}
      </Paragraph>
    </div>
  );
}
