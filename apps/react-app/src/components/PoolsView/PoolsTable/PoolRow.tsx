import { TokenLogo } from '@components/misc/TokenLogo';
import { useJetStore } from '@jet-lab/store';
import { UtilizationRate } from './UtilizationRate';
import { AvailableLiquidity } from './AvailableLiquidity';
import { BorrowedTokens } from './BorrowedTokens';
import { AssetInfo } from './AssetInfo';
import { LendingRate } from './LendingRate';

interface PoolRow {
  address: string;
}

// Component for each row of the PoolsTable
export const PoolRow = ({ address }: PoolRow) => {
  const { pool, prices, selectedPoolKey, selectPool } = useJetStore(state => ({
    pool: state.pools[address],
    prices: state.prices && state.pools[address] && state.prices[state.pools[address].token_mint],
    selectPool: state.selectPool,
    selectedPoolKey: state.selectedPoolKey
  }));

  const price = prices?.price ? prices.price : 0;
  const emaPrice = prices?.ema ? prices.ema : 0;

  // Align columns
  const alignRight: React.CSSProperties = { textAlign: 'right' };

  // Returns the class list for the PoolRow
  function getRowClassnames() {
    return `ant-table-row ant-table-row-level-0 ${pool.symbol}-pools-table-row ${
      pool.address === selectedPoolKey ? 'active' : ''
    }`;
  }

  return (
    <tr className={getRowClassnames()} onClick={() => selectPool(pool.address)} key={pool.symbol}>
      <td className="ant-table-cell" style={{ textAlign: 'left' }}>
        <div className="table-token">
          <TokenLogo height={22} symbol={pool?.symbol} />
          <AssetInfo pool={pool} price={price} ema={emaPrice} />
        </div>
      </td>
      <td className="ant-table-cell" style={alignRight}>
        <UtilizationRate pool={pool} />
      </td>
      <td className="ant-table-cell" style={alignRight}>
        <BorrowedTokens pool={pool} />
      </td>
      <td className="ant-table-cell" style={alignRight}>
        <AvailableLiquidity pool={pool} />
      </td>
      <td className="ant-table-cell" style={alignRight}>
        <LendingRate side="deposit" pool={pool} />
      </td>
      <td className="ant-table-cell" style={alignRight}>
        <LendingRate side="borrow" pool={pool} />
      </td>
    </tr>
  );
};
