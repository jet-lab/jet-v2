import { useEffect, useState } from 'react';
import { useResetRecoilState, useRecoilValue, useRecoilState, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { feesBuffer, MarginPrograms } from '@jet-lab/margin';
import { Dictionary } from '@state/settings/localization/localization';
import { ActionRefresh, SendingTransaction } from '@state/actions/actions';
import { NewAccountModal as NewAccountModalState } from '@state/modals/modals';
import { AccountNames, Accounts } from '@state/user/accounts';
import { WalletTokens } from '@state/user/walletTokens';
import { notify } from '@utils/notify';
import { useProvider } from '@utils/jet/provider';
import { useMarginActions } from '@utils/jet/marginActions';
import { ActionResponse } from '@utils/jet/marginActions';
import { getExplorerUrl } from '@utils/ui';
import { Input, Modal, Tooltip, Typography } from 'antd';
import { NetworkStateAtom } from '../../state/network/network-state';
import debounce from 'lodash.debounce';
import { useJetStore, ws } from '@jet-lab/store';

// Modal for user to create a new margin account
export function NewAccountModal(): JSX.Element {
  const { cluster, explorer } = useJetStore(state => state.settings);
  const dictionary = useRecoilValue(Dictionary);
  const { programs } = useProvider();
  const { publicKey } = useWallet();
  const newAccountModalOpen = useRecoilValue(NewAccountModalState);
  const resetNewAccountModal = useResetRecoilState(NewAccountModalState);
  const { createAccount } = useMarginActions();
  const [newAccountRentFee, setNewAccountRentFee] = useState<number | undefined>();
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const accountNames = useRecoilValue(AccountNames);
  const latestSeed = accounts.length ? accounts[accounts.length - 1].seed + 1 : 0;
  const [newAccountName, setNewAccountName] = useState<string | undefined>(undefined);
  const [disabled, setDisabled] = useState(true);
  const [inputError, setInputError] = useState<string | undefined>();
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);
  const networkState = useRecoilValue(NetworkStateAtom);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  const { Title, Paragraph, Text } = Typography;

  useEffect(() => {
    if (newAccountModalOpen) setActionRefresh(Date.now());
  }, [newAccountModalOpen]);

  // Create a new account with a deposit
  async function newAccount() {
    if (!publicKey || disabled) {
      return;
    }

    // Check newAccountName or set to default
    let accountName = newAccountName;
    if (accountName) {
      // If name is already in use, show the error
      const nameMatch = Object.values(accountNames).filter(
        name => name.toLowerCase() === accountName?.toLowerCase()
      )[0];
      if (nameMatch) {
        setInputError(dictionary.modals.editAccount.nameAlreadyTaken.replace('{{USED_NAME}}', accountName));
        return;
      }
    } else {
      // Default to "Account 1" style name
      accountName = `${dictionary.common.account} ${latestSeed + 1}`;
    }

    // Subscribe to websocket if this is the first account.
    // It can happen if a user connects quickly tbat tbeir subscription
    // is not registered. Subscribing on a new account ensures that it is registered.
    if (latestSeed === 0) {
      const subscriptionEvent: APPLICATION_WS_EVENTS = {
        type: 'SUBSCRIBE',
        payload: {
          wallet: publicKey.toBase58(),
          margin_accounts: []
        }
      };
      ws?.send(JSON.stringify(subscriptionEvent));
    }

    setSendingTransaction(true);
    // Create the new account
    const [txId, resp] = await createAccount();
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.newAccount.successTitle,
        dictionary.notifications.newAccount.successDescription.replaceAll('{{ACCOUNT_NAME}}', accountName ?? ''),
        'success',
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
      );

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
        'error',
        txId ? getExplorerUrl(txId, cluster, explorer) : undefined
      );
    }
    setSendingTransaction(false);
  }

  // Set rent fee for creating a new account
  useEffect(() => {
    if (!programs || networkState !== 'connected') {
      return;
    }
    async function getNewAccountRentFee(programs: MarginPrograms) {
      const rentFeeLamports = await programs.connection.getMinimumBalanceForRentExemption(
        programs.margin.account.marginAccount.size
      );
      const rentFee = rentFeeLamports / LAMPORTS_PER_SOL;
      setNewAccountRentFee(rentFee);
    }
    getNewAccountRentFee(programs);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [programs, networkState]);

  // Check if user has enough SOL to cover rent + fees.
  // If user's SOL is less than feeBuffer amount, disable user from creating new account.
  useEffect(() => {
    try {
      if (walletTokens && walletTokens.map.SOL.amount.lamports.gte(feesBuffer)) {
        setDisabled(false);
      } else {
        setDisabled(true);
      }
    } catch (err) {
      setDisabled(false);
      console.warn(err);
    }
  }, [walletTokens]);

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

  // TODO: enable custom names again
  if (newAccountModalOpen) {
    return (
      <Modal
        open
        className="header-modal new-account-modal"
        maskClosable={false}
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
          {renderSolBalance()}
        </div>
        <Tooltip title={dictionary.actions.newAccount.customNamesComingSoon}>
          <div>
            <Input
              type="text"
              className={inputError ? 'error' : ''}
              placeholder={dictionary.actions.newAccount.accountNamePlaceholder + '..'}
              value={/* newAccountName */ `${dictionary.common.account} ${latestSeed + 1}`}
              disabled={/* disabled || sendingTransaction */ true}
              onChange={debounce(e => setNewAccountName(e.target.value), 300)}
              onPressEnter={newAccount}
              style={{ boxShadow: 'unset' }}
            />
          </div>
        </Tooltip>
        <Text type="danger">{inputError ?? ''}</Text>
        <div className="rent-fee-info flex-centered column">
          {renderDisabledMessage()}
          <Paragraph type="secondary">
            The Solana Blockchain requires a one-time{' '}
            <Tooltip
              overlayStyle={{ minWidth: '400px' }}
              title="When you open your account on Jet, rent is charged by the Solana blockchain for the storage of that data. The rent fee can be fully refunded in the future when you close all your account obligations on Jet. This is true for all protocols which allocate space per user on-chain.">
              <span className="underlined">rent fee</span>
            </Tooltip>{' '}
            of {newAccountRentFee ? newAccountRentFee.toString() : dictionary.common.notAvailable} SOL for each new
            margin account.
          </Paragraph>
        </div>
      </Modal>
    );
  } else {
    return <></>;
  }
}
