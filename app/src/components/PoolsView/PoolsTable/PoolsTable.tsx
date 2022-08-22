import { useSetRecoilState, useRecoilValue, useRecoilState } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { PoolsViewOrder } from '../../../state/views/views';
import { FilteredPoolsList, PoolsTextFilter } from '../../../state/borrow/pools';
import { Input, Typography } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { ReorderArrows } from '../../misc/ReorderArrows';
import { Info } from '../../misc/Info';
import { PoolRow } from './PoolRow';

export function PoolsTable(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [poolsViewOrder, setPoolsViewOrder] = useRecoilState(PoolsViewOrder);
  const filteredPoolsList = useRecoilValue(FilteredPoolsList);
  const setFilter = useSetRecoilState(PoolsTextFilter);

  const { Paragraph, Text } = Typography;

  return (
    <div className="pools-table view-element view-element-hidden">
      <div className="pools-table-head view-element-item view-element-item-hidden flex align-center justify-between">
        <Paragraph strong>{dictionary.poolsView.poolsTable.allAssets}</Paragraph>
        <div className="account-table-search">
          <SearchOutlined />
          <Input
            type="text"
            placeholder={dictionary.poolsView.poolsTable.searchExample}
            onChange={e => setFilter(e.target.value)}
          />
        </div>
      </div>
      <div className="ant-table">
        <table style={{ tableLayout: 'auto' }}>
          <thead className="ant-table-thead">
            <tr>
              <th className="ant-table-cell" style={{ textAlign: 'left' }}>
                {dictionary.common.token}
              </th>
              <th>
                {
                  <Info term="collateralWeight">
                    <Text className="info-element">{dictionary.poolsView.collateralWeight}</Text>
                  </Info>
                }
              </th>
              <th className="ant-table-cell" style={{ textAlign: 'right' }}>
                <Info term="utilizationRate">
                  <Text className="info-element">{dictionary.poolsView.utilizationRate}</Text>
                </Info>
              </th>
              <th className="ant-table-cell" style={{ textAlign: 'right' }}>
                {dictionary.poolsView.totalBorrowed}
              </th>
              <th className="ant-table-cell" style={{ textAlign: 'right' }}>
                {dictionary.poolsView.availableLiquidity}
              </th>
              <th className="ant-table-cell" style={{ textAlign: 'right' }}>
                <Info term="depositBorrowRate">
                  <Text className="info-element">{dictionary.accountsView.depositBorrowRates}</Text>
                </Info>
              </th>
              <th className="ant-table-cell" style={{ textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody className="ant-table-tbody">
            {filteredPoolsList.map(pool => {
              return <PoolRow key={pool.symbol} pool={pool} />;
            })}
          </tbody>
        </table>
      </div>
      <ReorderArrows component="poolsTable" order={poolsViewOrder} setOrder={setPoolsViewOrder} vertical />
    </div>
  );
}
