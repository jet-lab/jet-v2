import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Typography } from 'antd';
import { FixedBorrowRowOrder } from '../../state/views/fixed-term';

export const FixedBorrowOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [swapsRowOrder, setSwapsRowOrder] = useRecoilState(FixedBorrowRowOrder);

  const { Paragraph } = Typography;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={swapsRowOrder} setOrder={setSwapsRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.fixedView.borrow.title}</Paragraph>
        </div>
      </div>
    </div>
  );
};
