import { useRecoilValue } from 'recoil';
import { Dictionary } from '../../../state/settings/localization/localization';
import { WalletInit } from '../../../state/user/walletTokens';
import { AccountsInit, CurrentAccount } from '../../../state/user/accounts';
import { useCurrencyFormatting } from '../../../utils/currency';
import { formatRiskIndicator } from '../../../utils/format';
import { useRiskLevel, useRiskStyle } from '../../../utils/risk';
import { Typography, Skeleton } from 'antd';
import { ConnectionFeedback } from '../ConnectionFeedback';
import { Info } from '../Info';
import { RiskMeter } from '../RiskMeter';

// Body of the Account Snapshot, where users can see data for the currently selected margin account
export function SnapshotBody(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const walletInit = useRecoilValue(WalletInit);
  const accountsInit = useRecoilValue(AccountsInit);
  const initialAccountsLoad = walletInit && !accountsInit;
  const currentAccount = useRecoilValue(CurrentAccount);
  const riskLevel = useRiskLevel();
  const riskStyle = useRiskStyle();
  const { Title, Text } = Typography;

  // Renders the account balance
  function renderAccountBalance() {
    // The account balance (deposits - liabilities)
    let accountBalance = 0;
    if (currentAccount && currentAccount.summary) {
      const depositedValue = currentAccount.summary.depositedValue;
      const borrowedValue = currentAccount.summary.borrowedValue;
      accountBalance = depositedValue - borrowedValue;
    }

    let render = <Title>{currencyFormatter(accountBalance, true)}</Title>;
    if (initialAccountsLoad) {
      render = <Skeleton className="align-center" paragraph={false} active />;
    }

    return render;
  }

  // Renders the account's required/effective collateral
  function renderCollateral(type: 'required' | 'effective') {
    const requiredCollateral = currentAccount?.valuation.requiredCollateral.toNumber() ?? 0;
    const effectiveCollateral = currentAccount?.valuation.effectiveCollateral.toNumber() ?? 0;
    let collateral = requiredCollateral;
    if (type === 'effective') {
      collateral = effectiveCollateral;
    }

    let render = <Title>{currencyAbbrev(collateral, true)}</Title>;
    if (initialAccountsLoad) {
      render = <Skeleton className="align-center" paragraph={false} active />;
    }

    return render;
  }

  // Renders the account's Risk Level
  function renderRiskLevel() {
    let render = <Title type={riskStyle}>{formatRiskIndicator(currentAccount?.riskIndicator ?? 0)}</Title>;
    if (initialAccountsLoad) {
      render = <Skeleton className="align-center" paragraph={false} active />;
    }

    return render;
  }

  // Returns the account's assets (if there are any)
  function getAccountAssets() {
    let accountAssets = 0;
    if (currentAccount && currentAccount.summary) {
      accountAssets = currentAccount.summary.depositedValue;
    }

    return accountAssets;
  }

  // Returns the account's liabilities (if there are any)
  function getAccountLiabilities() {
    let accountLiabilities = 0;
    if (currentAccount && currentAccount.summary) {
      accountLiabilities = currentAccount.summary.borrowedValue;
    }

    return accountLiabilities;
  }

  return (
    <div className="account-snapshot-body view-element-item view-element-item-hidden flex justify-center align-start wrap">
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="accountValue">
          <Text className="small-accent-text info-element">{dictionary.common.accountBalance}</Text>
        </Info>
        {renderAccountBalance()}
        <div className="assets-liabilities flex-centered">
          <Text type="success">
            {dictionary.common.assets} : {currencyAbbrev(getAccountAssets(), true)}
          </Text>
          <div className="assets-liabilities-divider"></div>
          <Text type="danger">
            {dictionary.accountSnapshot.liabilities} : {currencyAbbrev(getAccountLiabilities(), true)}
          </Text>
        </div>
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="requiredCollateral">
          <Text className="small-accent-text info-element">{dictionary.common.requiredCollateral}</Text>
        </Info>
        {renderCollateral('required')}
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="effectiveCollateral">
          <Text className="small-accent-text info-element">{dictionary.common.effectiveCollateral}</Text>
        </Info>
        {renderCollateral('effective')}
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="riskLevel">
          <Text className="small-accent-text info-element">{dictionary.common.riskLevel}</Text>
        </Info>
        {renderRiskLevel()}
        <Text type="secondary" italic>
          {walletInit && accountsInit ? dictionary.accountsView.riskMeter[`${riskLevel}Detail`] : ''}
        </Text>
        <RiskMeter />
      </div>
      <ConnectionFeedback />
    </div>
  );
}
