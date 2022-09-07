import { useRecoilValue, useSetRecoilState } from 'recoil';
import reactStringReplace from 'react-string-replace';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentAccount, CurrentAccountName } from '../../state/user/accounts';
import { CurrentAction } from '../../state/actions/actions';
import { formatRiskIndicator } from '../../utils/format';
import { useCurrencyFormatting } from '../../utils/currency';
import { Button, Modal, Typography } from 'antd';
import { WarningFilled } from '@ant-design/icons';
import { RiskMeter } from '../misc/RiskMeter';

export function LiquidationModal(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter } = useCurrencyFormatting();
  const currentAccount = useRecoilValue(CurrentAccount);
  const currentAccountName = useRecoilValue(CurrentAccountName);
  const requiredFunds = currentAccount?.poolPositions?.USDC?.liquidationEndingCollateral.tokens ?? 0;
  const setCurrentAction = useSetRecoilState(CurrentAction);
  const { Title, Paragraph, Text } = Typography;

  // Set up liquidation detail text
  let liquidationDetail = reactStringReplace(
    dictionary.modals.alerts.liquidation.liquidationDetail,
    '{{ACCOUNT_NAME}}',
    () => <strong>{currentAccountName}</strong>
  );
  liquidationDetail = reactStringReplace(liquidationDetail, '{{BRING_RISK_DOWN}}', () => (
    <strong>{dictionary.modals.alerts.liquidation.bringRiskDown}</strong>
  ));

  if (currentAccount && currentAccount.isBeingLiquidated) {
    return (
      <Modal visible className="liquidation-modal" footer={null} closable={false}>
        <Title type="danger">
          <WarningFilled />
          {dictionary.modals.alerts.liquidation.title}
        </Title>
        <img src="img/ui/region.gif" width="100%" alt="Geobanned gif" />
        <RiskMeter showRange />
        <Paragraph className="liquidation-modal-detail" type="secondary">
          {liquidationDetail}
        </Paragraph>
        <Text type="danger">{currentAccountName?.toUpperCase()}</Text>
        <div className="liquidation-modal-stats flex-centered column">
          <div className="flex align-center justify-between">
            <Paragraph type="danger">{dictionary.common.riskLevel}</Paragraph>
            <Paragraph type="danger">{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Paragraph>
          </div>
          <div className="flex align-center justify-between">
            <Paragraph type="secondary">{dictionary.modals.alerts.liquidation.fundsRequired}</Paragraph>
            <Paragraph type="secondary">{`~ ${currencyFormatter(requiredFunds, true)}`}</Paragraph>
          </div>
        </div>
        <div className="liquidation-modal-actions flex-centered">
          <Button size="small" onClick={() => setCurrentAction('deposit')}>
            {dictionary.actions.deposit.title}
          </Button>
          <Button size="small" onClick={() => setCurrentAction('transfer')}>
            {dictionary.actions.transfer.title}
          </Button>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
