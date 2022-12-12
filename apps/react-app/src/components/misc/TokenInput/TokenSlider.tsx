import { TokenAmount } from '@jet-lab/margin';
import { useRecoilValue, useSetRecoilState } from 'recoil';
import { Dictionary } from '@state/settings/localization/localization';
import { CurrentPool } from '@state/pools/pools';
import { TokenInputAmount, TokenInputString } from '@state/actions/actions';
import { Button, Slider } from 'antd';
import { getTokenAmountFromNumber } from '@utils/currency';
import { ReactNode } from 'react';

// Slider component for the TokenInput
export function TokenSlider(props: {
  // The maximum input value of the TokenInput
  maxInput: TokenAmount;
  // When to disable slider
  disabled: boolean;
}): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const currentPool = useRecoilValue(CurrentPool);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  const setTokenInputString = useSetRecoilState(TokenInputString);
  const formatter = (value?: number): ReactNode => `${value}%`;

  // Returns a slider percentage based on current input and max input
  function getSliderValue() {
    let value = 0;
    if (!props.maxInput.isZero()) {
      value = (tokenInputAmount.tokens / props.maxInput.tokens) * 100;
    }

    return value;
  }

  // Handle the slider value change
  function handleChange(percent: number) {
    if (!currentPool) {
      return;
    }

    const percentageAmount = props.maxInput.tokens * (percent / 100);
    const roundedAmount = (percentageAmount * 10 ** currentPool.decimals) / 10 ** currentPool.decimals;
    const roundedAmountTokens = getTokenAmountFromNumber(roundedAmount, currentPool.decimals);
    setTokenInputString(roundedAmountTokens.tokens.toString());
  }

  // Handle "max" button click
  function handleMaxClick() {
    if (!currentPool) {
      return;
    }

    const preciseMaxAmount = (props.maxInput.tokens * 10 ** currentPool.decimals) / 10 ** currentPool.decimals;
    const preciseMaxAmountTokens = getTokenAmountFromNumber(preciseMaxAmount, currentPool.decimals);
    setTokenInputString(preciseMaxAmountTokens.tokens.toString());
  }

  return (
    <div className="token-input-slider flex align-center justify-between">
      <Slider
        value={getSliderValue()}
        min={0}
        max={100}
        step={1}
        disabled={props.disabled}
        onChange={(percent: number) => handleChange(percent)}
        tooltip={{ formatter, placement: 'bottom' }}
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
        disabled={props.disabled}
        className={tokenInputAmount.eq(props.maxInput) ? 'active' : ''}
        onClick={handleMaxClick}>
        {dictionary.common.max}
      </Button>
    </div>
  );
}
