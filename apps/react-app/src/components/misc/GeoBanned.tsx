import { Dictionary } from '@state/settings/localization/localization';
import { Button, Typography } from 'antd';
import { useRecoilValue } from 'recoil';

export const GeobannedComponent = ({ countryCode }: { countryCode?: string }) => {
  const dictionary = useRecoilValue(Dictionary);
  const { Paragraph } = Typography;

  return (
    <div className="geobanned-container">
      <img src="img/ui/region.gif" width="100%" alt="Geobanned gif" />
      <Paragraph className="mb-2" type="secondary">
        <b>{dictionary.modals.uhOh}&nbsp;</b>
        {dictionary.modals.alerts.geobanned.regionNotAvailable}
      </Paragraph>
      {countryCode === 'US' && <Paragraph className="mb-2">Jet is not available in the USA.</Paragraph>}
      <Button block onClick={() => (window.location.href = 'https://jetprotocol.io/')}>
        {dictionary.modals.alerts.geobanned.debark}
      </Button>
    </div>
  );
};
