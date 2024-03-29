import { scaleLinear, scaleOrdinal } from '@visx/scale';
import { Bar, Line, LinePath } from '@visx/shape';
import { curveLinear } from '@visx/curve';
import { ParentSizeModern, ScaleSVG } from '@visx/responsive';
import { LegendOrdinal, LegendItem, LegendLabel } from '@visx/legend';
import { Tooltip, useTooltip, defaultStyles, TooltipWithBounds } from '@visx/tooltip';
import { AxisLeft, AxisBottom } from '@visx/axis';
import { createRef, RefObject, useCallback, useEffect, useMemo, useRef } from 'react';
import { Group } from '@visx/group';
import { localPoint } from '@visx/event';
import { pointAtCoordinateX } from './utils';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';
import { useSetRecoilState, useRecoilValue } from 'recoil';
import {
  SelectedFixedTermMarketAtom,
  CurrentOrderTab,
  CurrentOrderTabAtom,
  FixedTermMarketAtom
} from '@state/fixed-term/fixed-term-market-sync';
import { LoadingOutlined } from '@ant-design/icons';
import { useCurrencyFormatting } from '@utils/currency';
import { MarketAndConfig } from '@jet-lab/margin';

const tooltipStyles = {
  ...defaultStyles,
  background: '#3b6978',
  border: '1px solid white',
  color: 'white'
};

export interface ISeries {
  id: string;
  data: Array<{ x: number; y: number }>;
  type: string;
}

interface ILineChart {
  width: number;
  height: number;
  paddingTop: number;
  paddingLeft: number;
  paddingRight: number;
  paddingBottom: number;
  series: ISeries[];
  symbol: string;
}

interface IYValues {
  y: number;
  valueOfY: number;
  lineId: string;
}
interface ITooltipData {
  x: number;
  valueOfX: number;
  yValues: IYValues[];
}

