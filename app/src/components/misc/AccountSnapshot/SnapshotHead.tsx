import { useRecoilState, useSetRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletInit } from '../../../state/user/walletTokens';
import { EditAccountModal, NewAccountModal } from '../../../state/modals/modals';
import { AccountsInit, CurrentAccountName, FavoriteAccounts, useAccountNames } from '../../../state/user/accounts';
import { Typography, Button, Dropdown, Menu, Tabs } from 'antd';
import { EditOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { ReactComponent as AngleDown } from '../../../styles/icons/arrow-angle-down.svg';

// Head of the Account Snapshot, where user can select/edit/create their margin accounts
export function SnapshotHead(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { publicKey } = useWallet();
  const walletInit = useRecoilValue(WalletInit);
  const accountsInit = useRecoilValue(AccountsInit);
  const accountNames = useAccountNames();
  const [currentAccountName, setCurrentAccountName] = useRecoilState(CurrentAccountName);
  const [favoriteAccounts, setFavoriteAccounts] = useRecoilState(FavoriteAccounts);
  const walletFavoriteAccounts = publicKey ? favoriteAccounts[publicKey.toString()] ?? [] : [];
  const [newAccountModalOpen, setNewAccountModalOpen] = useRecoilState(NewAccountModal);
  const setEditAccountModalOpen = useSetRecoilState(EditAccountModal);
  const { Text } = Typography;
  const { TabPane } = Tabs;

  // Update user's favorite accounts
  function updateFavoriteAccounts(accountName: string, remove?: boolean) {
    if (!publicKey) {
      return;
    }

    const favoriteAccountsClone = { ...favoriteAccounts };
    const favoriteWalletAccounts = favoriteAccountsClone[publicKey.toString()] ?? [];
    const newFavorites: string[] = [...favoriteWalletAccounts];
    // If we're removing the account from favorites
    if (remove) {
      const accountIndex = newFavorites.indexOf(accountName);
      if (accountIndex > -1) {
        newFavorites.splice(accountIndex, 1);
      }
      // Otherwise, if we're adding the account to favorites
    } else {
      newFavorites.push(accountName);
    }
    favoriteAccountsClone[publicKey.toString()] = newFavorites;
    setFavoriteAccounts(favoriteAccountsClone);
  }

  // Renders favorite account tabs
  function renderFavoriteAccountTabs() {
    let render = <></>;

    // If wallet is init and we have favorite accounts to choose from
    if (walletInit && walletFavoriteAccounts.length) {
      render = (
        <Tabs
          activeKey={currentAccountName}
          onChange={(name: string) => setCurrentAccountName(name)}
          className={
            !currentAccountName || !walletFavoriteAccounts.includes(currentAccountName) ? 'no-active-tabs' : ''
          }>
          {walletFavoriteAccounts.map(name => (
            <TabPane
              key={name}
              tab={name.includes('...') ? name : name.toUpperCase()}
              active={currentAccountName?.toLocaleLowerCase() === name.toLocaleUpperCase()}></TabPane>
          ))}
        </Tabs>
      );
    }

    return render;
  }

  return (
    <div className="account-snapshot-head view-element-item view-element-item-hidden flex align-center justify-between">
      <div className="account-snapshot-head-tabs flex align-center justify-start">
        <StarFilled />
        {renderFavoriteAccountTabs()}
      </div>
      <div className="account-snapshot-head-accounts flex-centered">
        <Button
          className={`function-btn ${newAccountModalOpen ? 'active' : ''}`}
          disabled={!walletInit || !accountsInit}
          onClick={() => setNewAccountModalOpen(true)}>
          {dictionary.actions.newAccount.title} +
        </Button>
        <Dropdown
          disabled={!walletInit || !accountNames.length}
          trigger={['click']}
          overlay={
            <Menu className="all-accounts-menu">
              {accountNames.map(name => (
                <Menu.Item
                  key={name}
                  onClick={() => setCurrentAccountName(name)}
                  className={name === currentAccountName ? 'active' : ''}>
                  <div className="all-accounts-menu-name flex align-center justify-start">
                    {walletFavoriteAccounts.includes(name) ? (
                      <StarFilled style={{ opacity: 1 }} onClick={() => updateFavoriteAccounts(name, true)} />
                    ) : (
                      <StarOutlined onClick={() => updateFavoriteAccounts(name)} />
                    )}
                    {name}
                  </div>
                  <EditOutlined
                    onClick={() => {
                      setCurrentAccountName(name);
                      setEditAccountModalOpen(true);
                    }}
                  />
                </Menu.Item>
              ))}
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
