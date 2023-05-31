import { AxisBottom, AxisLeft } from '@visx/axis';
import { scaleLinear } from '@visx/scale';
import { useCallback, useMemo, useRef } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ScaleSVG } from '@visx/responsive';
import { Line, Bar, LinePath } from '@visx/shape';
import { TooltipWithBounds, defaultStyles, useTooltip } from '@visx/tooltip';
import { Threshold } from '@visx/threshold';
import { localPoint } from '@visx/event';
import { pointAtCoordinateX } from '@components/fixed-term/shared/charts/utils';

const tooltipStyles = {
  ...defaultStyles,
  display: 'flex',
  flexDirection: 'column' as 'column',
  color: '#444',
  fontSize: 16
};
interface ITooltipData {
  qty: number;
  price: number;
  type: 'ask' | 'bid' | 'hidden';
  x: number;
  y: number;
}
interface DepthChartProps {
  height: number;
  width: number;
  padding?: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  bidsDescending: [price: number, amt: number][];
  asksAscending: [price: number, amt: number][];
  midPoint?: number;
  xRange: [min: number, max: number];
  yRange: [min: number, max: number];
  base: {
    symbol: string;
    expo: number;
  };
  quote: {
    symbol: string;
    expo: number;
  };
  isPct?: boolean;
}

export const DepthChart = ({
  height,
  width,
  padding = { top: 20, left: 80, right: 32, bottom: 60 },
  bidsDescending = [],
  asksAscending = [],
  midPoint,
  xRange = [0, 0],
  yRange = [0, 0],
  base,
  quote,
  isPct
}: DepthChartProps) => {
  const dictionary = useRecoilValue(Dictionary);

  const { yMax, xMax, yMin, xMin } = useMemo(
    () => ({ xMin: xRange[0], xMax: xRange[1], yMin: yRange[0], yMax: yRange[1] }),
    [xRange, yRange]
  );

  const xScale = useMemo(() => {
    return scaleLinear<number>({
      domain: [xMin, xMax],
      range: [padding.left, width - padding.right],
      clamp: true
    });
  }, [xMax, width, padding]);

  const yScale = useMemo(() => {
    return scaleLinear<number>({
      domain: [yMin, yMax],
      range: [height - (padding.bottom + padding.top), padding.top],
      clamp: false
    });
  }, [yMax, height, padding]);

  const asksRef = useRef<SVGPathElement>(null);
  const bidsRef = useRef<SVGPathElement>(null);

  const { hideTooltip, showTooltip, tooltipData, tooltipLeft, tooltipTop } = useTooltip<ITooltipData>();

  const handleTooltip = useCallback(
    (event: React.TouchEvent<SVGGElement> | React.MouseEvent<SVGGElement>) => {
      const { x } = localPoint(event) || { x: 0, y: 0 };
      const asks = asksRef.current;
      const bids = bidsRef.current;
      let path: SVGPathElement | null;
      let type: 'bid' | 'ask' | 'hidden';
      if (!asksAscending[0] && !bidsDescending[0]) {
        hideTooltip();
        return
      }
      
      if (asksAscending[0] && x >= xScale(asksAscending[0][0])) {
        // asks
        path = asks;
        type = 'ask';
      } else if(bidsDescending[0] && x <= xScale(bidsDescending[0][0])) {
        path = bids;
        type = 'bid';
      } else {
        hideTooltip()
        return
      }
      if (path && path.getTotalLength() > 0) {
        const y = pointAtCoordinateX(path, x, 2);

        if (y) {
          showTooltip({
            tooltipData: {
              qty: yScale.invert(y),
              price: isPct ? xScale.invert(x) * 100 : xScale.invert(x),
              type,
              x,
              y
            },
            tooltipLeft: x,
            tooltipTop: y
          });
        } else {
          hideTooltip();
        }
      } else {
        hideTooltip();
      }
    },
    [height, width, bidsDescending, asksAscending, midPoint, asksRef.current, bidsRef.current]
  );

  return (
    <>
      <ScaleSVG height={height} width={width}>
        <Threshold
          id="asks"
          data={asksAscending}
          x={(d: PriceLevel) => xScale(d[0])}
          y0={(d: PriceLevel) => yScale(d[1])}
          y1={() => yScale(0)}
          clipAboveTo={0}
          clipBelowTo={0}
          aboveAreaProps={{
            fill: '#e36868',
            fillOpacity: 0.7
          }}
        />
        <LinePath
          stroke="#e36868"
          innerRef={asksRef}
          strokeWidth={1}
          data={asksAscending}
          x={d => xScale(d[0])}
          y={d => yScale(d[1])}
        />
        <Threshold
          id="bids"
          data={bidsDescending}
          x={(d: PriceLevel) => xScale(d[0])}
          y0={(d: PriceLevel) => yScale(d[1])}
          y1={() => yScale(0)}
          clipAboveTo={0}
          clipBelowTo={0}
          aboveAreaProps={{
            fill: '#84c1ca',
            fillOpacity: 0.7
          }}
        />
        <LinePath
          stroke="#84c1ca"
          innerRef={bidsRef}
          strokeWidth={1}
          data={bidsDescending}
          x={d => xScale(d[0])}
          y={d => yScale(d[1])}
        />
        {midPoint && (
          <Line
            stroke="#a79adb"
            strokeWidth={2}
            strokeDasharray="5"
            from={{ x: xScale(midPoint), y: padding.top + 48 /* leave extra space for the legend*/ }}
            to={{ x: xScale(midPoint), y: height - padding.top - padding.bottom }}
          />
        )}
        <AxisLeft
          key={dictionary.actions.swap.sellQuantity}
          label={dictionary.actions.swap.sellQuantity}
          left={padding.left}
          scale={yScale}
          numTicks={10}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 14, dx: 0, textAnchor: 'end' }}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'middle',
            dy: 4,
            dx: -12
          })}
        />
        <AxisBottom
          label={`${base.symbol} / ${quote.symbol}`}
          scale={xScale}
          top={height - padding.bottom - padding.top}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 12, dy: 15, textAnchor: 'middle' }}
          numTicks={10}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'end'
          })}
          tickFormat={x => (isPct ? `${(x.valueOf() * 100).toFixed(2)}%` : x.toString())}
        />
        {tooltipData && (
          <circle
            key={`radar-point`}
            cx={tooltipData.x}
            cy={tooltipData.y}
            r={3}
            fill={tooltipData.type === 'ask' ? '#e36868' : '#84c1ca'}
          />
        )}

        {height && height > 0 && width && width > 0 && (
          <Bar
            width={width - padding.left - padding.right}
            height={height - padding.top - padding.bottom}
            y={padding.top}
            x={padding.left}
            fill="transparent"
            onTouchStart={handleTooltip}
            onTouchMove={handleTooltip}
            onMouseMove={handleTooltip}
            onMouseLeave={() => hideTooltip()}
          />
        )}
      </ScaleSVG>
      {tooltipData && (
        <TooltipWithBounds top={tooltipTop} left={tooltipLeft} style={tooltipStyles}>
          <span>
            QTY: {tooltipData.qty.toFixed(-base.expo)} {base.symbol}
          </span>
          <span>
            {isPct ? 'Rate: ' : 'Price: '} {tooltipData.price.toFixed(-quote.expo)} {isPct ? '%' : quote.symbol}
          </span>
        </TooltipWithBounds>
      )}
    </>
  );
};
