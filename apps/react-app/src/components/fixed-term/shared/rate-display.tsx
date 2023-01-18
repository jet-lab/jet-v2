export const RateDisplay = ({ rate }: { rate: number | undefined }) =>
  rate === undefined || isNaN(rate) ? null : <span>{(rate * 100).toFixed(3)}%</span>;
