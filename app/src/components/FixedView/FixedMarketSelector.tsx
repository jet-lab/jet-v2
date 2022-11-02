import { useRecoilState, useRecoilValue } from 'recoil'; '../misc/ReorderArrows';
import { AllFixedMarketsAtom, SelectedFixedMarketAtom } from '@state/fixed/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { useCurrencyFormatting } from '@utils/currency';

const { Option } = Select;

interface FixedMarketSelectorProps {
  type: 'asks' | 'bids';
}
export const FixedMarketSelector = ({ type }: FixedMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedMarketsAtom);
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedMarketAtom);
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
              {market.name}
            </Option>
          ))}
        </Select>
        <div className="stats">
          <div className="single-stat">
            <div className="header">Total Lent</div>
            <div>{formatting.currencyAbbrev(0)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Total Borrow</div>
            <div>{formatting.currencyAbbrev(0)}</div>
          </div>
          <div className="single-stat">
            <div className="header">12 hrs change</div>
            <div>{formatting.currencyAbbrev(0)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">24 hrs change</div>
            <div>{formatting.currencyAbbrev(0)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">Volume</div>
            <div>{formatting.currencyAbbrev(0)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Daily Range</div>
            <div>
              {formatting.currencyAbbrev(0)}-{formatting.currencyAbbrev(0)}
            </div>
          </div>
        </div>
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
