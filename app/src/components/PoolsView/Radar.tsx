import { useEffect, useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import axios from 'axios';
import { Dictionary } from '../../state/settings/localization/localization';
import { PoolsRowOrder } from '../../state/views/views';
import { CurrentPool } from '../../state/pools/pools';
import { Skeleton, Table, Typography } from 'antd';
import { ReorderArrows } from '../misc/ReorderArrows';

// Component to display interest rate comparisons across protocols
export function Radar(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [poolsRowOrder, setPoolsRowOrder] = useRecoilState(PoolsRowOrder);
  const currentPool = useRecoilValue(CurrentPool);
  const [loaded, setLoaded] = useState(false);
  const { Paragraph } = Typography;

  // Track these protocols
  const [rates, setRates] = useState([
    {
      key: 'jet',
      rates: {} as any
    },
    {
      key: 'mango',
      rates: {} as any
    },
    {
      key: 'port',
      rates: {} as any
    },
    {
      key: 'solend',
      rates: {} as any
    }
  ]);

  // Table data
  const radarTableColumns = [
    {
      title: dictionary.poolsView.radar.protocol,
      dataIndex: 'key',
      key: 'protocol',
      align: 'left' as any,
      render: (key: string) => <img width="60px" height="auto" src={`img/protocols/${key}_white.png`} alt={key} />
    },
    {
      title: dictionary.common.depositRate,
      dataIndex: 'rates',
      key: 'deposit',
      align: 'right' as any,
      render: (rates: any) =>
        !loaded ? (
          <Skeleton className="align-right" paragraph={false} active />
        ) : currentPool?.symbol && typeof rates[currentPool.symbol]?.depositRate === 'number' ? (
          `${Math.ceil(rates[currentPool.symbol].depositRate * 100 * 100) / 100}%`
        ) : (
          <Paragraph className="not-available-text" italic>
            {dictionary.common.notAvailable}
          </Paragraph>
        )
    },
    {
      title: dictionary.common.borrowRate,
      dataIndex: 'rates',
      key: 'borrow',
      align: 'right' as any,
      render: (rates: any) =>
        !loaded ? (
          <Skeleton className="align-right" paragraph={false} active />
        ) : currentPool?.symbol && typeof rates[currentPool.symbol]?.borrowRate === 'number' ? (
          `${Math.ceil(rates[currentPool.symbol].borrowRate * 100 * 100) / 100}%`
        ) : (
          <Paragraph className="not-available-text" italic>
            {dictionary.common.notAvailable}
          </Paragraph>
        )
    }
  ];

  // Fetch rates on mount
  useEffect(() => {
    axios
      .get('https://api.jetprotocol.io/v1/radar')
      .then(({ data }) => {
        if (data) {
          const rates = Object.keys(data).map(protocol => ({
            key: protocol,
            rates: data[protocol]
          }));
          setRates(rates);
          setLoaded(true);
        }
      })
      .catch(err => err);
  }, []);

  return (
    <div className="radar view-element flex align-center justify-start column">
      <div className="radar-head flex-centered">
        <ReorderArrows component="radar" order={poolsRowOrder} setOrder={setPoolsRowOrder} />
        <Paragraph strong>{dictionary.poolsView.radar.interestRadar}</Paragraph>
      </div>
      <Table
        dataSource={rates}
        columns={radarTableColumns}
        className="no-row-interaction"
        rowKey={row => `${row.key}-${Math.random()}`}
        pagination={false}
      />
    </div>
  );
}
