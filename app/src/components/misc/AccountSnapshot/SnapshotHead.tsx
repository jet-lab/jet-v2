import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Dictionary } from '../../../state/settings/localization/localization';
import { NewAccountModal } from '../../../state/modals/modals';
import { WalletTokens } from '../../../state/user/walletTokens';
import { AccountNames, CurrentAccountAddress, FavoriteAccounts } from '../../../state/user/accounts';
import { Typography, Button, Dropdown, Menu, Tabs } from 'antd';
import { StarFilled, StarOutlined } from '@ant-design/icons';
import AngleDown from '../../../assets/icons/arrow-angle-down.svg';

// Head of the Account Snapshot, where user can select/edit/create their margin accounts
export function SnapshotHead(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const walletTokens = useRecoilValue(WalletTokens);
  // const accounts = useRecoilValue(Accounts);
  const accountNames = useRecoilValue(AccountNames);
  const [currentAccountAddress, setCurrentAccountAddress] = useRecoilState(CurrentAccountAddress);
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const walletFavoriteAccounts = publicKey ? favoriteAccounts[publicKey.toString()] ?? [] : [];
  const [newAccountModalOpen, setNewAccountModalOpen] = useRecoilState(NewAccountModal);
  // const setEditAccountModalOpen = useSetRecoilState(EditAccountModal);
  const { Text } = Typography;
  const { TabPane } = Tabs;

  // Update user's favorite accounts
  function updateFavoriteAccounts(accountPubkey: string, remove?: boolean) {
    if (!publicKey) {
      return;
    }

    for (const accountKey of Object.keys(accountNames)) {
      if (accountPubkey === accountKey) {
        const favoriteAccountsClone = { ...favoriteAccounts };
        const favoriteWalletAccounts = favoriteAccountsClone[publicKey.toString()] ?? [];
        const newFavorites: string[] = [...favoriteWalletAccounts];
        // If we're removing the account from favorites
        if (remove) {
          const accountIndex = newFavorites.indexOf(accountKey);
          if (accountIndex > -1) {
            newFavorites.splice(accountIndex, 1);
          }
          // Otherwise, if we're adding the account to favorites
        } else if (!newFavorites.includes(accountKey)) {
          newFavorites.push(accountKey);
          setCurrentAccountAddress(accountKey);
        }
        favoriteAccountsClone[publicKey.toString()] = newFavorites;
        setFavoriteAccounts(favoriteAccountsClone);
      }
    }
  }

  // Renders favorite account tabs
  function renderFavoriteAccountTabs() {
    let render = <></>;

    // If wallet is init and we have favorite accounts to choose from
    if (walletTokens && walletFavoriteAccounts.length) {
      render = (
        <Tabs
          activeKey={currentAccountAddress}
          onChange={(key: string) => setCurrentAccountAddress(key)}
          className={
            !currentAccountAddress || !walletFavoriteAccounts.includes(currentAccountAddress) ? 'no-active-tabs' : ''
          }>
          {walletFavoriteAccounts.map(key => {
            const name = accountNames[key] ?? '';
            return <TabPane key={key} tab={name.toUpperCase()} active={key === currentAccountAddress}></TabPane>;
          })}
        </Tabs>
      );
    }

    return render;
  }

  return (
    <div className="account-snapshot-head flex align-center justify-between">
      <div className="account-snapshot-head-tabs flex align-center justify-start">
        <StarFilled />
        {renderFavoriteAccountTabs()}
      </div>
      <div className="account-snapshot-head-accounts flex-centered">
        <Button
          className={`function-btn ${newAccountModalOpen ? 'active' : ''}`}
          disabled={!walletTokens}
          onClick={() => setNewAccountModalOpen(true)}>
          {dictionary.actions.newAccount.title} +
        </Button>
        <Dropdown
          disabled={!walletTokens || !Object.values(accountNames).length}
          trigger={['click']}
          overlay={
            <Menu className="all-accounts-menu">
              {Object.keys(accountNames).map(key => {
                const name = accountNames[key];
                return (
                  <Menu.Item
                    key={key}
                    onClick={() => setCurrentAccountAddress(key)}
                    className={key === currentAccountAddress ? 'active' : ''}>
                    <div className="all-accounts-menu-name flex align-center justify-start">
                      {walletFavoriteAccounts.includes(key) ? (
                        <StarFilled style={{ opacity: 1 }} onClick={() => updateFavoriteAccounts(key, true)} />
                      ) : (
                        <StarOutlined onClick={() => updateFavoriteAccounts(key)} />
                      )}
                      {name}
                    </div>
                  </Menu.Item>
                );
              })}
            </Menu>
          }>
          <Text type="secondary">
            {dictionary.accountSnapshot.allAccounts.toUpperCase()}
            <AngleDown className="jet-icon" />
          </Text>
        </Dropdown>
      </div>
    </div>
  );
}
