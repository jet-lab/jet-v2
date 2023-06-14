import { useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import reactStringReplace from 'react-string-replace';
import { Dictionary } from '@state/settings/localization/localization';
import { DisclaimersAccepted } from '@state/settings/settings';
import { Button, Checkbox, Modal, Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';

// Disclaimer modal if the current wallet hasn't accepted terms/conditions yet
export function DisclaimerModal(): JSX.Element {
  const cluster = useJetStore(state => state.settings.cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const [disclaimersAccepted, setDisclaimersAccepted] = useRecoilState(DisclaimersAccepted);
  const [dislaimerChecked, setDisclaimerChecked] = useState(false);
  const disclaimerNotAccepted = publicKey && cluster === 'mainnet-beta' && !disclaimersAccepted[publicKey.toBase58()];
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

  // If wallet hasn't accepted disclaimer, show modal
  if (disclaimerNotAccepted) {
    return (
      <Modal open className="disclaimer-modal" maskClosable={false} footer={null} closable={false}>
        <div className="modal-content flex-centered column">
          <img src="img/jet/jet_logo.png" width="100px" height="auto" alt="Jet Protocol" />
          <div className="disclaimer-modal-bullets flex align-start justify-center column">
            <Text>{disclaimer}</Text>
            <Text>
              WARNING if you have liabilities on your margin account, the health can change as prices change. There is NO
              warning system, so the only place you can see these fluctuations in your account health is in the app itself.{' '}
            </Text>
          </div>
          <br/>
          <Checkbox onChange={e => setDisclaimerChecked(e.target.checked)}>
            {dictionary.modals.alerts.disclaimer.acceptRisks}
          </Checkbox>
          <br/>
          <Button
            block
            size="small"
            disabled={!dislaimerChecked}
            onClick={() => setDisclaimersAccepted({ ...disclaimersAccepted, [publicKey.toBase58()]: true })}>
            {dictionary.modals.alerts.disclaimer.agreeToEnter}
          </Button>
          <br/>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
