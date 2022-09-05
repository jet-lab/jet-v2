import { useEffect, useState, useRef } from 'react';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { MarginAccount, PoolAction, TokenAmount } from '@jet-lab/margin';
import { CurrentPool, usePoolFromName } from '../../../state/borrow/pools';
import { WalletTokens } from '../../../state/user/walletTokens';
import { CurrentAction, MaxTradeAmounts, TokenInputAmount, TokenInputString } from '../../../state/actions/actions';
import {
  useTokenInputDisabledMessage,
  useTokenInputWarningMessage,
  useTokenInputErrorMessage
} from '../../../utils/actions/tokenInput';
import { getTokenAmountFromNumber } from '../../../utils/currency';
import { TokenSelect } from './TokenSelect';
import { TokenSlider } from './TokenSlider';
import { Input, Typography } from 'antd';

// Main component for token inputs when the user takes one of the main actions (deposit, borrow, etc)
export function TokenInput(props: {
  // A specific account to base input properties on (like maximum input, errors, etc)
  account: MarginAccount | undefined;
  // Specify what occurs when the user switches their token
  onChangeToken: (token: string) => unknown;
  // Specify what occurs if user presses enter while focusing the input
  onPressEnter: () => unknown;
  // Specify a condition to show the input's loading state
  loading: boolean;
  // Optionally, can specify what token the input is referencing (overrides currentPool reference)
  tokenSymbol?: string;
  // Optionally, can specify what token value of the input is (if based on other factors, overrides tokenInputAmount state)
  tokenValue?: TokenAmount;
  // Optionally, can specify which action to base input's references on (for instance, only show maximum inputs for the Swap action)
  action?: PoolAction;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  // The pool being interacted with (or specified externally)
  const currentPool = useRecoilValue(CurrentPool);
  const poolFromToken = usePoolFromName(props.tokenSymbol);
  const tokenPool = poolFromToken ?? currentPool;
  const tokenPoolRef = useRef(tokenPool);
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAction = useRecoilValue(CurrentAction);
  // If an action was specified, reference that action otherwise reference the currentAction
  const tokenAction = props.action ?? currentAction;
  // We track user input as a string (so they can enter decimals ex: '0.0001'), but then parse into a TokenAmount
  const [tokenInputAmount, setTokenInputAmount] = useRecoilState(TokenInputAmount);
  // Track/update the user's input amount as a string, to be converted to a TokenAmount
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  // If an input amount was specified, reference that amount otherwise reference the current tokenInputAmount
  const inputAmount = props.tokenValue ?? tokenInputAmount;
  const zeroInputAmount = TokenAmount.zero(tokenPool?.decimals ?? 0);
  const [maxInput, setMaxInput] = useState(zeroInputAmount);
  const setMaxTradeAmounts = useSetRecoilState(MaxTradeAmounts);
  const disabledMessage = useTokenInputDisabledMessage(props.account);
  const warningMessage = useTokenInputWarningMessage(props.account);
  const errorMessage = useTokenInputErrorMessage(props.account);
  // If we're given an external value, are loading, can't enter an input or have a disabled message
  const disabled = props.tokenValue !== undefined || disabledMessage.length > 0 || maxInput.isZero();
  const { Paragraph, Text } = Typography;

  // If current action changes, keep input within the max range
  useEffect(() => {
    if (tokenInputAmount.gt(maxInput)) {
      setTokenInputAmount(maxInput);
      setTokenInputString(maxInput.uiTokens);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenAction]);

  // Keep tokenInputAmount up to date with tokenInputString
  useEffect(() => {
    // Create TokenAmount from tokenInputString and update tokenInputAmount
    if (!tokenPool || tokenInputString === tokenInputAmount.uiTokens || props.tokenValue !== undefined) {
      return;
    }

    // Remove unnecessary 0's from beginning / end of input string
    let inputString = tokenInputString;
    while (
      inputString.includes('.') &&
      (inputString[inputString.length - 1] === '.' ||
        (inputString[inputString.length - 1] === '0' && inputString[inputString.length - 2] !== '.'))
    ) {
      inputString = inputString.substring(0, inputString.length - 1);
    }

    // Keep input within the user's max range
    const inputTokenAmount = getTokenAmountFromNumber(parseFloat(inputString), tokenPool.decimals);
    const withinMaxRange = TokenAmount.min(inputTokenAmount, maxInput);

    // Adjust state
    setTokenInputAmount(withinMaxRange);
  }, [
    tokenPool,
    tokenInputString,
    tokenInputAmount.uiTokens,
    props.tokenValue,
    maxInput,
    setTokenInputAmount,
    setTokenInputString
  ]);

  // Reset tokenInput and update maxInput on action / pool change
  useEffect(() => {
    // Use the reference to see if we've changed pools, and if so reset
    const hasChangedPools = tokenPool?.symbol !== tokenPoolRef.current?.symbol;
    if (hasChangedPools) {
      setTokenInputAmount(zeroInputAmount);
      setTokenInputString(zeroInputAmount.uiTokens);
      tokenPoolRef.current = tokenPool;
    }

    let max = zeroInputAmount;
    // If props.account is undefined or tokenAction is 'deposit', we will use the WALLET
    if (!props.account && walletTokens && tokenPool?.symbol) {
      max = walletTokens.map[tokenPool.symbol].amount;
      // Otherwise we're using an ACCOUNT
    } else if (tokenAction && props.account?.positions && tokenPool?.symbol) {
      const poolPosition = props.account.poolPositions[tokenPool.symbol];
      if (poolPosition) {
        const maxTradeAmounts = poolPosition.maxTradeAmounts;
        setMaxTradeAmounts(maxTradeAmounts);
        max = maxTradeAmounts[tokenAction];
      }
    }
    setMaxInput(max);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenAction, tokenPool, props.account]);

  // Handle / parse user input from a string
  function handleInputChange(inputString: string) {
    if (!tokenPool || props.tokenValue !== undefined) {
      return;
    }

    // Check string value of input and adjust if necessary
    while (!inputString.includes('.') && inputString.length > 1 && inputString[0] === '0') {
      inputString = inputString.substring(1);
    }
    // Check if input is invalid (less than 0, or NaN, or too large)
    if (isNaN(+inputString) || +inputString < 0 || +inputString > Number.MAX_SAFE_INTEGER) {
      inputString = '0';
    }

    // Update our input string tracker
    setTokenInputString(inputString);
  }

  // If not disabled, execute on enter keypress
  function handlePressEnter() {
    if (disabledMessage) {
      return;
    }

    const withinMaxRange = TokenAmount.min(tokenInputAmount, maxInput);
    setTokenInputAmount(withinMaxRange);
    setTokenInputString(withinMaxRange.uiTokens);
    if (!withinMaxRange.isZero()) {
      props.onPressEnter();
    }
  }

  // Returns the string of classNames for TokenInput
  function getClassNames() {
    let classNames = '';

    // If disabled
    if (disabledMessage.length > 0) {
      classNames += 'disabled';
    }

    // Has no input, style as such
    if (inputAmount.isZero()) {
      classNames += ' zeroed';
    }

    // Not disabled and has a warning or error message
    if (!disabled && warningMessage.length) {
      classNames += ' warning';
    }
    if (!disabled && errorMessage.length) {
      classNames += ' error';
    }

    return classNames;
  }

  // Render TokenSlider (if no external value is provided)
  function renderTokenSlider() {
    let render = <></>;
    if (!props.tokenValue) {
      render = <TokenSlider maxInput={maxInput} disabled={props.loading || disabled} />;
    }

    return render;
  }

  // Render input feedback / disabled messages (if no external value is provided)
  function renderFeedbackMessage() {
    let render = <></>;
    if (!props.tokenValue) {
      render = (
        <div className="token-input-message flex align-start justify-start">
          <Text type={warningMessage ? 'warning' : errorMessage ? 'danger' : 'secondary'}>
            {disabledMessage || warningMessage || errorMessage}
          </Text>
        </div>
      );
    }

    return render;
  }

  return (
    <div className="token-input flex-centered column">
      <div className="token-input-main flex-centered">
        <TokenSelect tokenPool={tokenPool} onChangeToken={props.onChangeToken} dropdownStyle={props.dropdownStyle} />
        <Input
          disabled={props.loading || disabled}
          value={props.tokenValue ? props.tokenValue.uiTokens : tokenInputString}
          className={getClassNames()}
          onChange={e => handleInputChange(e.target.value)}
          onPressEnter={() => handlePressEnter()}
        />
        <Paragraph>{props.tokenSymbol ?? currentPool?.symbol ?? '—'}</Paragraph>
      </div>
      {renderFeedbackMessage()}
      {renderTokenSlider()}
    </div>
  );
}
