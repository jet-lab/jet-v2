import { LegendItem, LegendLabel, LegendOrdinal } from '@visx/legend';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { scaleLinear, scaleOrdinal } from '@visx/scale';
import { useMemo, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ScaleSVG } from '@visx/responsive';
import { Line, AreaClosed } from '@visx/shape';

const ordinalColorScale = scaleOrdinal({
  domain: ['Asks', 'Bids', 'Oracle Price'],
  range: ['#e36868', '#84c1ca', '#a79adb']
});

interface SwapChartComponentProps {
  height: number;
  width: number;
  padding?: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  bids?: [price: number, amt: number][];
  asks?: [price: number, amt: number][];
  oraclePrice: number;
}

export const SwapChartComponent = ({
  height,
  width,
  padding = { top: 20, left: 80, right: 32, bottom: 60 },
  bids,
  asks,
  oraclePrice
}: SwapChartComponentProps) => {
  const dictionary = useRecoilValue(Dictionary);

  const { yMax, xMax, yMin, xMin } = useMemo(() => {
    if (!bids || !asks || bids.length === 0 || asks.length === 0) {
      return {
        xMin: 0,
        xMax: 0,
        yMin: 0,
        yMax: 0
      };
    } else {
      return {
        xMin: bids[bids.length - 1][0],
        xMax: asks[asks.length - 1][0],
        yMin: Math.min(asks[0][1], bids[0][1]),
        yMax: Math.max(asks[asks.length - 1][1], bids[bids.length - 1][1])
      };
    }
  }, [bids, asks]);

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

  return (
    <ScaleSVG height={height} width={width}>
      <AreaClosed fill="#e36868" yScale={yScale} data={bids} x={d => xScale(d[0])} y={d => yScale(d[1])} />
      <AreaClosed fill="#84c1ca" yScale={yScale} data={asks} x={d => xScale(d[0])} y={d => yScale(d[1])} />
      <Line
        stroke="#a79adb"
        strokeWidth={2}
        from={{ x: xScale(oraclePrice), y: padding.top }}
        to={{ x: xScale(oraclePrice), y: height - padding.top - padding.bottom }}
      />
      <AxisLeft
        key={dictionary.actions.swap.sellQuantity}
        label={dictionary.actions.swap.sellQuantity}
        left={padding.left}
        scale={yScale}
        numTicks={10}
        labelProps={{ fill: 'rgb(199, 199, 199)', fontSize: 14, dx: 10, textAnchor: 'end' }}
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
        label={`x axis label ${width}`}
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
    </ScaleSVG>
  );
};
