import { useState, useEffect } from 'react';
import { useTradeContext } from '../contexts/tradeContext';
import { Info } from './Info';

export function PercentageChart(props: { percentage: number; text: string; term?: string }): JSX.Element {
  const { currentPool } = useTradeContext();
  const [percent, setPercent] = useState(0);

  // Animate chart
  useEffect(() => {
    const animate = setInterval(() => {
      if (percent < props.percentage) {
        setPercent(p => p + 1);
      }
    }, 7);
    return () => clearInterval(animate);
  }, [props.percentage, percent]);

  // Reset on reserve change
  useEffect(() => {
    setPercent(0);
  }, [currentPool]);

  return (
    <div className="percentage-chart">
      <svg viewBox="0 0 36 36">
        <path
          d="M18 2.0845
            a 15.9155 15.9155 0 0 1 0 31.831
            a 15.9155 15.9155 0 0 1 0 -31.831"
        />
        <path
          strokeDasharray={`${percent}, 100`}
          d="M18 2.0845
            a 15.9155 15.9155 0 0 1 0 31.831
            a 15.9155 15.9155 0 0 1 0 -31.831"
        />
      </svg>
      <div className="inset-chart-shadow"></div>
      <div className="chart-info flex-centered column">
        <h1 className="modal-header">
          {props.percentage > 1 ? Math.floor(props.percentage) : Math.ceil(props.percentage)}%
        </h1>
        {props.text && (
          <span>
            {props.text}
            {props.term && <Info term={props.term} />}
          </span>
        )}
      </div>
    </div>
  );
}
