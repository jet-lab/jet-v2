import { useState, useEffect } from 'react';
import { Arc } from '@visx/shape';
import { Group } from '@visx/group';

interface PoolDetailsPieChart {
  percentage: number;
}

export function PieChart({ percentage }: PoolDetailsPieChart): JSX.Element {
  const [percent, setPercent] = useState(0);

  // Animate chart
  useEffect(() => {
    const animate = () => {
      if (percent < percentage) {
        setPercent(percent + 1);
      } else if (percent > percentage) {
        setPercent(percent - 1);
      }
    };

    const animationRef = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animationRef);
  }, [percentage, percent]);

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
