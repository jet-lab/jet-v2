import { scaleLinear, scaleOrdinal } from '@visx/scale';
import { LinePath } from '@visx/shape'
import { curveLinear } from '@visx/curve'
import { ParentSizeModern, ScaleSVG } from '@visx/responsive';
import { LegendOrdinal, LegendItem, LegendLabel } from '@visx/legend';
import { GridColumns } from '@visx/grid';
import { useMemo } from 'react';

interface ISeries {
    id: string;
    data: Array<{ x: number, y: number }>
}

const sampleData: ISeries = {
    id: 'series-1',
    data: [{
        x: 0,
        y: 0,
    }, {
        x: 1,
        y: 1,
    }, {
        x: 2,
        y: 10
    }]
}

const sampleData2: ISeries = {
    id: 'series-2',
    data: [{
        x: 0,
        y: 0.2,
    }, {
        x: 1,
        y: 3,
    }, {
        x: 2,
        y: 2
    }]
}

interface ILineChart {
    width: number
    height: number
    paddingTop: number
    series: ISeries[]
}

export const LineChart = ({
    height, width, paddingTop, series
}: ILineChart) => {
    const { xScale, yScale, ordinalColorScale } = useMemo(() => {
        const xScale = scaleLinear<number>({
            domain: [0, 2],
        });
        const yScale = scaleLinear<number>({
            domain: [10, 0],
        })
        xScale.range([0, width])
        yScale.range([paddingTop, height])

        const ordinalColorScale = scaleOrdinal({
            domain: series.map(s => s.id),
            range: ['#66d981', '#71f5ef', '#4899f1', '#7d81f6'],
        });
        return { xScale, yScale, ordinalColorScale }
    }, [width, height])
  
    return <>
        <ScaleSVG width={width} height={height}>
            {series.map(s => <LinePath
                key={s.id}
                curve={curveLinear}
                data={s.data}
                x={(d) => xScale(d.x) || 0}
                y={(d) => yScale(d.y) || 0}
                stroke={ordinalColorScale(s.id)}
                strokeWidth={2}
                strokeOpacity={1}
            />)}
            <GridColumns
                scale={xScale}
                height={height}
                strokeDasharray="1,3"
                stroke="#ffffff"
                strokeWidth={2}
                strokeOpacity={0.2}
                pointerEvents="none"
                top={paddingTop}
            />
        </ScaleSVG>
        <LegendOrdinal scale={ordinalColorScale} labelFormat={(label) => label}>
            {(labels) => {
                return (
                    <div className='chart-legend'>
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
                )
            }}
        </LegendOrdinal>
    </>
}

export const ResponsiveLineChart = ({

}) => {
    return <ParentSizeModern>
        {parent => <LineChart height={parent.height} width={parent.width} paddingTop={50} series={[sampleData, sampleData2]} />}
    </ParentSizeModern>
}