import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { AccountsInit, AccountsLoading } from '../../state/user/accounts';
import { Button } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';
import { NewAccountModal } from '../../state/modals/modals';
import { PoolsInit } from '../../state/borrow/pools';
import { ReactComponent as AccountLockedIcon } from '../../styles/icons/account-locked.svg';

// A feedback message overlaying components that can only used from an account
export function AccountMessage(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const poolsInit = useRecoilValue(PoolsInit);
  const accountsLoading = useRecoilValue(AccountsLoading);
  const accountsInit = useRecoilValue(AccountsInit);
  const setNewAccountModalOpen = useSetRecoilState(NewAccountModal);

  // Conditional render based on status of accounts
  function renderAccountsMessage() {
    let render = (
      <div className="flex-centered column">
        <AccountLockedIcon className="jet-icon" />
        <Button className="function-btn" size="small" onClick={() => setNewAccountModalOpen(true)}>
          {dictionary.accountsView.createAnAccount}
        </Button>
      </div>
    );

    // If pools are still loading or accounts are loading and not yet init (initial load), show loading spinner
    const initialAccountsLoad = accountsLoading && !accountsInit;
    if (initialAccountsLoad || !poolsInit) {
      render = <LoadingOutlined />;
    }

    return render;
  }

  return <div className="overlay-message view-element-item">{renderAccountsMessage()}</div>;
}
