export const RateDisplay = ({ rate }: { rate: number | undefined }) =>
  rate === undefined || isNaN(rate) ? <>0</> : <span>{(rate * 100).toFixed(3)}%</span>;
