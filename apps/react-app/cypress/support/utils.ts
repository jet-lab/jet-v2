export const formatWithCommas = <T>(value: T) => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
