import { useState } from 'react';
import { useRecoilValue, useRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { PoolsViewOrder } from '@state/views/views';
import { FilteredPools, PoolOptions } from '@state/pools/pools';
import { createDummyArray } from '@utils/ui';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Info } from '@components/misc/Info';
import { Input, Typography } from 'antd';
import { PoolRow } from './PoolRow';
import { SearchOutlined } from '@ant-design/icons';
import debounce from 'lodash.debounce';

// Table to display all Jet lending/borrowing pools
export function PoolsTable(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [poolsViewOrder, setPoolsViewOrder] = useRecoilState(PoolsViewOrder);
  const [filterText, setFilterText] = useState('');
  const filteredPools = useRecoilValue(FilteredPools(filterText));
  const poolOptions = useRecoilValue(PoolOptions);
  const placeholderArrayLength = Object.keys(poolOptions).length > 0 ? Object.keys(poolOptions).length : 4;
  const poolsArray = filteredPools.length ? filteredPools : createDummyArray(placeholderArrayLength, 'symbol');
  const { Paragraph, Text } = Typography;

  // Align columns
  const alignLeft: React.CSSProperties = { textAlign: 'left' };
  const alignRight: React.CSSProperties = { textAlign: 'right' };

  return (
    <div className="pools-table view-element">
      <div className="pools-table-head flex align-center justify-between">
        <Paragraph strong>{dictionary.poolsView.poolsTable.allAssets}</Paragraph>
        <div className="account-table-search">
          <SearchOutlined />
          <Input
            type="text"
            placeholder={dictionary.poolsView.poolsTable.searchExample}
            onChange={debounce(e => setFilterText(e.target.value), 300)}
          />
        </div>
      </div>
      <div className="ant-table">
        <table style={{ tableLayout: 'auto' }}>
          <thead className="ant-table-thead">
            <tr>
              <th className="ant-table-cell" style={alignLeft}>
                {dictionary.common.token}
              </th>
              <th className="ant-table-cell" style={alignRight}>
                <Info term="utilizationRate">
                  <Text className="info-element">{dictionary.poolsView.utilizationRate}</Text>
                </Info>
              </th>
              <th className="ant-table-cell" style={alignRight}>
                {dictionary.poolsView.totalBorrowed}
              </th>
              <th className="ant-table-cell" style={alignRight}>
                {dictionary.poolsView.availableLiquidity}
              </th>
              <th className="ant-table-cell" style={alignRight}>
                <Info term="depositRate">
                  <Text className="info-element">{dictionary.accountsView.depositRate}</Text>
                </Info>
              </th>
              <th className="ant-table-cell" style={alignRight}>
                <Info term="borrowRate">
                  <Text className="info-element">{dictionary.accountsView.borrowRate}</Text>
                </Info>
              </th>
            </tr>
          </thead>
          <tbody className="ant-table-tbody">
            {poolsArray.map(pool => (
              <PoolRow key={pool.symbol} pool={pool} />
            ))}
          </tbody>
        </table>
      </div>
      <ReorderArrows component="poolsTable" order={poolsViewOrder} setOrder={setPoolsViewOrder} vertical />
    </div>
  );
}
