import { useEffect, useRef, useState } from 'react';
import { useSetRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { OrderType, OrderPriceString } from '../../../state/trade/order';
import { MarketsInit, CurrentMarket, MarketPrice as MarketPriceState } from '../../../state/trade/market';
import { useCurrencyFormatting } from '../../../utils/currency';
import { animateDataUpdate } from '../../../utils/ui';
import { Skeleton, Typography } from 'antd';
import { ReactComponent as ArrowUp } from '../../../styles/icons/arrow-up.svg';
import { ReactComponent as ArrowDown } from '../../../styles/icons/arrow-down.svg';

export function MarketPrice(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter } = useCurrencyFormatting();
  const setOrderType = useSetRecoilState(OrderType);
  const setOrderPriceString = useSetRecoilState(OrderPriceString);
  const marketsInit = useRecoilValue(MarketsInit);
  const currentMarket = useRecoilValue(CurrentMarket);
  const marketPrice = useRecoilValue(MarketPriceState);
  const [prevMarketPrice, setPrevMarketPrice] = useState(marketPrice);
  const { Paragraph } = Typography;

  // Prefill order price to market price and change to market order
  function prefillMarketOrder() {
    if (!marketPrice) {
      return;
    }

    setOrderType('ioc');
    setOrderPriceString(marketPrice.toString());
  }

  // Track previous market price, update and animate UI on changes
  const prevPriceRef = useRef(marketPrice);
  useEffect(() => {
    setPrevMarketPrice(prevPriceRef.current);
    prevPriceRef.current = marketPrice;
    animateDataUpdate('flash-opacity', '.market-price');
  }, [marketPrice]);

  // Reset prev price when we change pairs
  useEffect(() => {
    prevPriceRef.current = undefined;
  }, [currentMarket]);

  return (
    <div className="market-price flex-centered">
      {marketPrice ? (
        <Paragraph
          strong
          type={prevMarketPrice ? (marketPrice > prevMarketPrice ? 'success' : 'danger') : undefined}
          onClick={() => prefillMarketOrder()}>
          {prevMarketPrice ? (
            marketPrice > prevMarketPrice ? (
              <ArrowUp className="jet-icon" />
            ) : (
              <ArrowDown className="jet-icon" />
            )
          ) : (
            ''
          )}
          {currencyFormatter(marketPrice, false, 3)}
        </Paragraph>
      ) : marketsInit && !marketPrice ? (
        <Paragraph className="not-available-text" italic>
          {dictionary.common.notAvailable}
        </Paragraph>
      ) : (
        <Skeleton className="align-center" paragraph={false} active />
      )}
    </div>
  );
}
