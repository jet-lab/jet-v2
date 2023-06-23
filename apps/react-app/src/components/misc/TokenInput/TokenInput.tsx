import { useEffect, useMemo, useState } from 'react';
import { useRecoilState, useRecoilValue, useResetRecoilState, useSetRecoilState } from 'recoil';
import { feesBuffer, MarginAccount, TokenAmount, PoolAction, Pool } from '@jet-lab/margin';
import { PoolOption, Pools } from '@state/pools/pools';
import {
  ActionRefresh,
  CurrentAction,
  MaxTradeAmounts,
  SendingTransaction,
  TokenInputAmount,
  TokenInputString
} from '@state/actions/actions';
import {
  useTokenInputDisabledMessage,
  useTokenInputWarningMessage,
  useTokenInputErrorMessage
} from '@utils/actions/tokenInput';
import { DEFAULT_DECIMALS, getTokenAmountFromNumber } from '@utils/currency';
import { TokenSelect } from './TokenSelect';
import { TokenSlider } from './TokenSlider';
import { Input, Typography } from 'antd';
import { WalletTokens } from '@state/user/walletTokens';
import { CurrentAccount } from '@state/user/accounts';
import { fromLocaleString } from '@utils/format';
import debounce from 'lodash.debounce';
import { useJetStore } from '@jet-lab/store';

