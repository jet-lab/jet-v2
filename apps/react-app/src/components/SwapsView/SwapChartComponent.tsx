import { AxisBottom, AxisLeft } from '@visx/axis';
import { scaleLinear } from '@visx/scale';
import { useMemo } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { ScaleSVG } from '@visx/responsive';
import { Line, AreaClosed } from '@visx/shape';
import { Tooltip, useTooltip } from '@visx/tooltip';

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
  priceRange?: [min: number, max: number];
  liquidityRange?: [min: number, max: number];
}

export const SwapChartComponent = ({
  height,
  width,
  padding = { top: 20, left: 80, right: 32, bottom: 60 },
  bids,
  asks,
  oraclePrice,
  priceRange = [0, 0],
  liquidityRange = [0, 0]
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

  const { tooltipData, tooltipLeft, tooltipTop, tooltipOpen, showTooltip, hideTooltip } = useTooltip();

  console.log(tooltipData, tooltipLeft, tooltipTop, tooltipOpen, showTooltip, hideTooltip);

  return (
    <>
      <Tooltip />
      <ScaleSVG height={height} width={width}>
        <AreaClosed fill="#84c1ca" yScale={yScale} data={bids} x={d => xScale(d[0])} y={d => yScale(d[1])} />
        <AreaClosed fill="#e36868" yScale={yScale} data={asks} x={d => xScale(d[0])} y={d => yScale(d[1])} />
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
    </>
  );
};
