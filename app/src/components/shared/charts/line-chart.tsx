import { scaleLinear, scaleOrdinal } from '@visx/scale';
import { Bar, Line, LinePath } from '@visx/shape';
import { curveLinear } from '@visx/curve';
import { ParentSizeModern, ScaleSVG } from '@visx/responsive';
import { LegendOrdinal, LegendItem, LegendLabel } from '@visx/legend';
import { Tooltip, useTooltip, defaultStyles, TooltipWithBounds } from '@visx/tooltip';
import { AxisLeft, AxisBottom } from '@visx/axis';
import { createRef, useCallback, useMemo, useRef } from 'react';
import { Group } from '@visx/group';
import { localPoint } from '@visx/event';
import { pointAtCoordinateX } from './utils';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';

const tooltipStyles = {
  ...defaultStyles,
  background: '#3b6978',
  border: '1px solid white',
  color: 'white'
};

interface ISeries {
  id: string;
  data: Array<{ x: number; y: number }>;
}

const sampleData: ISeries = {
  id: 'USDC_86400',
  data: [
    {
      x: 0,
      y: 0
    },
    {
      x: 1,
      y: 1
    },
    {
      x: 2,
      y: 10
    }
  ]
};

const sampleData2: ISeries = {
  id: 'USDC_604800',
  data: [
    {
      x: 0,
      y: 0.2
    },
    {
      x: 1,
      y: 3
    },
    {
      x: 2,
      y: 2
    }
  ]
};

interface ILineChart {
  width: number;
  height: number;
  paddingTop: number;
  paddingLeft: number;
  paddingRight: number;
  paddingBottom: number;
  series: ISeries[];
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
  series
}: ILineChart) => {
  // constraints
  const xMax = width - paddingLeft - paddingRight;
  const yMax = height - paddingTop - paddingBottom;

  const linesPathsRefs = useRef(series.map(() => createRef<SVGPathElement>()));

  const { xScale, yScale, ordinalColorScale } = useMemo(() => {
    const xScale = scaleLinear<number>({
      domain: [0, 2],
      clamp: true
    });
    const yScale = scaleLinear<number>({
      domain: [10, 0],
      clamp: true
    });
    xScale.range([0, xMax]);
    yScale.range([0, yMax]);

    const ordinalColorScale = scaleOrdinal({
      domain: series.map(s => s.id),
      range: ['#66d981', '#71f5ef', '#4899f1', '#7d81f6']
    });
    return { xScale, yScale, ordinalColorScale };
  }, [width, height]);

  const { hideTooltip, showTooltip, tooltipData, tooltipLeft, tooltipTop } = useTooltip<ITooltipData>();

  const handleTooltip = useCallback(
    (event: React.TouchEvent<SVGGElement> | React.MouseEvent<SVGGElement>) => {
      const { x } = localPoint(event) || { x: 0 };
      const paths = linesPathsRefs.current;
      const valueOfX = xScale.invert(x - paddingLeft);
      const yValues: IYValues[] = [];
      paths.map(ref => {
        const path = ref.current;
        const y = pointAtCoordinateX(path, x - paddingLeft);
        yValues.push({
          y,
          valueOfY: yScale.invert(y),
          lineId: path.getAttribute('id')
        });
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
    [showTooltip, height, width]
  );

  return (
    <>
      <ScaleSVG width={width} height={height}>
        <Group top={paddingTop} left={paddingLeft}>
          {/* This bar is used to target the tooltip across the whole chart */}
          {tooltipData && (
            <Line
              from={{ x: tooltipLeft - paddingLeft, y: 0 }}
              to={{ x: tooltipLeft - paddingLeft, y: yMax }}
              stroke="#75daad"
              strokeWidth={2}
              pointerEvents="none"
              strokeDasharray="5,2"
            />
          )}
          {tooltipData?.yValues.map(line => (
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
            scale={yScale}
            tickLabelProps={() => ({
              fill: '#fff',
              opacity: 0.6,
              textAnchor: 'end',
              dy: 4,
              dx: -8
            })}
          />
          <AxisBottom
            hideAxisLine={true}
            top={yMax}
            tickStroke="rgba(255,255,255,0.6)"
            scale={xScale}
            tickLabelProps={() => ({
              fill: '#fff',
              opacity: 0.6,
              textAnchor: 'middle',
              dy: 8
            })}
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
      {tooltipData && (
        <TooltipWithBounds top={tooltipTop - 12} left={tooltipLeft + 12} style={tooltipStyles}>
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
          {tooltipData.valueOfX.toFixed(2)}
        </Tooltip>
      )}
      <LegendOrdinal scale={ordinalColorScale} labelFormat={label => label}>
        {labels => {
          return (
            <div className="chart-legend">
              {labels.map((label, i) => (
                <LegendItem
                  key={`legend-quantile-${i}`}
                  margin="0 5px"
                  // onClick={() => {
                  //     if (events) alert(`clicked: ${JSON.stringify(label)}`);
                  // }}
                >
                  <svg width={12} height={12}>
                    <circle cx="50%" cy="50%" fill={label.value} r={6} />
                  </svg>
                  <LegendLabel align="left" margin="0 0 0 4px">
                    {label.text}
                  </LegendLabel>
                </LegendItem>
              ))}
            </div>
          );
        }}
      </LegendOrdinal>
    </>
  );
};

export const ResponsiveLineChart = ({}) => {
  return (
    <ParentSizeModern>
      {parent => (
        <LineChart
          height={parent.height}
          width={parent.width}
          paddingTop={60}
          paddingBottom={40}
          paddingLeft={40}
          paddingRight={16}
          series={[sampleData, sampleData2]}
        />
      )}
    </ParentSizeModern>
  );
};
