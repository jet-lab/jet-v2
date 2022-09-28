import { useRecoilState, useRecoilValue } from "recoil"
import { AllFixedMarketsAtom, FixedMarketAtom, SelectedFixedMarketAtom } from "../../state/fixed/fixed-term-market-sync"
import { FixedBorrowViewOrder, FixedLendViewOrder } from "../../state/views/fixed-term"
import { ReorderArrows } from "../misc/ReorderArrows"
import { Select } from "antd"
import AngleDown from "../../styles/icons/arrow-angle-down.svg"
import { useCurrencyFormatting } from "../../utils/currency"
import { generateMarkets } from "../../state/fixed/mocks"

const { Option } = Select

interface FixedMarketSelectorProps {
  type: "asks" | "bids"
}
const mockMarkets = generateMarkets()

export const FixedMarketSelector = ({ type }: FixedMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === "asks" ? FixedLendViewOrder : FixedBorrowViewOrder)
  const markets = useRecoilValue(AllFixedMarketsAtom)
  const mockMarket = mockMarkets[0]
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedMarketAtom)
  const formatting = useCurrencyFormatting()
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
            <div>{formatting.currencyAbbrev(mockMarket.totalLent)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Total Borrow</div>
            <div>{formatting.currencyAbbrev(mockMarket.totalBorrowed)}</div>
          </div>
          <div className="single-stat">
            <div className="header">12 hrs change</div>
            <div>{formatting.currencyAbbrev(mockMarket.change12hrs)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">24 hrs change</div>
            <div>{formatting.currencyAbbrev(mockMarket.change24hrs)}%</div>
          </div>
          <div className="single-stat">
            <div className="header">Volume</div>
            <div>{formatting.currencyAbbrev(mockMarket.volume)}</div>
          </div>
          <div className="single-stat">
            <div className="header">Daily Range</div>
            <div>
              {formatting.currencyAbbrev(mockMarket.dailyRange[0])}-
              {formatting.currencyAbbrev(mockMarket.dailyRange[1])}
            </div>
          </div>
        </div>
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  )
}
