import { useRecoilValue, useSetRecoilState } from 'recoil';
import reactStringReplace from 'react-string-replace';
import { Dictionary } from '../../state/settings/localization/localization';
import { AccountNames, CurrentAccount, CurrentAccountAddress } from '../../state/user/accounts';
import { CurrentAction } from '../../state/actions/actions';
import { formatRiskIndicator } from '../../utils/format';
import { useCurrencyFormatting } from '../../utils/currency';
import { Button, Modal, Typography } from 'antd';
import { WarningFilled } from '@ant-design/icons';
import { RiskMeter } from '../misc/RiskMeter';

// Modal to show user they are currently being liquidated
export function LiquidationModal(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter } = useCurrencyFormatting();
  const accountNames = useRecoilValue(AccountNames);
  const currentAccount = useRecoilValue(CurrentAccount);
  const currentAccountAddress = useRecoilValue(CurrentAccountAddress);
  const setCurrentAction = useSetRecoilState(CurrentAction);
  // Amount of USDC needed to end liquidation
  const requiredFunds = currentAccount?.poolPositions?.USDC?.liquidationEndingCollateral.tokens ?? 0;
  const { Title, Paragraph, Text } = Typography;

  // Set up liquidation detail text
  let liquidationDetail = reactStringReplace(
    dictionary.modals.alerts.liquidation.liquidationDetail,
    '{{ACCOUNT_NAME}}',
    () => <strong>{currentAccountAddress}</strong>
  );
  liquidationDetail = reactStringReplace(liquidationDetail, '{{BRING_RISK_DOWN}}', () => (
    <strong>{dictionary.modals.alerts.liquidation.bringRiskDown}</strong>
  ));

  // If our current account is being liquidated
  if (currentAccount && currentAccount.isBeingLiquidated) {
    return (
      <Modal visible className="liquidation-modal" maskClosable={false} footer={null} closable={false}>
        <Title type="danger">
          <WarningFilled />
          {dictionary.modals.alerts.liquidation.title}
        </Title>
        <img src="img/ui/region.gif" width="100%" alt="Geobanned gif" />
        <RiskMeter showRange />
        <Paragraph className="liquidation-modal-detail" type="secondary">
          {liquidationDetail}
        </Paragraph>
        <Text type="danger">{accountNames[currentAccountAddress]?.toUpperCase()}</Text>
        <div className="liquidation-modal-stats flex-centered column">
          <div className="flex align-center justify-between">
            <Paragraph type="danger">{dictionary.common.riskLevel}</Paragraph>
            <Paragraph type="danger">{formatRiskIndicator(currentAccount.riskIndicator)}</Paragraph>
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
