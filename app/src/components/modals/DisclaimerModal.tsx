import { useState } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import reactStringReplace from 'react-string-replace';
import { Dictionary } from '../../state/settings/localization/localization';
import { Cluster, DisclaimersAccepted } from '../../state/settings/settings';
import { Button, Checkbox, Modal, Typography } from 'antd';

// Disclaimer modal if the current wallet hasn't accepted terms/conditions yet
export function DisclaimerModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
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
            <ul>
              <li>{dictionary.modals.alerts.disclaimer.lendAndBorrow}</li>
              <li>{dictionary.modals.alerts.disclaimer.tradeWithSize}</li>
              <li>{dictionary.modals.alerts.disclaimer.swapFavoriteCoins}</li>
              <li>{dictionary.modals.alerts.disclaimer.manageAllAssets}</li>
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
