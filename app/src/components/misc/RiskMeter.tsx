import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { AccountsInit, CurrentAccount } from '../../state/user/accounts';
import { useRiskLevel } from '../../utils/risk';
import { Typography } from 'antd';

export function RiskMeter(props: { detail?: boolean; showRange?: boolean }): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const accountsInit = useRecoilValue(AccountsInit);
  const currentAccount = useRecoilValue(CurrentAccount);
  const riskLevel = useRiskLevel();
  const [percentage, setPercentage] = useState(0);
  const { Text } = Typography;

  // Adjust risk meter based on label
  useEffect(() => {
    if (!accountsInit) {
      setPercentage(0);
      return;
    }

    setPercentage((currentAccount?.riskIndicator ?? 0) * 100);
  }, [accountsInit, riskLevel, currentAccount?.riskIndicator]);

  return (
    <div className="risk-meter flex-centered column">
      <div className="risk-meter-bar">
        {accountsInit ? (
          <div className="risk-meter-bar-indicator flex-centered column" style={{ left: `${percentage}%` }}>
            <div className={`risk-meter-bar-indicator-arrow risk-meter-bar-indicator-arrow-${riskLevel}`}></div>
            {props.detail && (
              <Text className="risk-meter-bar-indicator-label" style={riskLevel === 'low' ? { left: 0 } : {}}>
                {dictionary.accountsView.riskMeter[riskLevel]?.toUpperCase()}
              </Text>
            )}
          </div>
        ) : (
          <></>
        )}
      </div>
      {props.showRange && (
        <div className="risk-meter-range flex align-center justify-between">
          <Text type="success">0</Text>
          <Text type="danger">1</Text>
        </div>
      )}
    </div>
  );
}
