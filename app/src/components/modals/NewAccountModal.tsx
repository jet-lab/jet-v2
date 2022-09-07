import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useSetRecoilState, useRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { feesBuffer } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { Cluster, BlockExplorer } from '../../state/settings/settings';
import { NewAccountModal as NewAccountModalState } from '../../state/modals/modals';
import {
  AccountNames,
  Accounts,
  CurrentAccountName,
  FavoriteAccounts,
  NewAccountRentFee
} from '../../state/user/accounts';
import { WalletTokens } from '../../state/user/walletTokens';
import { notify } from '../../utils/notify';
import { useMarginActions } from '../../utils/jet/marginActions';
import { ActionResponse } from '../../utils/jet/marginActions';
import { getExplorerUrl } from '../../utils/ui';
import { Input, Modal, Typography } from 'antd';

export function NewAccountModal(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const { publicKey } = useWallet();
  const newAccountModalOpen = useRecoilValue(NewAccountModalState);
  const resetNewAccountModal = useResetRecoilState(NewAccountModalState);
  const setCurrentAccountName = useSetRecoilState(CurrentAccountName);
  const { createAccount } = useMarginActions();
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const newAccountRentFee = useRecoilValue(NewAccountRentFee);
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const accountNames = useRecoilValue(AccountNames);
  const [newAccountName, setNewAccountName] = useState<string | undefined>(undefined);
  const [disabled, setDisabled] = useState(true);
  const [inputError, setInputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useState(false);
  const { Title, Paragraph, Text } = Typography;

  // Create a new account with a deposit
  async function newAccount() {
    if (!publicKey || disabled) {
      return;
    }

    // Check newAccountName or set to default
    let accountName = newAccountName;
    if (accountName) {
      const nameMatch = Object.values(accountNames).filter(
        name => name.toLowerCase() === accountName?.toLowerCase()
      )[0];
      if (nameMatch) {
        setInputError(dictionary.modals.editAccount.nameAlreadyTaken.replace('{{USED_NAME}}', accountName));
        return;
      }
    } else {
      const latestSeed = accounts.length ? accounts[accounts.length - 1].seed + 1 : 0;
      accountName = `${dictionary.common.account} ${latestSeed + 1}`;
    }

    setSendingTransaction(true);
    const [txId, resp] = await createAccount(accountName);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.newAccount.successTitle,
        dictionary.notifications.newAccount.successDescription.replaceAll('{{ACCOUNT_NAME}}', accountName ?? ''),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );

      // Update favorite accounts and set UI to new account
      const favoriteAccountsClone = { ...favoriteAccounts };
      const favoriteWalletAccounts = favoriteAccountsClone[publicKey.toString()] ?? [];
      const newWalletFavorites: string[] = [...favoriteWalletAccounts];
      newWalletFavorites.push(accountName);
      favoriteAccountsClone[publicKey.toString()] = newWalletFavorites;
      setFavoriteAccounts(favoriteAccountsClone);
      setCurrentAccountName(accountName);

      // Reset modal
      setNewAccountName(undefined);
      resetNewAccountModal();
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.newAccount.cancelledTitle,
        dictionary.notifications.newAccount.cancelledDescription,
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.newAccount.failedTitle,
        dictionary.notifications.newAccount.failedDescription,
        'error'
      );
    }
    setSendingTransaction(false);
  }

  // Check if user has enough SOL to cover rent + fees
  useEffect(() => {
    if (walletTokens && walletTokens.map.SOL.amount.lamports.toNumber() >= feesBuffer) {
      setDisabled(false);
    } else {
      setDisabled(true);
    }
  }, [walletTokens]);

  if (newAccountModalOpen) {
    return (
      <Modal
        visible
        className="header-modal new-account-modal"
        onCancel={() => {
          resetNewAccountModal();
          setNewAccountName(undefined);
        }}
        onOk={newAccount}
        okText={dictionary.accountsView.createAccount}
        okButtonProps={{
          loading: sendingTransaction,
          disabled: disabled || sendingTransaction
        }}>
        <div className="modal-header flex-centered">
          <Title className="modal-header-title green-text">{dictionary.actions.newAccount.title}</Title>
        </div>
        <div className="flex align-center justify-between">
          <Text className="small-accent-text">{dictionary.common.walletBalance.toUpperCase()}</Text>
          {walletTokens && <Paragraph type="secondary" italic>{`${walletTokens.map.SOL.amount.tokens} SOL`}</Paragraph>}
        </div>
        <Input
          type="text"
          className={inputError ? 'error' : ''}
          placeholder={`${dictionary.actions.newAccount.accountNamePlaceholder}..`}
          value={newAccountName}
          disabled={disabled || sendingTransaction}
          onChange={e => {
            setNewAccountName(e.target.value);
          }}
          onPressEnter={newAccount}
        />
        <Text type="danger">{inputError ?? ''}</Text>
        <div className="rent-fee-info flex-centered column">
          {disabled && (
            <Text type="danger" italic style={{ marginBottom: 10 }}>
              {dictionary.actions.newAccount.disabledMessages.noSolForRentFee}
            </Text>
          )}
          <Paragraph type="secondary">
            {dictionary.actions.newAccount.rentFeeInfo.replaceAll('{{RENT_FEE}}', newAccountRentFee.toString())}
          </Paragraph>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
