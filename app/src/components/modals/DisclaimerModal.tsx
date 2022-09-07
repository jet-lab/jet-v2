import { useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import reactStringReplace from 'react-string-replace';
import { Dictionary } from '../../state/settings/localization/localization';
import { Cluster, DisclaimersAccepted } from '../../state/settings/settings';
import { Button, Checkbox, Modal, Typography } from 'antd';

export function DisclaimerModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const [disclaimersAccepted, setDisclaimersAccepted] = useRecoilState(DisclaimersAccepted);
  const [dislaimerChecked, setDisclaimerChecked] = useState(false);
  const { Text } = Typography;

  // Set up dislaimer with inline links
  let disclaimer = reactStringReplace(dictionary.modals.alerts.disclaimer.disclaimerDetail, '{{TERMS_LINK}}', () => (
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

  if (publicKey && cluster === 'mainnet-beta' && !disclaimersAccepted[publicKey.toBase58()]) {
    return (
      <Modal visible className="disclaimer-modal" footer={null} closable={false}>
        <div className="modal-content flex-centered column">
          <img src="img/jet/jet_logo.png" width="100px" height="auto" alt="Jet Protocol" />
          <div className="disclaimer-modal-bullets flex align-start justify-center column">
            <ul>
              <li>TODO: Add something</li>
              <li>TODO: Add something</li>
              <li>TODO: Add something</li>
              <li>TODO: Add something</li>
              <li>TODO: Add something</li>
            </ul>
          </div>
          <Text>{disclaimer}</Text>
          <Checkbox onChange={e => setDisclaimerChecked(e.target.checked)}>
            {dictionary.modals.alerts.disclaimer.acceptRisks}
          </Checkbox>
          <Button
            block
            size="small"
            disabled={!dislaimerChecked}
            onClick={() => setDisclaimersAccepted({ ...disclaimersAccepted, [publicKey.toBase58()]: true })}>
            {dictionary.modals.alerts.disclaimer.enterMainnet}
          </Button>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
