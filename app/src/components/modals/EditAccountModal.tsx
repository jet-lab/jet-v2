import { useState } from 'react';
import { useRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import axios from 'axios';
import { useWallet } from '@solana/wallet-adapter-react';
import { SendingTransaction } from '@state/actions/actions';
import { EditAccountModal as EditAccountModalOpen } from '@state/modals/modals';
import { Cluster } from '@state/settings/settings';
import { Dictionary } from '@state/settings/localization/localization';
import { CurrentAccountAddress, CurrentAccount, AccountNames } from '@state/user/accounts';
import { notify } from '@utils/notify';
import { Input, Modal, Typography } from 'antd';

// Modal for editing account information
export function EditAccountModal(): JSX.Element {
  const editAccountModalOpen = useRecoilValue(EditAccountModalOpen);
  const resetEditAccountModal = useResetRecoilState(EditAccountModalOpen);
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const currentAccount = useRecoilValue(CurrentAccount);
  const [currentAccountAddress, setCurrentAccountAddress] = useRecoilState(CurrentAccountAddress);
  const accountNames = useRecoilValue(AccountNames);
  const [newAccountName, setNewAccountName] = useState<string | undefined>(undefined);
  const [inputError, setInputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const { Title, Text } = Typography;

  // Change account name
  async function changeAccountName() {
    if (!currentAccount || !newAccountName || !publicKey) {
      return;
    }

    // Check if name is already in use, error if so
    const nameMatch = Object.values(accountNames).filter(
      name => name.toLowerCase() === newAccountName.toLowerCase()
    )[0];
    if (nameMatch) {
      setInputError(dictionary.modals.editAccount.nameAlreadyTaken.replace('{{USED_NAME}}', newAccountName));
      return;
    }

    // Begin loading and track the old account name for notification
    setSendingTransaction(true);
    const oldAccountName = accountNames[currentAccountAddress] ?? '';
    // Update in database (TODO: put editing account names back in)
    axios
      .put(``, {
        alias: newAccountName,
        network: cluster,
        publicKey: currentAccount.address.toString()
      })
      .then(() => {
        notify(
          dictionary.notifications.editAccount.successTitle,
          dictionary.notifications.editAccount.successDescription
            .replace('{{OLD_ACCOUNT_NAME}}', oldAccountName)
            .replace('{{NEW_ACCOUNT_NAME}}', newAccountName),
          'success'
        );

        setCurrentAccountAddress(newAccountName);
        setNewAccountName(undefined);
        resetEditAccountModal();
      })
      .catch(err => {
        console.error(err);
        notify(
          dictionary.notifications.editAccount.failedTitle,
          dictionary.notifications.editAccount.failedDescription,
          'error'
        );
      });
    setSendingTransaction(false);
  }

  if (editAccountModalOpen) {
    return (
      <Modal
        className="header-modal edit-account-modal"
        maskClosable={false}
        open={editAccountModalOpen}
        onCancel={resetEditAccountModal}
        onOk={changeAccountName}
        okText={dictionary.settingsModal.save}
        okButtonProps={{
          loading: sendingTransaction,
          disabled: sendingTransaction || !newAccountName
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">
            {dictionary.modals.editAccount.title.replace('{{ACCOUNT_NAME}}', accountNames[currentAccountAddress] ?? '')}
          </Title>
        </div>
        <Text className="small-accent-text">{dictionary.actions.newAccount.accountName.toUpperCase()}</Text>
        <Input
          type="text"
          className={inputError ? 'error' : ''}
          placeholder={accountNames[currentAccountAddress]}
          value={newAccountName}
          disabled={sendingTransaction}
          onChange={e => {
            setNewAccountName(e.target.value);
          }}
          onPressEnter={changeAccountName}
        />
        <Text type="danger">{inputError ?? ''}</Text>
      </Modal>
    );
  } else {
    return <></>;
  }
}
