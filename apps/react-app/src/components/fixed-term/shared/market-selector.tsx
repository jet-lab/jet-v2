import { useRecoilState, useRecoilValue } from 'recoil';
('../misc/ReorderArrows');
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { useCurrencyFormatting } from '@utils/currency';
import { marketToString } from '@utils/jet/fixed-term-utils';

const { Option } = Select;

interface FixedTermMarketSelectorProps {
  type: 'asks' | 'bids';
}
export const FixedTermMarketSelector = ({ type }: FixedTermMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedTermMarketAtom);
  const formatting = useCurrencyFormatting();
  return (
    <div className="fixed-term-selector-view view-element">
      <div className="fixed-term-selector-view-container">
        <Select
          value={selectedMarket + 1}
          showSearch={true}
          suffixIcon={<AngleDown className="jet-icon" />}
          onChange={value => setSelectedMarket(value - 1)}>
          {markets.map((market, index) => (
            <Option key={market.name} value={index + 1}>
              {marketToString(market.config)}
            </Option>
          ))}
        </Select>
        <div className="stats">
          <div className="single-stat">
            <div className="header">Total Lent</div>
            <div>{formatting.currencyAbbrev(0, 2)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Total Borrow</div>
            <div>{formatting.currencyAbbrev(0, 2)}</div>
          </div>
          <div className="single-stat">
            <div className="header">12 hrs change</div>
            <div>{formatting.currencyAbbrev(0, 2)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">24 hrs change</div>
            <div>{formatting.currencyAbbrev(0, 2)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">Volume</div>
            <div>{formatting.currencyAbbrev(0, 2)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Daily Range</div>
            <div>
              {formatting.currencyAbbrev(0, 2)}-{formatting.currencyAbbrev(0, 2)}
            </div>
          </div>
        </div>
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
