import { scaleLinear } from '@visx/scale';
import { LinePath } from '@visx/shape'
import { curveLinear } from '@visx/curve'
import { ParentSizeModern, ScaleSVG } from '@visx/responsive';
import { useMemo } from 'react';




const sampleData = [{
    x: 0,
    y: 0,
}, {
    x: 1,
    y: 1,
}, {
    x: 2,
    y: 10
}]


interface ILineChart {
    width: number
    height: number
    paddingTop: number
}
export const LineChart = ({
    height, width, paddingTop
}: ILineChart) => {
    const { xScale, yScale } = useMemo(() => {
        const xScale = scaleLinear<number>({
            domain: [0, 2],
        });
        const yScale = scaleLinear<number>({
            domain: [10, 0],
        })
        xScale.range([0, width])
        yScale.range([paddingTop, height])
        return { xScale, yScale }
    }, [width, height])

    return <ScaleSVG width={width} height={height}>
        <LinePath
            curve={curveLinear}
            data={sampleData}
            x={(d) => xScale(d.x) || 0}
            y={(d) => yScale(d.y) || 0}
            stroke="#884444"
            strokeWidth={2}
            strokeOpacity={1}
        />
    </ScaleSVG>
}

export const ResponsiveLineChart = ({

}) => {
    return <ParentSizeModern>
        {parent => <LineChart height={parent.height} width={parent.width} paddingTop={60} />}
    </ParentSizeModern>
}