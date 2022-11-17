import { useRecoilState } from 'recoil';
import { Pool } from '@jet-lab/margin';
import { CurrentPoolSymbol } from '@state/pools/pools';
import { formatRate } from '@utils/format';
import { useCurrencyFormatting } from '@utils/currency';
import { TokenLogo } from '@components/misc/TokenLogo';
import { Skeleton, Typography } from 'antd';
import { Info } from 'app/src/components/misc/Info';

// Component for each row of the PoolsTable
export const PoolRow = (props: { pool: Pool }) => {
  const { pool } = props;
  const [currentPoolSymbol, setCurrentPoolSymbol] = useRecoilState(CurrentPoolSymbol);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const poolPrice = currencyFormatter(pool.tokenPrice, true);
  const { Text } = Typography;

  // Align columns
  const alignRight: React.CSSProperties = { textAlign: 'right' };

  // Returns the class list for the PoolRow
  function getRowClassnames() {
    return `ant-table-row ant-table-row-level-0 ${pool.symbol}-pools-table-row ${
      pool.symbol === currentPoolSymbol ? 'active' : ''
    }`;
  }

  // Renders pool asset info
  function renderAssetInfo() {
    let render = <Skeleton className="align-left" paragraph={false} active />;
    if (pool.tokenPrice >= 0) {
      if (pool.tokenPrice === 0) {
        render = (
          <>
            <Info term="pythDataStale">
              <div className="info-element">
                <Text
                  className="table-token-name"
                  strong
                  style={{ textDecoration: 'line-through', color: '#e36868' }}>
                  {pool.name}
                </Text>
                <Text className="table-token-abbrev" strong>
                  {pool.symbol}
                </Text>
                <Text
                  className="price-name"
                  style={{ textDecoration: 'line-through', color: '#e36868' }}>{`${pool.symbol} ≈ ${poolPrice}`}</Text>
                <Text
                  className="price-abbrev"
                  style={{ textDecoration: 'line-through', color: '#e36868' }}>{`≈ ${poolPrice}`}</Text>
              </div>
            </Info>
          </>
        );
      } else {
        render = (
          <div>
            <Text className="table-token-name" strong>
              {pool.name}
            </Text>
            <Text className="table-token-abbrev" strong>
              {pool.symbol}
            </Text>
            <Text className="price-name">{`${pool.symbol} ≈ ${poolPrice}`}</Text>
            <Text className="price-abbrev">{`≈ ${poolPrice}`}</Text>
          </div>
        );
      }
    }
    return render;
  }

  // Renders the utilization rate for the pool
  function renderUtilizationRate() {
    let render = <Skeleton className="align-right" paragraph={false} active />;
    if (!isNaN(pool.utilizationRate)) {
      render = <Text>{formatRate(pool.utilizationRate)}</Text>;
    }

    return render;
  }

  // Renders the borrowed tokens for the pool
  function renderBorrowedTokens() {
    let render = <Skeleton className="align-right" paragraph={false} active />;
    if (pool.borrowedTokens) {
      const tokensAbbrev = currencyAbbrev(pool.borrowedTokens.tokens, false, pool.tokenPrice, pool.precision);
      render = <Text>{`${tokensAbbrev}`}</Text>;
    }

    return render;
  }

  // Renders the available liquidity for the pool
  function renderAvailableLiquidity() {
    let render = <Skeleton className="align-right" paragraph={false} active />;
    if (pool.borrowedTokens) {
      const tokensAbbrev = currencyAbbrev(pool.vault.tokens, false, pool.tokenPrice, pool.precision);
      render = <Text>{`${tokensAbbrev}`}</Text>;
    }

    return render;
  }

  // Renders the borrow / deposit rates for the pool
  function renderLendingRate(side: 'borrow' | 'deposit') {
    let render = <Skeleton className="align-right" paragraph={false} active />;
    const rate = side === 'borrow' ? pool.borrowApr : pool.depositApy;
    if (!isNaN(rate)) {
      render = <Text type={side === 'borrow' ? 'danger' : 'success'}>{formatRate(rate)}</Text>;
    }

    return render;
  }

  return (
    <tr className={getRowClassnames()} onClick={() => setCurrentPoolSymbol(pool.symbol)} key={pool.symbol}>
      <td className="ant-table-cell" style={{ textAlign: 'left' }}>
        <div className="table-token">
          <TokenLogo height={22} symbol={pool?.symbol} />
          {renderAssetInfo()}
        </div>
      </td>
      <td className="ant-table-cell" style={alignRight}>
        {renderUtilizationRate()}
      </td>
      <td className="ant-table-cell" style={alignRight}>
        {renderBorrowedTokens()}
      </td>
      <td className="ant-table-cell" style={alignRight}>
        {renderAvailableLiquidity()}
      </td>
      <td className="ant-table-cell" style={alignRight}>
        {renderLendingRate('deposit')}
      </td>
      <td className="ant-table-cell" style={alignRight}>
        {renderLendingRate('borrow')}
      </td>
    </tr>
  );
};
