import { useState, useEffect } from 'react';
import { useRecoilValue } from 'recoil';
import { CurrentPool } from '@state/pools/pools';

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
    <div className="pie-chart">
      <svg viewBox="0 0 36 36">
        <path
          strokeDasharray={`${percent}, 100`}
          d="M18 2.0845
            a 15.9155 15.9155 0 0 1 0 31.831
            a 15.9155 15.9155 0 0 1 0 -31.831"
        />
      </svg>
    </div>
  );
}
