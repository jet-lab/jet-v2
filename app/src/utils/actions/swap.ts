import { useRecoilValue } from 'recoil';
import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { computeOutputAmount } from '@orca-so/stablecurve';
import { BN } from 'bn.js';
import { MarginAccount, Pool, SPLSwapPool, TokenAmount } from '@jet-lab/margin';
import { Dictionary } from '../../state/settings/localization/localization';
import { TokenInputAmount } from '../../state/actions/actions';
import { DEFAULT_DECIMALS, getTokenAmountFromNumber } from '../currency';

// Calculate the token output for a constant product swap
function constantProductSwap(sourceAmount: number, swapSourceAmount: number, swapDestinationAmount: number): number[] {
  const invariant = swapSourceAmount * swapDestinationAmount;
  const newSwapSourceAmount = swapSourceAmount + sourceAmount;
  const newSwapDestinationAmount = invariant / newSwapSourceAmount;

  const sourceAmountSwapped = newSwapSourceAmount - swapSourceAmount;
  const destinationAmountSwapped = swapDestinationAmount - newSwapDestinationAmount;

  return [sourceAmountSwapped, destinationAmountSwapped];
}

// Calculate token output based on current prices
export function getOutputTokenAmount(
  swapAmount: TokenAmount | undefined,
  sourceTokenAmount: TokenAmount | undefined,
  destinationTokenAmount: TokenAmount | undefined,
  poolType: 'constantProduct' | 'stable' | undefined,
  fees: number,
  amp: number = 100
): TokenAmount | undefined {
  if (
    !swapAmount ||
    swapAmount.isZero() ||
    !sourceTokenAmount ||
    sourceTokenAmount.isZero() ||
    !destinationTokenAmount ||
    destinationTokenAmount.isZero()
  ) {
    return undefined;
  }

  if (poolType === 'constantProduct') {
    const [, b] = constantProductSwap(
      swapAmount.tokens * (1 - fees),
      sourceTokenAmount.tokens,
      destinationTokenAmount.tokens
    );

    return getTokenAmountFromNumber(b, destinationTokenAmount.decimals);
  } else if (poolType === 'stable') {
    const outputAmount = computeOutputAmount(
      swapAmount.lamports,
      sourceTokenAmount.lamports,
      destinationTokenAmount.lamports,
      new BN(amp)
    );
    return new TokenAmount(outputAmount, destinationTokenAmount.decimals);
  } else {
    return;
  }
}

// Calculate minimum output based on input and slippage
export function getMinOutputAmount(
  swapAmount: TokenAmount | undefined,
  sourceTokenAmount: TokenAmount | undefined,
  destinationTokenAmount: TokenAmount | undefined,
  poolType: 'constantProduct' | 'stable' | undefined,
  fees: number,
  slippage: number,
  amp: number = 100
) {
  const outputAmount =
    getOutputTokenAmount(swapAmount, sourceTokenAmount, destinationTokenAmount, poolType, fees) ??
    TokenAmount.zero(destinationTokenAmount?.decimals ?? DEFAULT_DECIMALS);
  return getTokenAmountFromNumber(outputAmount.tokens - outputAmount.tokens * slippage, outputAmount.decimals);
}

// Show review message for swap
export function useSwapReviewMessage(
  swapAccount: MarginAccount | undefined,
  inputToken: Pool | undefined,
  outputToken: Pool | undefined,
  sourceTokenAmount: TokenAmount | undefined,
  destinationTokenAmount: TokenAmount | undefined,
  poolType: 'constantProduct' | 'stable' | undefined,
  fees: number,
  amp: number = 100
): string {
  const dictionary = useRecoilValue(Dictionary);
  const tokenInputAmount = useRecoilValue(TokenInputAmount);
  if (
    !swapAccount ||
    !inputToken ||
    !outputToken ||
    !sourceTokenAmount ||
    !destinationTokenAmount ||
    tokenInputAmount.isZero()
  ) {
    return '';
  }
  const swapAccountPoolPosition = swapAccount.poolPositions[inputToken.symbol];
  const outputTokenAmount = getOutputTokenAmount(
    tokenInputAmount,
    sourceTokenAmount,
    destinationTokenAmount,
    poolType,
    fees
  );
  if (!swapAccountPoolPosition) {
    return '';
  }

  // Margin swapping (totally) review
  if (!swapAccountPoolPosition.depositBalance.tokens) {
    return dictionary.actions.swap.reviewMessages.marginSwapTotalReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens)
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.uiTokens ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
    // Margin swapping (partially) review
  } else if (swapAccountPoolPosition.depositBalance.tokens < tokenInputAmount.tokens) {
    const marginAmount = tokenInputAmount.tokens - swapAccountPoolPosition.depositBalance.tokens;
    return dictionary.actions.swap.reviewMessages.marginSwapPartialReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens)
      .replaceAll('{{MARGIN_AMOUNT}}', marginAmount.toFixed(inputToken.decimals))
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.uiTokens ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
    // A normal swap review
  } else {
    return dictionary.actions.swap.reviewMessages.swapReview
      .replaceAll('{{AMOUNT}}', tokenInputAmount.uiTokens)
      .replaceAll('{{INPUT_TOKEN}}', inputToken.symbol)
      .replaceAll('{{SIZE}}', outputTokenAmount?.uiTokens ?? '')
      .replaceAll('{{OUTPUT_TOKEN}}', outputToken.symbol);
  }
}

