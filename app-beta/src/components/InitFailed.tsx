import { useGeoban, useLanguage } from '../contexts/localization/localization';

export function InitFailed(): JSX.Element {
  const { dictionary } = useLanguage();
  const isGeobanned = useGeoban();

  return (
    <div className="init-failed view flex-centered column">
      <img src="img/ui/failed_init.gif" width="600px" alt="Failed To Init App" />
      <h1 className="danger-text">{dictionary.copilot.alert.failed}</h1>
      <p className="center-text">{isGeobanned ? dictionary.cockpit.geobanned : dictionary.cockpit.noMarket}</p>
    </div>
  );
}
