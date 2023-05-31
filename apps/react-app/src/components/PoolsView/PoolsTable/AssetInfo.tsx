import { Info } from '@components/misc/Info';
import { Skeleton, Typography } from 'antd';

// Renders pool asset info
export const AssetInfo = ({ pool, price, ema }: { pool: PoolData; price: number; ema: number }) => {
  if (price > 0) {
    return (
      <div>
        <Typography.Text className="table-token-name" strong>
          {pool.name}
        </Typography.Text>
        <Typography.Text className="table-token-abbrev" strong>
          {pool.symbol}
        </Typography.Text>
        <Typography.Text className="price-name">{`${pool.symbol} ≈ ${price.toFixed(pool.precision)}`}</Typography.Text>
        <Typography.Text className="price-abbrev">{`≈ ${price}`}</Typography.Text>
      </div>
    );
  }
  if (price === 0) {
    return (
      <Info term="pythDataStale">
        <div className="info-element">
          <Typography.Text className="table-token-name table-token-disabled">{pool.name}</Typography.Text>
          <Typography.Text className="table-token-abbrev table-token-disabled">{pool.symbol}</Typography.Text>
          <Typography.Text className="price-name table-token-disabled">{`${pool.symbol} ≈ ${ema.toFixed(
            pool.precision
          )}`}</Typography.Text>
          <Typography.Text className="price-abbrev table-token-disabled">{`≈ ${ema}`}</Typography.Text>
        </div>
      </Info>
    );
  }
  return <Skeleton className="align-left" paragraph={false} active />;
};
