import Title from "antd/lib/typography/Title"
import ApexCharts from "apexcharts"
import { Suspense, useEffect, useRef, useState } from "react"
import { useRecoilState, useRecoilValue } from "recoil"
import { ReorderArrows } from "../misc/ReorderArrows"
import { Dictionary } from "../../state/settings/localization/localization"
import { FixedBorrowRowOrder, FixedLendRowOrder } from "../../state/views/fixed-term"
import { AllFixedMarketsOrderBooksAtom, FixedMarketOrderBookAtom } from "../../state/fixed/fixed-term-market-sync"
import { MockBook, MockOrder } from "../../state/fixed/mocks"
import { useCurrencyFormatting } from "../../utils/currency"

interface Formatter {
  currencyFormatter: (
    value: number,
    fiatValues?: boolean | undefined,
    decimals?: number | undefined,
    ciel?: boolean | undefined
  ) => string
  currencyAbbrev: (
    total: number,
    fiatValues?: boolean | undefined,
    price?: number | undefined,
    decimals?: number | undefined
  ) => string
}

interface FixedChart {
  type: "bids" | "asks"
  decimals?: number
}

interface DataPoint {
  x: number
  y: number
}

// Setup data for the chart
const getChartData = (orders: MockOrder[]): DataPoint[] =>
  orders.reduce((all, order) => {
    const previousPoint = all.length > 0 ? all[all.length - 1] : { x: 0, y: 0 }
    const point = {
      x: previousPoint.x + Number(order.quote_size),
      y: previousPoint.y + Number(order.base_size)
    }
    if (typeof point.x === "number" && typeof point.y === "number") all.push(point)
    return all
  }, [] as Array<{ x: number; y: number }>)

const getOptions = (books: MockBook[], decimals: number, type: string, formatting: Formatter) => {
  const series = books.map(book => ({
    name: book.market,
    data: getChartData(type === "asks" ? book.asks : book.bids)
  }))
  return {
    chart: {
      type: "line",
      width: "100%",
      height: "515px"
    },
    xaxis: {
      type: "numeric",
      tickAmount: 10,
      labels: {
        formatter: (val: string) => formatting.currencyFormatter(parseInt(val), false, decimals)
      }
    },
    yaxis: {
      type: "numeric",
      tickAmount: 10,
      labels: {
        formatter: (val: string) => formatting.currencyFormatter(parseInt(val), false, decimals)
      }
    },
    series,
    theme: {
      mode: "dark"
    },
    tooltip: {
      enabled: true,
      custom: ({ dataPointIndex, seriesIndex }: any) => {
        const item = series[seriesIndex].data[dataPointIndex]
        if (!item) return
        const rate = 1 / (item.x / item.y) - 1 // TODO FIXME, scale rate by tenor length
        const amount = item.x
        const repay = item.y
        return `<div className="flex-centered column">
            <div>Amount ${formatting.currencyFormatter(amount, false, 2)}</div>
            <div>Repay ${formatting.currencyFormatter(repay, false, 2)}</div>
            <div>Rate ${(rate * 100).toFixed(2)}%</div>
          </div>`
      }
    }
  }
}

const FixedPriceChart = ({ type, decimals = 6 }: FixedChart) => {
  const orderBook = useRecoilValue(FixedMarketOrderBookAtom)
  const [currentChart, setCurrentChart] = useState<ApexCharts | undefined>(undefined)
  const formatting = useCurrencyFormatting()

  const books = useRecoilValue(AllFixedMarketsOrderBooksAtom)
  const ref = useRef<HTMLDivElement>(null)

  const orders = orderBook[type]

  // Initialize chart
  useEffect(() => {
    if (ref.current && !currentChart) {
      const opts = getOptions(books, decimals, type, formatting)
      const fixedPriceChart = new ApexCharts(document.querySelector(`.fixed-term-graph-container`), opts)
      fixedPriceChart.render()
      setCurrentChart(fixedPriceChart)
    }
  }, [ref.current])

  // Update chart
  useEffect(() => {
    if (ref.current && orders.length > 0 && currentChart) {
      const opts = getOptions(books, decimals, type, formatting)
      currentChart.updateOptions(opts)
    }
  }, [orders, currentChart, ref.current])

  // Clean on dismount
  useEffect(() => currentChart?.destroy(), [])
  return (
    <div
      ref={ref}
      className="fixed-term-graph-container view-element-item view-element-item-hidden flex-centered"></div>
  )
}

export const FixedPriceChartContainer = ({ type }: FixedChart) => {
  const dictionary = useRecoilValue(Dictionary)
  const [rowOrder, setRowOrder] = useRecoilState(type === "asks" ? FixedLendRowOrder : FixedBorrowRowOrder)

  return (
    <div className="fixed-term-graph view-element view-element-hidden flex align-center justify-end column">
      <div className="fixed-term-graph-head view-element-item view-element-item-hidden flex justify-center column">
        <div className="fixed-term-graph-head-info flex align-end">
          <div className="flex-centered">
            <Title level={2}>{dictionary.fixedView[type === "asks" ? "lend" : "borrow"].chartTitle}</Title>
          </div>
        </div>
      </div>
      <Suspense fallback={<div>Loading</div>}>
        <FixedPriceChart type={type} decimals={6} />
      </Suspense>
      <ReorderArrows component="fixedChart" order={rowOrder} setOrder={setRowOrder} />
    </div>
  )
}
