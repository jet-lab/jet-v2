import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useRecoilState, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Airspace } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { ActionRefresh, SendingTransaction } from '@state/actions/actions';
import { AccountUpgradeModal as AccountUpgradeModalState } from '@state/modals/modals';
import { CurrentAccount, CurrentAccountAddress } from '@state/user/accounts';
import { WalletTokens } from '@state/user/walletTokens';
import { notify } from '@utils/notify';
import { useProvider } from '@utils/jet/provider';
import { getExplorerUrl } from '@utils/ui';
import { List, Modal, Tooltip, Typography } from 'antd';
import { useJetStore } from '@jet-lab/store';
import { checkUpgradeLookupRegistry } from '@utils/lookupTables';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';

/*
We want to create a user's registry and populate it.
We need the config to do so.
We need their existing registry account so we can determine if it needs upgrading.
    We should check all the addresses against existing ones, or we could do this 
    on the server, which would be better and cacheable.
    Even if we don't send actual instructions and allow the UI to construct it,
    we can still send a list of addresses to add to lookup tables.
 */

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
  const walletTokens = useRecoilValue(WalletTokens);
  const [disabled, _setDisabled] = useState(false);
  const [inputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  const { Title, Paragraph, Text } = Typography;
  const [upgradeInstructions, setUpgradeInstructions] = useState<any[]>([]);
  const actions = [...new Set(upgradeInstructions.map(ix => ix.summary))];

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
        setUpgradeInstructions(response.instructions);
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

  // Renders the wallet balance for SOL
  function renderSolBalance() {
    let render = <></>;
    if (walletTokens) {
      const balance = walletTokens.map.SOL.amount.tokens;
      render = <Paragraph type="secondary" italic>{`${balance} SOL`}</Paragraph>;
    }

    return render;
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
        <div className="flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {renderSolBalance()}
        </div>
        <Text type="danger">{inputError ?? ''}</Text>
        <div className="rent-fee-info flex-centered column">
          {renderDisabledMessage()}
          <Paragraph type="secondary">
            {' '}
            <Tooltip
              overlayStyle={{ minWidth: '400px' }}
              title="We may require upgrading your margin account from time to time. Examples include closing unused positions, upgrading on-chain structures, or other maintenance changes. Depending on the nature of changes, you may incur a minimal amount of SOL for rent, or recover some rent for closed accounts.">
              <span className="underlined">Your margin account requires upgrading before you continue.</span>
            </Tooltip>{' '}
          </Paragraph>
          <Paragraph type="secondary">
            The following actions will be performed on your account.
          </Paragraph>
          <List
            dataSource={actions}
            renderItem={(item) => (
              <List.Item>
                {item}
              </List.Item>
            )}
          ></List>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
