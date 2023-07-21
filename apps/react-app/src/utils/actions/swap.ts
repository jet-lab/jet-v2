import { PublicKey } from '@solana/web3.js';
import { TokenAmount } from '@jet-lab/margin';
import axios from 'axios';

export async function getSwapRoutes(
  endpoint: string,
  sourceToken: PublicKey,
  targetToken: PublicKey,
  swapAmount: TokenAmount
): Promise<SwapQuote[] | undefined> {
  console.log('sending request', Date.now())
  return (
    await axios.get<any, any>(
      `${endpoint}/swap/quote/${sourceToken.toBase58()}/${targetToken.toBase58()}/${swapAmount.lamports.toNumber()}`
      , {
        timeout: 1000,
      })
  ).data;
}

export interface SwapQuote {
  token_in: string;
  token_out: string;
  tokens_in: number;
  tokens_out: number;
  market_price: number;
  trade_price: number;
  effective_price: number;
  price_impact: number;
  fees: Record<string, number>;
  swaps: SwapStepOutput[][];
}

type SwapStepOutput = SwapStep | SwapOutput;

export interface SwapStep {
  from_token: string;
  to_token: string;
  program: string;
  swap_pool: string;
}

export interface SwapOutput {
  tokens_out: number;
  tokens_in: number;
  fee_tokens_in: number;
  fee_tokens_out: number;
  market_price: number;
  trade_price: number;
  effective_price: number;
  price_impact: number;
  unfilled_tokens_in: number;
}
