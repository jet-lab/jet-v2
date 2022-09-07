import { useState } from 'react';
import { useRecoilState, useResetRecoilState, useRecoilValue } from 'recoil';
import axios from 'axios';
import { useWallet } from '@solana/wallet-adapter-react';
import { EditAccountModal as EditAccountModalOpen } from '../../state/modals/modals';
import { Cluster } from '../../state/settings/settings';
import { Dictionary } from '../../state/settings/localization/localization';
import { CurrentAccountName, FavoriteAccounts, CurrentAccount, AccountNames } from '../../state/user/accounts';
import { notify } from '../../utils/notify';
import { Input, Modal, Typography } from 'antd';

export function EditAccountModal(): JSX.Element {
  const editAccountModalOpen = useRecoilValue(EditAccountModalOpen);
  const resetEditAccountModal = useResetRecoilState(EditAccountModalOpen);
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const currentAccount = useRecoilValue(CurrentAccount);
  const [currentAccountName, setCurrentAccountName] = useRecoilState(CurrentAccountName);
  const accountNames = useRecoilValue(AccountNames);
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const [newAccountName, setNewAccountName] = useState<string | undefined>(undefined);
  const [inputError, setInputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Text } = Typography;

  // Change account name
  async function changeAccountName() {
    if (!currentAccount || !newAccountName || !publicKey) {
      return;
    }
    const nameMatch = Object.values(accountNames).filter(
      name => name.toLowerCase() === newAccountName.toLowerCase()
    )[0];
    if (nameMatch) {
      setInputError(dictionary.modals.editAccount.nameAlreadyTaken.replace('{{USED_NAME}}', newAccountName));
      return;
    }

    setSendingTransaction(true);
    const oldAccountName = currentAccountName ?? '';
    axios
      .put(`https://api.jetprotocol.io/v1/margin/${publicKey.toString()}/accounts`, {
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

        const favoriteAccountsClone = { ...favoriteAccounts };
        const walletFavoriteAccounts = favoriteAccountsClone[publicKey.toString()] ?? [];
        walletFavoriteAccounts[walletFavoriteAccounts.indexOf(oldAccountName)] = newAccountName;
        favoriteAccountsClone[publicKey.toString()] = walletFavoriteAccounts;
        setFavoriteAccounts(favoriteAccountsClone);
        setCurrentAccountName(newAccountName);
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
        visible={editAccountModalOpen}
        onCancel={resetEditAccountModal}
        onOk={changeAccountName}
        okText={dictionary.settingsModal.save}
        okButtonProps={{
          loading: sendingTransaction,
          disabled: sendingTransaction || !newAccountName
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">
            {dictionary.modals.editAccount.title.replace('{{ACCOUNT_NAME}}', currentAccountName ?? '')}
          </Title>
        </div>
        <Text className="small-accent-text">{dictionary.actions.newAccount.accountName.toUpperCase()}</Text>
        <Input
          type="text"
          className={inputError ? 'error' : ''}
          placeholder={currentAccountName}
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
