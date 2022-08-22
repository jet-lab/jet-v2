import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { PairSearchModal } from '../../../state/modals/modals';
import { TradeViewOrder } from '../../../state/views/views';
import { CurrentMarketPair, MarketsInit, MarketPairs, MarketPrice } from '../../../state/trade/market';
import { CurrentPoolSymbol } from '../../../state/borrow/pools';
import { OneDayVolume } from '../../../state/trade/recentTrades';
import { PriceHistoryLoading, PriceHistory } from '../../../state/trade/priceHistory';
import { formatMarketPair, formatRate } from '../../../utils/format';
import { useCurrencyFormatting } from '../../../utils/currency';
import { Select, Skeleton, Typography } from 'antd';
import { ReorderArrows } from '../../misc/ReorderArrows';
import { TokenLogo } from '../../misc/TokenLogo';
import { PriceHistorySparkline } from '../../misc/PriceHistorySparkline';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';

export function PairSelector(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const setPairSearchModalOpen = useSetRecoilState(PairSearchModal);
  const [tradeViewOrder, setTradeViewOrder] = useRecoilState(TradeViewOrder);
  const setCurrentPoolSymbol = useSetRecoilState(CurrentPoolSymbol);
  const marketsInit = useRecoilValue(MarketsInit);
  const marketPairs = useRecoilValue(MarketPairs);
  const marketPrice = useRecoilValue(MarketPrice);
  const [currentMarketPair, setCurrentMarketPair] = useRecoilState(CurrentMarketPair);
  const [pairSelectOptions, setPairSelectOptions] = useState<{ label: JSX.Element; value: string }[]>([]);
  const priceHistory = useRecoilValue(PriceHistory);
  const oneDayVolume = useRecoilValue(OneDayVolume);
  const sixHourPriceHistory = priceHistory['3'];
  const oneDayPriceHistory = priceHistory['15'];
  const priceHistoryLoading = useRecoilValue(PriceHistoryLoading);
  const { Option } = Select;
  const { Paragraph, Text } = Typography;

  // Setup keypress listener for quick pair search
  window.addEventListener('keypress', e => {
    if (e.ctrlKey && e.key === '/') {
      setPairSearchModalOpen(true);
    }
  });

  useEffect(() => {
    const selectOptions = [];
    for (const pair of marketPairs) {
      selectOptions.push({
        label: (
          <>
            <TokenLogo height={20} symbol={pair.split('/')[0]} />
            {formatMarketPair(pair) ?? '—'}
          </>
        ),
        value: pair
      });
    }
    setPairSelectOptions(selectOptions);
  }, [marketPairs]);

  return (
    <div className="pair-selector view-element view-element-hidden flex align-center justify-start">
      <div className="pair-selector-select view-element-item view-element-item-hidden flex align-center justify-start">
        <div className="flex-centered column">
          <Select
            value={currentMarketPair}
            dropdownClassName="lg-dropdown rounded-dropdown"
            suffixIcon={<AngleDown className="jet-icon" />}
            onChange={pair => {
              setCurrentMarketPair(pair.replaceAll(' ', ''));
              setCurrentPoolSymbol(pair.split('/')[0]);
            }}>
            {pairSelectOptions.map(pair => (
              <Option key={pair.value} value={pair.value}>
                {pair.label}
              </Option>
            ))}
          </Select>
          <Text italic>{dictionary.tradeView.pairSelector.typeToEnterPair}</Text>
        </div>
      </div>
      <div className="pair-selector-data view-element-item view-element-item-hidden flex align-start justify-center">
        <div className="pair-selector-data-half flex">
          <div className="pair-selector-data-item flex column">
            <Text className="pair-selector-data-item-title small-accent-text">{dictionary.common.marketPrice}</Text>
            {marketPrice ? (
              <Paragraph>{currencyFormatter(marketPrice, false, 3)}</Paragraph>
            ) : marketsInit && !marketPrice ? (
              <Paragraph className="not-available-text" italic>
                {dictionary.common.notAvailable}
              </Paragraph>
            ) : (
              <Skeleton paragraph={false} active />
            )}
          </div>
          <div className="pair-selector-data-item flex column">
            <Text className="pair-selector-data-item-title small-accent-text">
              {dictionary.tradeView.pairSelector.sixHourChange}
            </Text>
            {!marketsInit || (!sixHourPriceHistory && priceHistoryLoading) ? (
              <Skeleton paragraph={false} active />
            ) : sixHourPriceHistory && sixHourPriceHistory.percentageChange ? (
              <Paragraph>{formatRate(sixHourPriceHistory.percentageChange)}</Paragraph>
            ) : (
              <Paragraph className="not-available-text" italic>
                {dictionary.common.notAvailable}
              </Paragraph>
            )}
          </div>
        </div>
        <div className="pair-selector-data-half flex">
          <div className="pair-selector-data-item flex column">
            <Text className="pair-selector-data-item-title small-accent-text">
              {dictionary.tradeView.pairSelector.dayChange}
            </Text>
            {!marketsInit || (!oneDayPriceHistory && priceHistoryLoading) ? (
              <Skeleton paragraph={false} active />
            ) : oneDayPriceHistory && oneDayPriceHistory.percentageChange ? (
              <Paragraph>{formatRate(oneDayPriceHistory.percentageChange)}</Paragraph>
            ) : (
              <Paragraph className="not-available-text" italic>
                {dictionary.common.notAvailable}
              </Paragraph>
            )}
          </div>
          <div className="pair-selector-data-item flex column">
            <Text className="pair-selector-data-item-title small-accent-text">
              {dictionary.tradeView.pairSelector.dayVolume}
            </Text>
            {marketsInit ? (
              oneDayVolume ? (
                <Paragraph>{currencyAbbrev(oneDayVolume, true)}</Paragraph>
              ) : (
                <Paragraph className="not-available-text" italic>
                  {dictionary.common.notAvailable}
                </Paragraph>
              )
            ) : (
              <Skeleton paragraph={false} active />
            )}
          </div>
        </div>
      </div>
      <PriceHistorySparkline />
      <div className="view-element-item view-element-item-hidden">
        <ReorderArrows component="pairSelector" order={tradeViewOrder} setOrder={setTradeViewOrder} vertical />
      </div>
    </div>
  );
}
