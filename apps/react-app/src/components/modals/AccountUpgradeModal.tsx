import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useRecoilState, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Airspace } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { ActionRefresh, SendingTransaction } from '@state/actions/actions';
import { AccountUpgradeModal as AccountUpgradeModalState } from '@state/modals/modals';
import { CurrentAccount, CurrentAccountAddress } from '@state/user/accounts';
import { notify } from '@utils/notify';
import { useProvider } from '@utils/jet/provider';
import { getExplorerUrl } from '@utils/ui';
import { Modal, Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';
import { checkUpgradeLookupRegistry } from '@utils/lookupTables';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';

// Modal for user to create a new margin account
export function AccountUpgradeModal(): JSX.Element {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const { programs } = useProvider();
  const { publicKey } = useWallet();
  const [accountUpgradeModalOpen, setAccountUpgradeModalOpen] = useRecoilState(AccountUpgradeModalState);
  const resetAccountUpgradeModal = useResetRecoilState(AccountUpgradeModalState);
  const currentAccountAddress = useRecoilValue(CurrentAccountAddress);
  const currentAccount = useRecoilValue(CurrentAccount);
  const [disabled, _setDisabled] = useState(false);
  const [inputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  const { Title, Paragraph, Text } = Typography;

  const airspace = !programs ? "" : Airspace.deriveAddress(programs.airspace.programId, programs.config.airspaces[0].name).toString();

  useEffect(() => {
    if (accountUpgradeModalOpen) setActionRefresh(Date.now());
  }, [accountUpgradeModalOpen]);

  useEffect(() => {
    if (currentAccountAddress === '' || !publicKey || !programs || airspace === "") { return; }
    // Check if user should upgrade their account
    checkUpgradeLookupRegistry(airspace, currentAccountAddress, publicKey.toString()).then(response => {
      // If there are instructions, we should request the user to upgrade their account
      if (response.instructions.length) {
        setAccountUpgradeModalOpen(true);
      }
    })
  }, [currentAccountAddress, programs, publicKey])

  // Create a new account with a deposit
  async function upgradeAccount() {
    if (!programs || !currentAccount || !publicKey || currentAccountAddress === '') {
      return;
    }
    setSendingTransaction(true);

    // Get the instructions again as slots might be stale
    const upgrade = await checkUpgradeLookupRegistry(airspace, currentAccountAddress, publicKey.toString());

    const lookupInstructions = upgrade.instructions.map(ix => {
      return [new TransactionInstruction({
        programId: programs.margin.programId,
        keys: ix.instruction.accounts.map((acc: any) => {
          return {
            pubkey: new PublicKey(acc.pubkey),
            isSigner: acc.isSigner,
            isWritable: acc.isWritable
          }
        }),
        data: Buffer.from(ix.instruction.data, 'base64')
      })]
    });

    currentAccount.sendAll(lookupInstructions).then(txId => {
      setSendingTransaction(false);
      resetAccountUpgradeModal();
      notify(
        "Account Upgraded",
        "Your account has successfully been upgraded",
        'success',
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
      );
    }).catch((err: any) => {
      console.table(err);
      setSendingTransaction(false);
      if (err.toString().includes('User rejected') || err.toString().includes('Failed to sign')) {
        notify(
          "Account Upgrade Cancelled",
          "You either rejected or failed to sign the transaction. Please try again.",
          'warning'
        );
      } else {
        notify(
          "Account Upgrade Failed",
          "Your account could not be upgraded, please ensure that you have enough SOL and try again.",
          'error',
          err.signature ? getExplorerUrl(err.signature, cluster, explorer) : undefined
        );
      }
    });
  }

  // Renders disabled message for modal
  function renderDisabledMessage() {
    let render = <></>;
    if (disabled) {
      render = (
        <Text type="danger" italic style={{ marginBottom: 10 }}>
          {dictionary.actions.newAccount.disabledMessages.noSolForRentFee}
        </Text>
      );
    }

    return render;
  }

  if (accountUpgradeModalOpen) {
    return (
      <Modal
        open
        className="header-modal new-account-modal"
        maskClosable={false}
        onCancel={() => {
          resetAccountUpgradeModal();
        }}
        onOk={upgradeAccount}
        okText={dictionary.accountUpgradeModal.upgradeAccount}
        okButtonProps={{
          loading: sendingTransaction,
          disabled: disabled || sendingTransaction
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.accountUpgradeModal.title}</Title>
        </div>

        <Text type="danger">{inputError ?? ''}</Text>
        <div className="rent-fee-info flex-centered column">
          {renderDisabledMessage()}
          <Paragraph type="secondary">
            <strong>Your margin account requires upgrading before you continue.</strong>
          </Paragraph>
          <p></p>
          <Paragraph type="secondary">
            We may require upgrading your margin account from time to time.

          </Paragraph>
          <Paragraph type="secondary">

          </Paragraph>
          <Paragraph type="secondary">
            Examples include closing unused positions, upgrading on-chain structures, or other maintenance changes.
          </Paragraph>
          <p></p>
          <Paragraph type="secondary">
            Depending on the nature of changes, you may incur a minimal amount of SOL for rent, or recover some rent for closed accounts.
          </Paragraph>

        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
