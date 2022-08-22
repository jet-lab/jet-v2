import { useEffect, useState, useRef } from 'react';
import { useRecoilState, useRecoilValue } from 'recoil';
import { MarginAccount, TokenAmount } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { PoolOptions, CurrentPool, usePoolFromName } from '../../state/borrow/pools';
import { WalletTokens } from '../../state/user/walletTokens';
import { CurrentAction, TokenInputAmount, TokenInputString } from '../../state/actions/actions';
import {
  useTokenInputDisabledMessage,
  useTokenInputWarningMessage,
  useTokenInputErrorMessage
} from '../../utils/actions/tokenInput';
import { getTokenAmountFromNumber } from '../../utils/currency';
import { Button, Input, Select, Slider, Typography } from 'antd';
import { TokenLogo } from './TokenLogo';

export function TokenInput(props: {
  account: MarginAccount | undefined;
  tokenSymbol?: string;
  tokenValue?: TokenAmount;
  onChangeToken: (token: string) => unknown;
  onPressEnter: () => unknown;
  loading: boolean;
}): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const poolOptions = useRecoilValue(PoolOptions);
  const currentPool = useRecoilValue(CurrentPool);
  const poolFromToken = usePoolFromName(props.tokenSymbol);
  const tokenPool = poolFromToken ?? currentPool;
  const tokenPoolRef = useRef(tokenPool);
  const walletTokens = useRecoilValue(WalletTokens);
  const currentAction = useRecoilValue(CurrentAction);
  const [tokenInputAmount, setTokenInputAmount] = useRecoilState(TokenInputAmount);
  const [tokenInputString, setTokenInputString] = useRecoilState(TokenInputString);
  const inputAmount = props.tokenValue ?? tokenInputAmount;
  const zeroInputAmount = TokenAmount.zero(tokenPool?.decimals ?? 0);
  const [maxInput, setMaxInput] = useState(zeroInputAmount);
  const disabledMessage = useTokenInputDisabledMessage(props.account);
  const warningMessage = useTokenInputWarningMessage(props.account);
  const errorMessage = useTokenInputErrorMessage(props.account);
  const { Paragraph, Text } = Typography;
  const { Option } = Select;

  // Create TokenAmount from tokenInputString and update tokenInputAmount
  function updateTokenInputAmount(stripEnding: boolean = false, adjustString: boolean = false) {
    if (!tokenPool || tokenInputString === tokenInputAmount.uiTokens) {
      return;
    }

    // Remove unnecessary 0's
    let inputString = tokenInputString;
    while (
      stripEnding &&
      inputString.includes('.') &&
      (inputString[inputString.length - 1] === '.' ||
        (inputString[inputString.length - 1] === '0' && inputString[inputString.length - 2] !== '.'))
    ) {
      inputString = inputString.substring(0, inputString.length - 1);
    }

    // Keep input within the user's max range
    const inputTokenAmount = getTokenAmountFromNumber(parseFloat(inputString), tokenPool.decimals);
    const withinMaxRange = TokenAmount.min(inputTokenAmount, maxInput);
    setTokenInputAmount(withinMaxRange);
    if (adjustString) {
      setTokenInputString(withinMaxRange.uiTokens);
    }
  }

  // If current action changes, adjust users max input if necessary
  useEffect(() => {
    if (tokenInputAmount.gt(maxInput)) {
      setTokenInputAmount(maxInput);
      setTokenInputString(maxInput.uiTokens);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentAction]);

  // Keep tokenInputAmount up to date with tokenInputString
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(updateTokenInputAmount, [tokenPool, tokenInputString]);

  // Reset tokenInput and update maxInput on action / pool change
  useEffect(() => {
    if (!currentAction || tokenPool?.symbol !== tokenPoolRef.current?.symbol) {
      setTokenInputAmount(zeroInputAmount);
      setTokenInputString(zeroInputAmount.uiTokens);
      tokenPoolRef.current = tokenPool;
    }

    let max = zeroInputAmount;
    // If props.account is undefined or currentAction is 'deposit', we will use the wallet
    if (!props.account && walletTokens && tokenPool?.symbol) {
      max = walletTokens.map[tokenPool.symbol].amount;
      // Dealing with a margin account
    } else if (currentAction && props.account?.positions && tokenPool?.symbol) {
      const poolPosition = props.account.poolPositions[tokenPool.symbol];
      max = poolPosition.maxTradeAmounts[currentAction];
    }
    setMaxInput(max);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentAction, tokenPool]);

  return (
    <div className="token-input flex-centered column">
      <div className="token-input-main flex-centered">
        <Select
          dropdownClassName="token-input-dropdown dropdown-space-between"
          value={tokenPool?.symbol}
          onChange={tokenSymbol => props.onChangeToken(tokenSymbol)}>
          {poolOptions.map(option => (
            <Option key={option.symbol} value={option.symbol}>
              <div className="flex-centered">
                <TokenLogo height={20} symbol={option.symbol} />
                <Paragraph>{option.symbol}</Paragraph>
              </div>
              <Text type="secondary">{option.name}</Text>
            </Option>
          ))}
        </Select>
        <Input
          disabled={disabledMessage.length > 0 || props.loading || maxInput.isZero()}
          value={tokenInputString}
          className={`${props.tokenValue === undefined && disabledMessage.length > 0 ? 'disabled' : ''} ${
            props.tokenValue === undefined && warningMessage.length > 0
              ? 'warning'
              : props.tokenValue === undefined && errorMessage.length > 0
              ? 'error'
              : ''
          } ${inputAmount.isZero() ? 'zeroed' : ''}`}
          onChange={e => {
            if (!tokenPool) {
              return;
            }

            // Check string value of input and adjust if necessary
            let inputString = e.target.value;
            while (!inputString.includes('.') && inputString[0] === '0') {
              inputString = inputString.substring(1);
            }
            if (isNaN(+inputString) || +inputString < 0 || +inputString > Number.MAX_SAFE_INTEGER) {
              inputString = '0';
            }
            setTokenInputString(inputString);
          }}
          onBlur={() => updateTokenInputAmount(true, true)}
          onPressEnter={() => {
            if (disabledMessage) {
              return;
            }

            const withinMaxRange = TokenAmount.min(tokenInputAmount, maxInput);
            setTokenInputAmount(withinMaxRange);
            setTokenInputString(withinMaxRange.uiTokens);
            updateTokenInputAmount(true, true);
            if (!withinMaxRange.isZero()) {
              props.onPressEnter();
            }
          }}
        />
        <Paragraph>{props.tokenSymbol ?? currentPool?.symbol ?? '—'}</Paragraph>
      </div>
      {props.tokenValue === undefined && (
        <div className="token-input-message flex align-start justify-start">
          <Text type={warningMessage ? 'warning' : errorMessage ? 'danger' : 'secondary'}>
            {disabledMessage || warningMessage || errorMessage}
          </Text>
        </div>
      )}
      {props.tokenValue === undefined && (
        <div className="token-input-slider flex align-center justify-between">
          <Slider
            value={maxInput.isZero() ? 0 : (inputAmount.tokens / maxInput.tokens) * 100}
            min={0}
            max={100}
            step={1}
            disabled={disabledMessage.length > 0 || props.loading || maxInput.isZero()}
            onChange={percent => {
              if (!tokenPool) {
                return;
              }
              const percentageAmount = maxInput.tokens * (percent / 100);
              const roundedAmount = (percentageAmount * 10 ** tokenPool.decimals) / 10 ** tokenPool.decimals;
              const roundedAmountTokens = getTokenAmountFromNumber(roundedAmount, tokenPool.decimals);
              setTokenInputString(roundedAmountTokens.uiTokens);
              updateTokenInputAmount(false, false);
            }}
            tipFormatter={value => value + '%'}
            tooltipPlacement="bottom"
            marks={{
              0: '0%',
              25: '25%',
              50: '50%',
              75: '75%',
              100: '100%'
            }}
          />
          <Button
            size="small"
            type="text"
            shape="round"
            disabled={disabledMessage.length > 0 || props.loading || maxInput.isZero()}
            className={inputAmount.eq(maxInput) ? 'active' : ''}
            onClick={() => {
              if (!tokenPool) {
                return;
              }

              const preciseMaxAmount = (maxInput.tokens * 10 ** tokenPool.decimals) / 10 ** tokenPool.decimals;
              const preciseMaxAmountTokens = getTokenAmountFromNumber(preciseMaxAmount, tokenPool.decimals);
              setTokenInputString(preciseMaxAmountTokens.uiTokens);
              updateTokenInputAmount(false, false);
            }}>
            {dictionary.common.max}
          </Button>
        </div>
      )}
    </div>
  );
}
