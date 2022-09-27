import { useRecoilState, useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { ReorderArrows } from '../misc/ReorderArrows';
import { Button, Input, Typography } from 'antd';
import { FixedLendRowOrder } from '../../state/views/fixed-term';
import { FixedMarketAtom } from '../../state/fixed/fixed-term-market-sync';
import { CurrentAccount } from '../../state/user/accounts';

export const FixedLendOrderEntry = () => {
  const dictionary = useRecoilValue(Dictionary);
  const [rowOrder, setRowOrder] = useRecoilState(FixedLendRowOrder);
  const market = useRecoilValue(FixedMarketAtom);
  const marginAccount = useRecoilValue(CurrentAccount);

  const { Paragraph } = Typography;

  return (
    <div className="order-entry fixed-lend-entry view-element view-element-hidden flex column">
      <div className="order-entry-head view-element-item view-element-item-hidden flex column">
        <ReorderArrows component="fixedLendEntry" order={rowOrder} setOrder={setRowOrder} />
        <div className="order-entry-head-top flex-centered">
          <Paragraph className="order-entry-head-top-title">{dictionary.fixedView.lend.title}</Paragraph>
        </div>
      </div>
      <div className="order-entry-body">
        <Input placeholder="enter order value" type="number" />
        <Input placeholder="enter interest" type="number" />
        <Button>Create Lend Order</Button>
      </div>
    </div>
  );
};
