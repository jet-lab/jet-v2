import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { WalletTokens } from '@state/user/walletTokens';
import { Accounts, CurrentAccount } from '@state/user/accounts';
import { useCurrencyFormatting } from '@utils/currency';
import { formatRiskIndicator } from '@utils/format';
import { useRiskStyle } from '@utils/risk';
import { Typography, Skeleton } from 'antd';
import { ConnectionFeedback } from '../ConnectionFeedback/ConnectionFeedback';
import { Info } from '../Info';
import { RiskMeter } from '../RiskMeter';
import axios from 'axios';
import { USDConversionRates } from '@state/settings/settings';
import { useEffect } from 'react';

// Body of the Account Snapshot, where users can see data for the currently selected margin account
export function SnapshotBody(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const setUsdConversion = useSetRecoilState(USDConversionRates);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const walletTokens = useRecoilValue(WalletTokens);
  const accounts = useRecoilValue(Accounts);
  const initialAccountsLoad = walletTokens && !accounts.length;
  const currentAccount = useRecoilValue(CurrentAccount);
  const riskStyle = useRiskStyle();
  const { Title, Text } = Typography;

  useEffect(() => {
    axios
      .get('https://api.jetprotocol.io/v1/rates')
      .then(resp => {
        const conversions = resp.data;
        if (conversions) {
          setUsdConversion(conversions.rates);
        }
      })
      .catch(err => err);
  }, []);

  // Renders the account balance
  function renderAccountBalance() {
    // The account balance (deposits - liabilities)
    let accountBalance = 0;
    if (currentAccount && currentAccount.summary) {
      const depositedValue = currentAccount.summary.depositedValue;
      const borrowedValue = currentAccount.summary.borrowedValue;
      accountBalance = depositedValue - borrowedValue;
    }

    let render = <Title>{currencyFormatter(accountBalance, true, 0)}</Title>;
    if (initialAccountsLoad) {
      render = <Skeleton className="align-center" paragraph={false} active />;
    }

    return render;
  }

  // Renders the account's available collateral
  function renderAvailableCollateral() {
    const availableCollateral = currentAccount
      ? currentAccount.valuation.effectiveCollateral.sub(currentAccount.valuation.requiredCollateral).toNumber()
      : 0;
    let render = <Title>{currencyFormatter(availableCollateral, true, 0)}</Title>;
    if (initialAccountsLoad) {
      render = <Skeleton className="align-center" paragraph={false} active />;
    }

    return render;
  }

  // Renders the account's required/effective collateral
  function getCollateral(type: 'required' | 'effective') {
    const requiredCollateral = currentAccount?.valuation.requiredCollateral.toNumber() ?? 0;
    const effectiveCollateral = currentAccount?.valuation.effectiveCollateral.toNumber() ?? 0;
    let collateral = requiredCollateral;
    if (type === 'effective') {
      collateral = effectiveCollateral;
    }

    return collateral;
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
    <div className="account-snapshot-body flex justify-center align-start wrap">
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="accountValue">
          <Text className="small-accent-text info-element">{dictionary.common.accountBalance}</Text>
        </Info>
        {renderAccountBalance()}
        <div className="assets-liabilities flex-centered">
          <Text type="success">
            {dictionary.common.assets} : {currencyAbbrev(getAccountAssets(), 1, true, undefined)}
          </Text>
          <div className="assets-liabilities-divider"></div>
          <Text type="danger">
            {dictionary.accountSnapshot.liabilities} : {currencyAbbrev(getAccountLiabilities(), 1, true, undefined)}
          </Text>
        </div>
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="availableCollateral">
          <Text className="small-accent-text info-element">{dictionary.common.availableCollateral}</Text>
        </Info>
        {renderAvailableCollateral()}
        <div className="assets-liabilities flex-centered">
          <Text type="secondary">
            {dictionary.common.effective} : {currencyAbbrev(getCollateral('effective'), 1, true, undefined)}
          </Text>
          <div className="assets-liabilities-divider"></div>
          <Text type="secondary">
            {dictionary.common.required} : {currencyAbbrev(getCollateral('required'), 1, true, undefined)}
          </Text>
        </div>
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="accountLeverage">
          <Text className="small-accent-text info-element">Account Leverage</Text>
        </Info>
        <Title>{currentAccount?.summary.leverage ? `${currentAccount.summary.leverage.toFixed(2)}x` : '1.00x'}</Title>
      </div>
      <div className="account-snapshot-body-item flex-centered column">
        <Info term="riskLevel">
          <Text className="small-accent-text info-element">{dictionary.common.riskLevel}</Text>
        </Info>
        {renderRiskLevel()}
        <RiskMeter />
      </div>
      <ConnectionFeedback />
    </div>
  );
}
