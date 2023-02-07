import { useMemo } from 'react';
import { useRecoilValue, useRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { PoolsViewOrder } from '@state/views/views';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Info } from '@components/misc/Info';
import { Typography } from 'antd';
import { PoolRow } from './PoolRow';
import { createDummyArray } from '@utils/ui';
import { useJetStore } from '@jet-lab/store';

// Table to display all Jet lending/borrowing pools
export function PoolsTable(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [poolsViewOrder, setPoolsViewOrder] = useRecoilState(PoolsViewOrder);
  const { pools } = useJetStore(({ pools, prices, selectedPoolKey, selectPool }) => ({
    pools,
    prices,
    selectedPoolKey,
    selectPool
  }));
  const { Paragraph, Text } = Typography;

  const poolsArray: string[] = useMemo(() => (pools ? Object.keys(pools) : createDummyArray(4, 'symbol')), [pools]);

  // Align columns
  const alignLeft: React.CSSProperties = { textAlign: 'left' };
  const alignRight: React.CSSProperties = { textAlign: 'right' };

  return (
    <div className="pools-table view-element">
      <div className="pools-table-head flex align-center justify-between">
        <Paragraph strong>{dictionary.poolsView.poolsTable.allAssets}</Paragraph>
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
              <PoolRow key={pool} address={pool} />
            ))}
          </tbody>
        </table>
      </div>
      <ReorderArrows component="poolsTable" order={poolsViewOrder} setOrder={setPoolsViewOrder} vertical />
    </div>
  );
}
