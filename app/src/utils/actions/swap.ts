import { useRecoilValue } from 'recoil';
import { MarginAccount, Pool, TokenAmount } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { TokenInputAmount } from '../../state/actions/actions';
import { getTokenAmountFromNumber } from '../currency';

// Calculate token output based on current prices
export function getOutputTokenAmount(
  swapAmount: TokenAmount | undefined,
  inputToken: Pool | undefined,
  outputToken: Pool | undefined
) {
  if (!swapAmount || !inputToken || !outputToken?.tokenPrice) {
    return undefined;
  }

  return getTokenAmountFromNumber(
    (swapAmount.tokens * inputToken.tokenPrice) / outputToken.tokenPrice,
    outputToken.decimals
  );
}

// Calculate minimum output based on input and slippage
export function getMinOutputAmount(
  swapAmount: TokenAmount | undefined,
  inputToken: Pool | undefined,
  outputToken: Pool | undefined,
  slippage: number
) {
  const outputAmount =
    getOutputTokenAmount(swapAmount, inputToken, outputToken) ?? TokenAmount.zero(outputToken?.decimals ?? 6);
  return getTokenAmountFromNumber(outputAmount.tokens - outputAmount.tokens * slippage, outputAmount.decimals);
}

// Show review message for swap
export function useSwapReviewMessage(
  swapAccount: MarginAccount | undefined,
  inputToken: Pool | undefined,
  outputToken: Pool | undefined
): string {
  const dictionary = useRecoilValue(Dictionary);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  if (!swapAccount || !inputToken || !outputToken || tokenInputAmount.isZero()) {
    return '';
  }
  const swapAccountPoolPosition = swapAccount.poolPositions[inputToken.symbol];
  const outputTokenAmount = getOutputTokenAmount(tokenInputAmount, inputToken, outputToken);
  if (!swapAccountPoolPosition) {
    return '';
  }

  // Margin swapping (totally) review
  if (!swapAccountPoolPosition.depositBalance.tokens) {
    return dictionary.actions.swap.reviewMessages.marginSwapTotalReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.tokens.toPrecision(inputToken.decimals / 2))
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.tokens.toPrecision(outputTokenAmount.decimals / 2) ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
    // Margin swapping (partially) review
  } else if (swapAccountPoolPosition.depositBalance.tokens < tokenInputAmount.tokens) {
    const marginAmount = tokenInputAmount.tokens - swapAccountPoolPosition.depositBalance.tokens;
    return dictionary.actions.swap.reviewMessages.marginSwapPartialReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.tokens.toPrecision(inputToken.decimals / 2))
      .replaceAll('{{MARGIN_AMOUNT}}', marginAmount.toPrecision(inputToken.decimals / 2))
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.tokens.toPrecision(outputTokenAmount.decimals / 2) ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
    // A normal swap review
  } else {
    return dictionary.actions.swap.reviewMessages.swapReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.tokens.toPrecision(inputToken.decimals / 2))
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.tokens.toPrecision(outputTokenAmount.decimals / 2) ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
  }
}
