import { useEffect, useState } from 'react';
import { MarginAccount } from '@jet-lab/margin';
import { useLanguage } from '../contexts/localization/localization';
import { useMargin } from '../contexts/marginContext';

export function HealthBar(props: { fullDetail?: boolean }): JSX.Element {
  const { dictionary } = useLanguage();
  const { marginAccount } = useMargin();
  const [healthGauge, setHealthGauge] = useState<Record<string, number | string>>({
    percentage: 0,
    standing: ''
  });

  // Range of health meter is liquidation (125%) to 200%
  useEffect(() => {
    if (!marginAccount) {
      return;
    }

    const { riskIndicator } = marginAccount;

    if (riskIndicator >= MarginAccount.RISK_CRITICAL_LEVEL) {
      setHealthGauge({
        percentage: 95,
        standing: 'critical'
      });
    } else if (
      riskIndicator >= MarginAccount.RISK_WARNING_LEVEL &&
      riskIndicator <= MarginAccount.RISK_CRITICAL_LEVEL
    ) {
      setHealthGauge({
        percentage: riskIndicator * 100,
        standing: 'moderate'
      });
    } else {
      setHealthGauge({
        percentage: riskIndicator * 100,
        standing: 'good'
      });
    }
  }, [marginAccount]);

  return (
    <div className="healthbar flex-centered column">
      <div className="healthbar-bar">
        <div className="healthbar-bar-indicator flex-centered column" style={{ left: `${healthGauge.percentage}%` }}>
          <div className={`healthbar-bar-indicator-arrow healthbar-bar-indicator-arrow-${healthGauge.standing}`}></div>
        </div>
      </div>
      {props.fullDetail && (
        <div className="healthbar-full-detail flex justify-evenly align-start">
          {['critical', 'low', 'moderate', 'good'].map(standing => (
            <div
              key={standing}
              className={`healthbar-full-detail-status flex-centered column
              ${healthGauge.standing === standing ? 'active' : ''}`}>
              <p>{dictionary.healthbar[standing]?.toUpperCase()}</p>
              <span className="center-text">{dictionary.healthbar[`${standing}Detail`]}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
