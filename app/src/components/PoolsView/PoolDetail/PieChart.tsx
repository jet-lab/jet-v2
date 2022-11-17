import { useState, useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { CurrentPool } from '@state/pools/pools';
import { Arc } from '@visx/shape';
import { Group } from '@visx/group';

// Pie Chart component showing a percentage value
export function PieChart(props: {
  // Percentage of data to fill in chart
  percentage: number;
  // Data-related text
  text: string;
  // Optionally add a term's definition of tooltip
  term?: string;
}): JSX.Element {
  const currentPool = useRecoilValue(CurrentPool);
  const [percent, setPercent] = useState(0);

  // Animate chart
  useEffect(() => {
    const percentage = props.percentage * 100;
    const animate = setInterval(() => {
      if (percent < percentage) {
        setPercent(p => p + 1);
      }
    }, 5);
    return () => clearInterval(animate);
  }, [props.percentage, percent]);

  // Reset on pool change
  useEffect(() => {
    setPercent(0);
  }, [currentPool]);

  return (
    <>
      <div className="pie-chart">
        <svg width="100%" height="100%">
          <Group transform="translate(37.5, 37.5)">
            <Arc
              data={percent}
              startAngle={0}
              endAngle={percent * 0.063}
              outerRadius={38}
              innerRadius={0}
              padAngle={0}
              cornerRadius={0}
              fill={'#e36868'}></Arc>
          </Group>
        </svg>
      </div>
    </>
  );
}