// Main component for token inputs when the user takes one of the main actions (deposit, borrow, etc)
export function TokenInput(props: {
  // Optionally, specify a value to be represented by this input (default to tokenInputAmount)
  value?: TokenAmount;
  // Optionally specify an account (defaults to currentAccount)
  account?: MarginAccount;
  // Optionally, can specify which action to base input's references on (defaults to currentAction)
  action?: PoolAction;
  // Optionally, specify which pool this input should base its references on (defaults to currentPool)
  poolSymbol?: string;
  // Optionally, specify which tokens a user can choose from in the TokenSelect
  tokenOptions?: PoolOption[];
  // Optionally, specify what occurs when the user switches their token (defaults to updating currentPool)
  onChangeToken?: (token: string) => unknown;
  // Specify what occurs if user presses enter while focusing the input
  onPressEnter: () => unknown;
  // Optionally, hide input slider
  hideSlider?: boolean;
  // Optionally, override the styles of the token selector dropdown
  dropdownStyle?: React.CSSProperties;
}): JSX.Element {
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAccount = useRecoilValue(CurrentAccount);
  const selectedPoolKey = useJetStore(state => state.selectedPoolKey);
  const account = props.account ?? currentAccount;
  // The pool being interacted with (or specified externally)
  const pools = useRecoilValue(Pools);
  const tokenPool = useMemo(
    () =>
      pools?.tokenPools &&
      Object.values(pools?.tokenPools).find(pool =>
        props.poolSymbol ? pool.symbol === props.poolSymbol : pool.address.toBase58() === selectedPoolKey
      ),
    [selectedPoolKey, props.poolSymbol, pools]
  );
  const currentAction = useRecoilValue(CurrentAction);
  // If an action was specified, reference that action otherwise reference the currentAction
  const tokenAction = props.action ?? currentAction;
  // We track user input as a string (so they can enter decimals ex: '0.0001'), but then parse into a TokenAmount
  const [tokenInputAmount, setTokenInputAmount] = useRecoilState(TokenInputAmount);
  const resetTokenInputAmount = useResetRecoilState(TokenInputAmount);
  // Track/update the user's input amount as a string, to be converted to a TokenAmount
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const resetTokenInputString = useResetRecoilState(TokenInputString);
  // If an input amount was specified, reference that amount otherwise reference the current tokenInputAmount
  const inputAmount = props.value ?? tokenInputAmount;
  const [maxTradeAmounts, setMaxTradeAmounts] = useRecoilState(MaxTradeAmounts);
  const disabledMessage = useTokenInputDisabledMessage();
  const warningMessage = useTokenInputWarningMessage();
  const errorMessage = useTokenInputErrorMessage();
  // If we're given an external value, are sendingTransaction, can't enter an input or have a disabled message
  const sendingTransaction = useRecoilValue(SendingTransaction);
  const setActionRefresh = useSetRecoilState(ActionRefresh);

  const zeroInputAmount = useMemo(
    () => TokenAmount.zero(tokenPool?.decimals ?? DEFAULT_DECIMALS),
    [tokenPool?.decimals]
  );
  const [maxInput, setMaxInput] = useState(zeroInputAmount);

  const disabled = sendingTransaction || props.value !== undefined || disabledMessage.length > 0 || maxInput.isZero();
  const { Text } = Typography;

  useEffect(() => {
    setActionRefresh(Date.now());
  }, [tokenPool]);

  // If current action changes, keep input within the maxInput range
  useEffect(() => {
    if (tokenInputAmount.gte(maxInput)) {
      setTokenInputAmount(maxInput);
      setTokenInputString(maxInput.tokens.toString());
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenAction]);

  const debouncedUpdateTokenAmount = useMemo(
    () =>
      debounce((tokenPool: Pool, tokenInputString: string, maxInput: TokenAmount, value?: TokenAmount) => {
        // Create TokenAmount from tokenInputString and update tokenInputAmount
        if (!tokenPool || value !== undefined) {
          return;
        }

        // Remove unnecessary 0's from beginning / end of input string
        const inputString = parseFloat(fromLocaleString(tokenInputString)).toString();

        // Keep input within the user's maxInput range
        const inputTokenAmount = getTokenAmountFromNumber(parseFloat(inputString), tokenPool.decimals);
        const withinMaxRange = TokenAmount.min(inputTokenAmount, maxInput);

        // Adjust state
        setTokenInputAmount(withinMaxRange);
        if (inputTokenAmount.gt(withinMaxRange)) {
          const { format } = new Intl.NumberFormat(navigator.language);
          setTokenInputString(format(withinMaxRange.tokens));
        }
      }, 300),
    []
  );

  // Keep tokenInputAmount up to date with tokenInputString
  useEffect(() => {
    if (tokenPool) {
      debouncedUpdateTokenAmount(tokenPool, tokenInputString, maxInput, props.value);
    }
  }, [tokenPool, tokenInputString, props.value, maxInput]);

  // Update maxInput on pool position update
  useEffect(() => {
    if (!tokenPool || !tokenAction || !account) {
      return;
    }

    let maxInput = zeroInputAmount;
    const poolPosition = account.poolPositions[tokenPool.symbol];

    if (poolPosition) {
      const maxInputTradeAmounts = poolPosition.maxTradeAmounts;
      setMaxTradeAmounts(maxInputTradeAmounts);

      // If user is depositing or swapping with jupiter, reference their wallet
      if (tokenAction === 'deposit') {
        maxInput = walletTokens ? walletTokens.map[tokenPool.symbol]?.amount : zeroInputAmount;
        // If SOL, need to save some for fees
        if (tokenPool.symbol === 'SOL') {
          maxInput = maxInput.subb(feesBuffer);
        }
        // Otherwise reference their margin account
      } else if (tokenAction === 'repay') {
        // If the deposit balance > loan, constrain to loan, else deposit
        if (poolPosition.depositBalance.gt(poolPosition.loanBalance)) {
          maxInput = poolPosition.loanBalance;
        } else {
          maxInput = poolPosition.depositBalance;
        }
      } else {
        maxInput = maxTradeAmounts[tokenAction] ?? maxInputTradeAmounts[tokenAction];
      }
      setMaxInput(maxInput);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenAction, tokenPool?.symbol, account?.poolPositions]);

  // Reset input on action / pool change
  useEffect(() => {
    resetTokenInputAmount();
    resetTokenInputString();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tokenAction, tokenPool?.symbol]);

  // Handle / parse user input from a string
  function handleInputChange(inputString: string) {
    if (!tokenPool || props.value !== undefined) {
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
    setTokenInputString(withinMaxRange.tokens.toString());
    if (!withinMaxRange.isZero()) {
      props.onPressEnter();
    }
  }

  // Returns the string of classNames for TokenInput
  function getClassNames() {
    let classNames = '';

    // If disabled
    if (!props.value && disabledMessage.length > 0) {
      classNames += 'disabled';
    }

    // Has no input, style as such
    if (inputAmount.isZero()) {
      classNames += 'zeroed';
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
    if (!props.hideSlider && !props.value) {
      render = <TokenSlider maxInput={maxInput} disabled={disabled} />;
    }

    return render;
  }

  // Render input feedback / disabled messages (if no external value is provided)
  function renderFeedbackMessage() {
    let render = <></>;
    if (!props.value) {
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

  // Reset recoil state
  const reset = useResetRecoilState(MaxTradeAmounts);

  return (
    <div className={`token-input flex-centered column ${props.value ? 'external-value' : ''}`}>
      <div
        data-testid="reset-max-trade"
        className="pixel"
        onClick={() => {
          reset();
        }}
      />
      <div className="token-input-main flex-centered">
        <TokenSelect
          poolSymbol={tokenPool?.symbol}
          tokenOptions={props.tokenOptions}
          onChangeToken={props.onChangeToken}
          dropdownStyle={props.dropdownStyle}
        />
        <Input
          disabled={disabled}
          value={props.value ? props.value.uiTokens : tokenInputString}
          className={getClassNames()}
          onChange={e => handleInputChange(e.target.value)}
          onPressEnter={() => {
            if (!disabled) {
              setTokenInputString(tokenInputAmount.tokens.toString());
              handlePressEnter();
            }
          }}
          onBlur={() => setTokenInputString(tokenInputAmount.tokens.toString())}
        />
      </div>
      {renderFeedbackMessage()}
      {renderTokenSlider()}
    </div>
  );
}
