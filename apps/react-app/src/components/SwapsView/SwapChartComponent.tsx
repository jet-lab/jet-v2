import { AxisBottom, AxisLeft } from '@visx/axis';
import { scaleLinear } from '@visx/scale';
import { useCallback, useMemo, useRef } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ScaleSVG } from '@visx/responsive';
import { Line, Bar, LinePath } from '@visx/shape';
import { TooltipWithBounds, defaultStyles, useTooltip } from '@visx/tooltip';
import { Threshold } from '@visx/threshold';
import { PriceLevel, SwapLiquidityTokenInfo } from '@jet-lab/store';
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
  type: 'ask' | 'bid';
}
interface SwapChartComponentProps {
  height: number;
  width: number;
  padding?: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  bids: [price: number, amt: number][];
  asks: [price: number, amt: number][];
  oraclePrice: number;
  priceRange: [min: number, max: number];
  liquidityRange: [min: number, max: number];
  base: SwapLiquidityTokenInfo;
  quote: SwapLiquidityTokenInfo;
}

export const SwapChartComponent = ({
  height,
  width,
  padding = { top: 20, left: 80, right: 32, bottom: 60 },
  bids = [],
  asks = [],
  oraclePrice,
  priceRange = [0, 0],
  liquidityRange = [0, 0],
  base,
  quote
}: SwapChartComponentProps) => {
  const dictionary = useRecoilValue(Dictionary);

  const { yMax, xMax, yMin, xMin } = useMemo(
    () => ({ xMin: priceRange[0], xMax: priceRange[1], yMin: liquidityRange[0], yMax: liquidityRange[1] }),
    [priceRange, liquidityRange]
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
      range: [height - (padding.top + padding.bottom), padding.top],
      clamp: true
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
      const oraclePriceX = xScale(oraclePrice);
      let path: SVGPathElement | null;
      let type: 'bid' | 'ask';

      if (x <= oraclePriceX) {
        // bids
        path = bids;
        type = 'bid';
      } else {
        // asks
        path = asks;
        type = 'ask';
      }
      if (path && path.getTotalLength() > 0) {
        const y = pointAtCoordinateX(path, x, 5);
        if (y) {
          showTooltip({
            tooltipData: {
              qty: yScale.invert(y),
              price: xScale.invert(x),
              type
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
    [height, width, bids, asks, oraclePrice, asksRef, bidsRef]
  );

  return (
    <>
      <ScaleSVG height={height} width={width}>
        <Threshold
          id="bids"
          data={bids}
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
          strokeWidth={2}
          data={bids}
          x={d => xScale(d[0])}
          y={d => yScale(d[1])}
        />

        <Threshold
          id="asks"
          data={asks}
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
          strokeWidth={2}
          data={asks}
          x={d => xScale(d[0])}
          y={d => yScale(d[1])}
        />

        <Line
          stroke="#a79adb"
          strokeWidth={2}
          strokeDasharray="5"
          from={{ x: xScale(oraclePrice), y: padding.top + 48 /* leave extra space for the legend*/ }}
          to={{ x: xScale(oraclePrice), y: height - padding.top - padding.bottom }}
        />
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
          top={height - (padding.bottom + padding.top)}
          labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 12, dy: 15, textAnchor: 'middle' }}
          numTicks={10}
          tickLabelProps={() => ({
            fontSize: 10,
            fill: '#fff',
            opacity: 0.6,
            textAnchor: 'end'
          })}
        />
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
            Price: {tooltipData.price.toFixed(-quote.expo)} {quote.symbol}
          </span>
        </TooltipWithBounds>
      )}
    </>
  );
};