// Generate swap prices for charts
export function generateSwapPrices(
  sourceTokensIn: number,
  destinationTokensOut: number,
  maxSwappableAmount: number,
  poolType: 'constantProduct' | 'stable' | undefined,
  fees: number,
  sourceDecimals: number,
  destinationDecimals: number,
  swapAtoB: boolean,
  amp: number = 100
): number[][] {
  const swappedAmounts = [];

  // The below commented out code creates more points on the chart,
  // but at a performance cost to time-to-render.
  // We can tweak the logic based on feedback from users.

  const interval =
    maxSwappableAmount > 0
      ? maxSwappableAmount / 50
      : Math.round(swapAtoB ? sourceTokensIn * 0.0005 : destinationTokensOut * 0.0005);
  const swapMaxToken =
    maxSwappableAmount > 0
      ? maxSwappableAmount * 1.1
      : Math.round(swapAtoB ? sourceTokensIn * 0.02 : destinationTokensOut * 0.02);
  let tokenSwap = interval * 0.02;
  const priceWithFee = swapAtoB ? 1 - fees : 1 + fees;

  // Move the if-else out of the loop
  if (poolType === 'constantProduct') {
    while (tokenSwap <= swapMaxToken) {
      const [a, b] = constantProductSwap(tokenSwap * (1.0 - fees), sourceTokensIn, destinationTokensOut);
      const sellForPrice = [
        tokenSwap / sourceDecimals,
        (swapAtoB ? b / destinationDecimals / (a / sourceDecimals) : a / sourceDecimals / (b / destinationDecimals)) *
          priceWithFee
      ];
      swappedAmounts.push(sellForPrice);
      // Decrement tokens to swap
      tokenSwap += interval;
    }
  } else if (poolType === 'stable') {
    while (tokenSwap <= swapMaxToken) {
      const a = tokenSwap * (1.0 - fees);
      const outputAmount = computeOutputAmount(
        new BN(a),
        new BN(sourceTokensIn),
        new BN(destinationTokensOut),
        new BN(amp)
      );
      const b = outputAmount.toNumber();
      const sellForPrice = [
        tokenSwap / sourceDecimals,
        (swapAtoB ? b / destinationDecimals / (a / sourceDecimals) : a / sourceDecimals / (b / destinationDecimals)) *
          priceWithFee
      ];
      swappedAmounts.push(sellForPrice);
      // Decrement tokens to swap
      tokenSwap += interval;
    }
  }

  return swappedAmounts;
}

export async function getSwapPoolPrice(provider: AnchorProvider, pool: SPLSwapPool) {
  const tokenAccountBalanceA = (await provider.connection.getTokenAccountBalance(new PublicKey(pool.tokenA))).value;
  const tokenAccountBalanceB = (await provider.connection.getTokenAccountBalance(new PublicKey(pool.tokenB))).value;
  const balanceTokenA = getTokenAmountFromNumber(tokenAccountBalanceA?.uiAmount ?? 0, tokenAccountBalanceA.decimals);
  const balanceTokenB = getTokenAmountFromNumber(tokenAccountBalanceB?.uiAmount ?? 0, tokenAccountBalanceB.decimals);
  return {
    balanceTokenA,
    balanceTokenB
  };
}
