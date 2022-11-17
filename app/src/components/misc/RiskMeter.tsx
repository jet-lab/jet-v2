import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { CurrentAccount } from '@state/user/accounts';
import { useRiskLevel } from '@utils/risk';
import { Typography } from 'antd';

// Component to show account's Risk Level shown as a meter
export function RiskMeter(props: { showRange?: boolean }): JSX.Element {
  const currentAccount = useRecoilValue(CurrentAccount);
  const riskLevel = useRiskLevel();
  const [percentage, setPercentage] = useState(0);
  const { Text } = Typography;

  // Adjust risk meter based on indicator for current account
  useEffect(() => {
    // If account isn't loaded, default to 0
    let riskIndicator = 0;
    if (currentAccount) {
      riskIndicator = currentAccount.riskIndicator;
    }

    setPercentage(riskIndicator * 100);
  }, [currentAccount]);

  // Renders the risk meter's arrow
  function renderMeterArrow() {
    let render = <></>;
    if (currentAccount) {
      render = (
        <div className="risk-meter-bar-indicator flex-centered column" style={{ left: `${percentage}%` }}>
          <div className={`risk-meter-bar-indicator-arrow risk-meter-bar-indicator-arrow-${riskLevel}`}></div>
        </div>
      );
    }

    return render;
  }

  // Renders the risk meter range
  function renderMeterRange() {
    let render = <></>;
    if (props.showRange) {
      render = (
        <div className="risk-meter-range flex align-center justify-between">
          <Text type="success">0</Text>
          <Text type="danger">1</Text>
        </div>
      );
    }

    return render;
  }

  return (
    <div className="risk-meter flex-centered column">
      <div className="risk-meter-bar">{renderMeterArrow()}</div>
      {renderMeterRange()}
    </div>
  );
}
