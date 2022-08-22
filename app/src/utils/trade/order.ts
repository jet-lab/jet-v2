import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentMarket } from '../../state/trade/market';
import { OrderSide, OrderType, OrderPrice, OrderAmount, OrderSize } from '../../state/trade/order';
import { CurrentAccount } from '../../state/user/accounts';

export function useOrderReviewMessage(): string {
  const dictionary = useRecoilValue(Dictionary);
  const currentMarket = useRecoilValue(CurrentMarket);
  const currentAccount = useRecoilValue(CurrentAccount);
  const baseAssetPoolPosition = currentMarket && currentAccount?.poolPositions[currentMarket.baseSymbol];
  const quoteAssetPoolPosition = currentMarket && currentAccount?.poolPositions[currentMarket.quoteSymbol];
  const orderSide = useRecoilValue(OrderSide);
  const orderType = useRecoilValue(OrderType);
  const orderPrice = useRecoilValue(OrderPrice);
  const orderAmount = useRecoilValue(OrderAmount);
  const orderSize = useRecoilValue(OrderSize);

  // Short selling (totally) review
  if (orderSide === 'sell' && baseAssetPoolPosition?.depositBalance?.isZero()) {
    return dictionary.tradeView.orderEntry.shortSellingTotalReview
      .replaceAll('{{AMOUNT}}', orderAmount.uiTokens)
      .replaceAll('{{BASE_CURRENCY}}', currentMarket?.baseSymbol ?? '')
      .replaceAll('{{SIZE}}', orderSize.uiTokens)
      .replaceAll('{{QUOTE_CURRENCY}}', currentMarket?.quoteSymbol ?? '');
    // Short selling (partially) review
  } else if (orderSide === 'sell' && orderAmount && baseAssetPoolPosition?.depositBalance?.lt(orderAmount)) {
    const shortAmount = orderAmount.sub(baseAssetPoolPosition?.depositBalance);
    return dictionary.tradeView.orderEntry.shortSellingPartialReview
      .replaceAll('{{COLLATERAL_AMOUNT}}', baseAssetPoolPosition.depositBalance.uiTokens)
      .replaceAll('{{SHORT_AMOUNT}}', shortAmount.uiTokens)
      .replaceAll('{{BASE_CURRENCY}}', currentMarket?.baseSymbol ?? '')
      .replaceAll('{{SIZE}}', orderSize.uiTokens)
      .replaceAll('{{QUOTE_CURRENCY}}', currentMarket?.quoteSymbol ?? '');
    // A normal order review
  } else if (orderSide === 'buy' && quoteAssetPoolPosition?.depositBalance?.isZero()) {
    return dictionary.tradeView.orderEntry.buyingOnMarginTotalReview
      .replaceAll('{{AMOUNT}}', orderAmount.uiTokens)
      .replaceAll('{{BASE_CURRENCY}}', currentMarket?.baseSymbol ?? '')
      .replaceAll('{{SIZE}}', orderSize.uiTokens)
      .replaceAll('{{QUOTE_CURRENCY}}', currentMarket?.quoteSymbol ?? '');
    // Short selling (partially) review
  } else if (orderSide === 'buy' && orderAmount && quoteAssetPoolPosition?.depositBalance?.lt(orderAmount)) {
    const marginAmount = orderAmount.sub(quoteAssetPoolPosition?.depositBalance);
    return dictionary.tradeView.orderEntry.buyingOnMarginPartialReview
      .replaceAll('{{COLLATERAL_AMOUNT}}', quoteAssetPoolPosition.depositBalance.uiTokens)
      .replaceAll('{{MARGIN_AMOUNT}}', marginAmount.uiTokens)
      .replaceAll('{{BASE_CURRENCY}}', currentMarket?.baseSymbol ?? '')
      .replaceAll('{{SIZE}}', orderSize.uiTokens)
      .replaceAll('{{QUOTE_CURRENCY}}', currentMarket?.quoteSymbol ?? '');
    // A normal order review
  } else {
    return dictionary.tradeView.orderEntry.orderReview
      .replaceAll('{{SIDE}}', orderSide.toUpperCase())
      .replaceAll('{{TYPE}}', orderType.toUpperCase())
      .replaceAll('{{BASE_CURRENCY}}', currentMarket?.baseSymbol ?? '')
      .replaceAll('{{QUOTE_CURRENCY}}', currentMarket?.quoteSymbol ?? '')
      .replaceAll('{{AMOUNT}}', orderAmount.uiTokens)
      .replaceAll('{{SIZE}}', orderSize.uiTokens)
      .replaceAll('{{PRICE}}', orderPrice.toString());
  }
}
