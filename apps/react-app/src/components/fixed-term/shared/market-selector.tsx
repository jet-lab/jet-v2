import { useRecoilState, useRecoilValue } from 'recoil';
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { MarketSelectorButtons } from './market-selector-buttons';

const { Option } = Select;

interface FixedTermMarketSelectorProps {
  type: 'asks' | 'bids';
}

export const FixedTermMarketSelector = ({ type }: FixedTermMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedTermMarketAtom);

  return (
    <div className="fixed-term-selector-view view-element">
      <div className="fixed-term-selector-view-container">
        <Select
          value={selectedMarket + 1}
          showSearch={true}
          suffixIcon={<AngleDown className="jet-icon" />}
          filterOption={(val, opt) => {
            return opt?.name && opt.name.indexOf(val) !== -1;
          }}
          onChange={value => setSelectedMarket(value - 1)}>
          {markets.map((market, index) => (
            <Option key={market.name} name={marketToString(market.config)} value={index + 1}>
              {marketToString(market.config)}
            </Option>
          ))}
        </Select>
        <MarketSelectorButtons
          marginAccount={marginAccount}
          markets={markets}
          selectedMarket={markets[selectedMarket]}
        />
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
