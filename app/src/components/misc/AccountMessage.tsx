import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '../../state/settings/localization/localization';
import { AccountsInit, AccountsLoading } from '../../state/user/accounts';
import { Button } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';
import { NewAccountModal } from '../../state/modals/modals';
import { PoolsInit } from '../../state/borrow/pools';
import { ReactComponent as AccountLockedIcon } from '../../styles/icons/account-locked.svg';

export function AccountMessage(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const poolsInit = useRecoilValue(PoolsInit);
  const accountsLoading = useRecoilValue(AccountsLoading);
  const accountsInit = useRecoilValue(AccountsInit);
  const setNewAccountModalOpen = useSetRecoilState(NewAccountModal);

  return (
    <div className="overlay-message view-element-item">
      {(accountsLoading && !accountsInit) || !poolsInit ? (
        <LoadingOutlined />
      ) : (
        <div className="flex-centered column">
          <AccountLockedIcon className="jet-icon" />
          <Button className="function-btn" size="small" onClick={() => setNewAccountModalOpen(true)}>
            {dictionary.accountsView.createAnAccount}
          </Button>
        </div>
      )}
    </div>
  );
}
