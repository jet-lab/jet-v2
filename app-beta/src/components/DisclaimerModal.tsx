import { useState } from 'react';
import reactStringReplace from 'react-string-replace';
import { useLanguage } from '../contexts/localization/localization';
import { Button, Checkbox, Modal, Typography } from 'antd';
import { useClusterSetting } from '../contexts/clusterSetting';

export function DisclaimerModal(): JSX.Element {
  const { clusterSetting } = useClusterSetting();
  const { dictionary } = useLanguage();
  const [disclaimerAccepted, setDisclaimerAccepted] = useState(
    localStorage.getItem('jetV1DisclaimerAccepted') === 'true'
  );
  const [dislaimerChecked, setDisclaimerChecked] = useState(false);
  const { Text } = Typography;

  // Set up dislaimer with inline links
  let disclaimer = reactStringReplace(dictionary.copilot.alert.disclaimer.disclaimerDetail, '{{TERMS_LINK}}', () => (
    <a
      className="link-btn"
      key="tos"
      href="https://www.jetprotocol.io/legal/terms-of-service"
      target="_blank"
      rel="noopener noreferrer">
      {dictionary.termsPrivacy.termsOfService}
    </a>
  ));
  disclaimer = reactStringReplace(disclaimer, '{{PRIVACY_LINK}}', () => (
    <a
      className="link-btn"
      key="privacy"
      href="https://www.jetprotocol.io/legal/privacy-policy"
      target="_blank"
      rel="noopener noreferrer">
      {dictionary.termsPrivacy.privacyPolicy}
    </a>
  ));

  return (
    <Modal
      className="disclaimer-modal"
      footer={null}
      closable={false}
      visible={clusterSetting === 'mainnet-beta' && !disclaimerAccepted}>
      <div className="modal-content flex-centered column">
        <img src="img/jet/jet_logo.png" width="100px" height="auto" alt="Jet Protocol" />
        <br></br>
        <Text>{disclaimer}</Text>
        <br></br>
        <Checkbox onChange={e => setDisclaimerChecked(e.target.checked)}>
          {dictionary.copilot.alert.disclaimer.acceptRisks}
        </Checkbox>
        <br></br>
        <Button
          block
          size="small"
          disabled={!dislaimerChecked}
          onClick={() => {
            localStorage.setItem('jetV1DisclaimerAccepted', 'true');
            setDisclaimerAccepted(true);
          }}>
          {dictionary.copilot.alert.disclaimer.enterMainnet}
        </Button>
      </div>
    </Modal>
  );
}
