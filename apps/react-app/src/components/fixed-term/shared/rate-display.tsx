export const RateDisplay = ({ rate }: { rate: number | undefined }) =>
  rate === undefined || isNaN(rate) ? <span>0</span> : <span>{(rate * 100).toFixed(3)}%</span>;