export const LineChart = ({
  height,
  width,
  paddingTop,
  paddingLeft,
  paddingRight,
  paddingBottom,
  series,
  symbol
}: ILineChart) => {
  const setMarket = useSetRecoilState(SelectedFixedTermMarketAtom);
  const formatting = useCurrencyFormatting();

  // constraints
  const xMax = width - paddingLeft - paddingRight;
  const yMax = height - paddingTop - paddingBottom;

  const linesPathsRefs = useRef<RefObject<SVGPathElement>[]>([]);

  useEffect(() => {
    linesPathsRefs.current = series.map(() => createRef<SVGPathElement>());
  }, [series]);

  const { xScale, yScale, ordinalColorScale } = useMemo(() => {
    const maxValueOfX = series.reduce((max, series) => {
      const seriesMax = Math.max(...series.data.map(d => d.x));
      if (seriesMax > max) {
        max = seriesMax;
      }
      return max;
    }, 0);
    const maxValueOfY = series.reduce((max, series) => {
      const seriesMax = Math.max(...series.data.map(d => d.y));
      if (seriesMax > max) {
        max = seriesMax;
      }
      return max * 1.05;
    }, 0);
    const xScale = scaleLinear<number>({
      domain: [0, maxValueOfX],
      clamp: true
    });
    const yScale = scaleLinear<number>({
      domain: [maxValueOfY, 0],
      clamp: true
    });
    xScale.range([0, xMax]);
    yScale.range([0, yMax]);

    const ordinalColorScale = scaleOrdinal({
      domain: series.map(s => s.id),
      range: ['#66d981', '#71f5ef', '#4899f1', '#7d81f6']
    });
    return { xScale, yScale, ordinalColorScale, maxValueOfY };
  }, [width, height, series]);

  const { hideTooltip, showTooltip, tooltipData, tooltipLeft, tooltipTop } = useTooltip<ITooltipData>();

  const handleTooltip = useCallback(
    (event: React.TouchEvent<SVGGElement> | React.MouseEvent<SVGGElement>) => {
      const { x } = localPoint(event) || { x: 0 };
      const paths = linesPathsRefs.current;
      const valueOfX = xScale.invert(x - paddingLeft);
      const yValues: IYValues[] = [];
      paths.map(ref => {
        const path = ref.current;
        if (path) {
          const y = pointAtCoordinateX(path, x - paddingLeft, 5);
          if (y) {
            yValues.push({
              y,
              valueOfY: yScale.invert(y) * 100,
              lineId: path.getAttribute('id') || ''
            });
          }
        }
      });

      showTooltip({
        tooltipData: {
          x,
          valueOfX,
          yValues
        },
        tooltipLeft: x,
        tooltipTop: yValues.reduce((all, val) => all + val.y, 0) / yValues.length
      });
    },
    [showTooltip, height, width, series]
  );

  return (
    <>
      <LegendOrdinal scale={ordinalColorScale} labelFormat={label => label}>
        {labels => {
          return (
            <div className="chart-legend">
              {labels.map((label, i) => {
                const split = label.text.split('_');
                const marketName = friendlyMarketName(split[0], parseInt(split[1]));
                return (
                  <LegendItem
                    key={`legend-quantile-${i}`}
                    className="chart-legend-item"
                    onClick={() => {
                      setMarket(i);
                    }}>
                    <svg width={12} height={12}>
                      <circle cx="50%" cy="50%" fill={label.value} r={6} />
                    </svg>
                    <LegendLabel align="left">{marketName}</LegendLabel>
                  </LegendItem>
                );
              })}
            </div>
          );
        }}
      </LegendOrdinal>
      <ScaleSVG width={width} height={height}>
        <Group style={{ cursor: 'crosshair' }} top={paddingTop} left={paddingLeft}>
          {/* This bar is used to target the tooltip across the whole chart */}
          {tooltipData && tooltipLeft && (
            <Line
              from={{ x: tooltipLeft - paddingLeft, y: 0 }}
              to={{ x: tooltipLeft - paddingLeft, y: yMax }}
              stroke="#75daad"
              strokeWidth={2}
              pointerEvents="none"
              strokeDasharray="5,2"
            />
          )}
          {tooltipLeft &&
            tooltipData?.yValues.map(line => (
              <g key={`${line.lineId}-marker`}>
                <circle
                  fill={ordinalColorScale(line.lineId)}
                  r={4}
                  stroke="#fff"
                  strokeWidth={1}
                  cx={tooltipData.x - paddingLeft}
                  cy={line.y}
                />
                <Line
                  from={{ x: 0, y: line.y }}
                  to={{ x: tooltipLeft - paddingLeft, y: line.y }}
                  stroke={ordinalColorScale(line.lineId)}
                  strokeWidth={1}
                  opacity={0.4}
                  strokeDasharray="5,2"
                />
              </g>
            ))}
          {series.map((s, index) => (
            <LinePath
              id={s.id}
              innerRef={linesPathsRefs.current[index]}
              key={s.id}
              curve={curveLinear}
              data={s.data}
              x={d => xScale(d.x) || 0}
              y={d => yScale(d.y) || 0}
              stroke={ordinalColorScale(s.id)}
              strokeWidth={2}
              strokeOpacity={1}
            />
          ))}
          <AxisLeft
            tickStroke="rgba(255,255,255,0.6)"
            hideAxisLine={true}
            tickFormat={val => `${(val.valueOf() * 100).toFixed(2)}%`}
            scale={yScale}
            tickLabelProps={() => ({
              fontSize: 10,
              fill: '#fff',
              opacity: 0.6,
              textAnchor: 'end',
              dy: 4,
              dx: -8
            })}
            numTicks={8}
          />
          <AxisBottom
            hideAxisLine={true}
            top={yMax}
            tickFormat={val =>
              formatting.currencyAbbrev(val.valueOf(), 0, true, undefined, undefined, undefined, 'thousands')
            }
            tickStroke="rgba(255,255,255,0.6)"
            scale={xScale}
            tickLabelProps={() => ({
              fontSize: 10,
              fill: '#fff',
              opacity: 0.6,
              textAnchor: 'middle',
              dy: 8
            })}
            numTicks={8}
          />
          {height > 0 && width > 0 && (
            <Bar
              width={xMax}
              height={yMax}
              fill="transparent"
              rx={8}
              onTouchStart={handleTooltip}
              onTouchMove={handleTooltip}
              onMouseMove={handleTooltip}
              onMouseLeave={() => hideTooltip()}
            />
          )}
        </Group>
      </ScaleSVG>
      {tooltipData && tooltipData.yValues.length > 0 && (
        <TooltipWithBounds top={tooltipTop} left={tooltipLeft} style={tooltipStyles}>
          {tooltipData.yValues.map(s => {
            const split = s.lineId.split('_');
            return (
              <div key={`${s.lineId}-tooltip-value`}>
                {friendlyMarketName(split[0], parseInt(split[1]))}: {s.valueOfY.toFixed(2)}%
              </div>
            );
          })}
        </TooltipWithBounds>
      )}
      {tooltipData && (
        <Tooltip
          top={yMax + paddingTop}
          left={tooltipLeft}
          style={{
            ...defaultStyles,
            minWidth: 72,
            textAlign: 'center',
            transform: 'translateX(calc(-50% - 8px))'
          }}>
          {symbol}{' '}
          {formatting.currencyAbbrev(tooltipData.valueOfX, 0, false, undefined, undefined, undefined, 'thousands')}
        </Tooltip>
      )}
    </>
  );
};

interface ResponsiveLineChartProps {
  series: ISeries[];
  symbol: string;
}
export const ResponsiveLineChart = ({ series, symbol }: ResponsiveLineChartProps) => {
  const market = useRecoilValue(FixedTermMarketAtom);
  const currentTab = useRecoilValue(CurrentOrderTabAtom);
  return (
    <div className="responsive-line-chart-container">
      <div className="responsive-line-chart-y-label">Annualised interest rate</div>
      <ParentSizeModern>
        {parent =>
          series.length > 0 ? (
            <LineChart
              height={parent.height}
              width={parent.width}
              paddingTop={64}
              paddingBottom={80}
              paddingLeft={96}
              paddingRight={24}
              series={series.filter(s => s.data.length > 0)}
              symbol={symbol}
            />
          ) : (
            <LoadingOutlined />
          )
        }
      </ParentSizeModern>
      <div className="responsive-line-chart-x-label">{getChartXAxisLabel(currentTab, market)}</div>
    </div>
  );
};

const getChartXAxisLabel = (currentTab: CurrentOrderTab, market: MarketAndConfig | null) => {
  if (!market?.config?.symbol) return '';
  switch (currentTab) {
    case 'borrow-now':
      return `Cumulative ${market.config.symbol} offers`;
    case 'lend-now':
      return `Cumulative ${market.config.symbol} requests`;
    case 'offer-loan':
      return `Cumulative ${market.config.symbol} offers`;
    case 'request-loan':
      return `Cumulative ${market.config.symbol} requests`;
    default:
      return ``;
  }
};
