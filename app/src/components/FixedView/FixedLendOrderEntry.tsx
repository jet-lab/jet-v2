import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Typography } from 'antd';
import { FixedLendRowOrder } from '../../state/views/fixed-term';

export const FixedLendOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);

  const { Paragraph, Text } = Typography;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={rowOrder} setOrder={setRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.fixedView.lend.title}</Paragraph>
        </div>
      </div>
      <Text>Content will go here</Text>
    </div>
  );
};
