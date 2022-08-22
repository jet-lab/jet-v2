import { useRecoilValue } from 'recoil';
import { Geobanned, Dictionary } from '../../state/settings/localization/localization';
import { Button, Modal, Typography } from 'antd';

export function GeobannedModal(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const geobanned = useRecoilValue(Geobanned);
  const { Paragraph } = Typography;

  if (geobanned) {
    return (
      <Modal visible className="geobanned-modal" footer={null} closable={false}>
        <img src="img/ui/region.gif" width="100%" alt="Geobanned gif" />
        <Paragraph type="secondary">
          <b>{dictionary.modals.uhOh}&nbsp;</b>
          {dictionary.modals.alerts.geobanned.regionNotAvailable}
        </Paragraph>
        <Button block onClick={() => (window.location.href = 'https://jetprotocol.io/')}>
          {dictionary.modals.alerts.geobanned.debark}
        </Button>
      </Modal>
    );
  } else {
    return <></>;
  }
}
