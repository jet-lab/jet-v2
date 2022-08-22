import { Button, Modal, Typography } from 'antd';
import { WarningFilled } from '@ant-design/icons';
import { useLanguage } from '../contexts/localization/localization';
import { currencyFormatter } from '../utils/currency';
import { useMargin } from '../contexts/marginContext';
import { HealthBar } from './HealthBar';
import { useTradeContext } from '../contexts/tradeContext';
import { useLiquidationModal } from '../contexts/LiquidationModal';

export function LiquidationModal(): JSX.Element {
  const { dictionary } = useLanguage();
  const { marginAccount } = useMargin();
  const { setCurrentAction } = useTradeContext();
  const { open, setOpen, setClosed } = useLiquidationModal();
  const requiredFunds = marginAccount?.poolPositions?.USDC?.liquidationEndingCollateral.tokens ?? 0;
  const { Title, Paragraph } = Typography;

  if (open) {
    return (
      <Modal visible className="liquidation-modal" footer={null} closable={false}>
        <Title type="danger">
          <WarningFilled />
          {dictionary.copilot.alert.liquidation.title}
        </Title>
        <img src="img/ui/region.gif" width="100%" alt="Geobanned gif" />
        <HealthBar />
        <Paragraph className="liquidation-modal-detail" type="secondary">
          {dictionary.copilot.alert.liquidation.liquidationDetail}
        </Paragraph>
        <div className="liquidation-modal-stats flex-centered column">
          <div className="flex align-center justify-between">
            <Paragraph type="danger">{dictionary.cockpit.yourRisk}</Paragraph>
            <Paragraph type="danger">{marginAccount?.riskIndicator.toFixed(2)}</Paragraph>
          </div>
          <div className="flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.copilot.alert.liquidation.fundsRequired}</Paragraph>
            <Paragraph type="secondary">{`~${currencyFormatter(requiredFunds, true)}`}</Paragraph>
          </div>
        </div>
        <div className="liquidation-modal-actions flex-centered">
          <Button
            block
            size="small"
            onClick={() => {
              setCurrentAction('deposit');
              setOpen(false);
              setClosed(true);
            }}>
            {dictionary.cockpit.deposit}
          </Button>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
