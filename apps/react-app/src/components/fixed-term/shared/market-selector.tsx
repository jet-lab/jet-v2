import { useRecoilState, useRecoilValue } from 'recoil';
import { AllFixedTermMarketsAtom, SelectedFixedTermMarketAtom } from '@state/fixed-term/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { Dispatch, SetStateAction, useEffect, useState } from 'react';
import {
  MarginAccount,
  repay,
  TokenAmount,
  settle,
  MarketAndconfig,
  FixedTermMarket,
  Pool,
  AssociatedToken
} from '@jet-lab/margin';
import { getExplorerUrl } from '@utils/ui';
import { notify } from '@utils/notify';
import { Address, AnchorProvider } from '@project-serum/anchor';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { useProvider } from '@utils/jet/provider';
import { JetMarginPools, Pools } from '@state/pools/pools';
import BN from 'bn.js';
import { Loan, useOpenPositions } from '@jet-lab/store';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';

const { Option } = Select;

interface FixedTermMarketSelectorProps {
  type: 'asks' | 'bids';
}

const settleNow = async (
  marginAccount: MarginAccount,
  markets: MarketAndconfig[],
  selectedMarket: number,
  provider: AnchorProvider,
  setOwedTokens: Dispatch<SetStateAction<TokenAmount>>,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solscan' | 'solanaExplorer' | 'solanaBeach',
  pools: JetMarginPools,
  amount: TokenAmount,
) => {
  const token = markets[selectedMarket].token;
  if (!marginAccount || !token) return;
  let tx = 'failed_before_tx';
  try {
    tx = await settle({
      markets,
      selectedMarket,
      marginAccount,
      provider,
      pools: pools.tokenPools,
      amount: amount.lamports
    });
    notify(
      'Settle Successful',
      `Your assets have been sent to your margin account`,
      'success',
      getExplorerUrl(tx, cluster, blockExplorer)
    );
    setOwedTokens(new TokenAmount(new BN(0), token.decimals));
  } catch (e: any) {
    notify(
      'Settle Failed',
      `There was an issue settling your funds, please try again.`,
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
  }
};

const submitRepay = async (
  marginAccount: MarginAccount,
  provider: AnchorProvider,
  amount: BN,
  termLoans: Loan[],
  walletAddress: PublicKey | null,
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
  market: MarketAndconfig,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solscan' | 'solanaExplorer' | 'solanaBeach',
) => {
  if (!walletAddress) return;
  let tx = 'failed_before_tx';
  try {
    tx = await repay({
      provider,
      marginAccount,
      amount,
      termLoans,
      pools,
      markets,
      market
    });
    notify(
      'Repay Successful',
      `Your debt has been successfully repaid`,
      'success',
      getExplorerUrl(tx, cluster, blockExplorer)
    );
  } catch (e: any) {
    notify(
      'Repay Failed',
      `There was an issue repaying your debt, please try again.`,
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
    throw e;
  }
};

const getOwedTokens = async (mint: Address, marginAccount: PublicKey, provider: AnchorProvider, setOwedTokens: Dispatch<SetStateAction<TokenAmount>>) => {
  const pda = AssociatedToken.derive(mint, marginAccount);
      try {
        const exists = await provider.connection.getAccountInfo(pda)
        if (exists) {
          const { value } = await provider.connection.getTokenAccountBalance(pda)
          setOwedTokens(new TokenAmount(new BN(value.amount), value.decimals));
        }
      } catch (e) {
        console.log(e)
      }
}

export const FixedTermMarketSelector = ({ type }: FixedTermMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedTermMarketAtom);
  const pools = useRecoilValue(Pools);
  const wallet = useWallet();

  const [repayAmount, setRepayAmount] = useState('0');

  const [owedTokens, setOwedTokens] = useState<TokenAmount>(
    new TokenAmount(new BN(0), markets[selectedMarket]?.token.decimals || 6)
  );

  useEffect(() => {
    if (marginAccount?.address && markets[selectedMarket].token) {
      getOwedTokens(markets[selectedMarket].token.mint, marginAccount.address, provider, setOwedTokens);
    }
  }, [marginAccount?.address]);

  const { data } = useOpenPositions(markets[selectedMarket]?.market, marginAccount);

  const token = markets[selectedMarket]?.token;

  if (!marginAccount || !pools || !markets[selectedMarket] || !data || !token) return null;

  const hasToSettle = owedTokens?.tokens > 0;
  const hasToRepay = data.total_borrowed > 0;

  return (
    <div className="fixed-term-selector-view view-element">
      <div className="fixed-term-selector-view-container">
        <Select
          value={selectedMarket + 1}
          showSearch={true}
          suffixIcon={<AngleDown className="jet-icon" />}
          onChange={value => setSelectedMarket(value - 1)}>
          {markets.map((market, index) => (
            <Option key={market.name} value={index + 1}>
              {marketToString(market.config)}
            </Option>
          ))}
        </Select>
        <div className="selector-actions">
          {hasToSettle ? (
            <div className="assets-to-settle">
              <>
                There are {owedTokens?.uiTokens} {markets[selectedMarket]?.token.symbol} currently pending settment on
                this market.
              </>
              <Button
                onClick={() =>
                  settleNow(
                    marginAccount,
                    markets,
                    selectedMarket,
                    provider,
                    setOwedTokens,
                    cluster,
                    blockExplorer,
                    pools,
                    owedTokens,
                  )
                }>
                Settle Now
              </Button>
            </div>
          ) : hasToRepay ? (
            <div className="assets-to-settle">
              <>
                You owe {new TokenAmount(new BN(data.total_borrowed), token.decimals).tokens} {token.symbol} on this
                market.
              </>
              <input
                value={repayAmount}
                onChange={e => {
                  const parsed = parseFloat(e.target.value);
                  if (isNaN(parsed)) {
                    setRepayAmount('0');
                  } else {
                    const total = new TokenAmount(new BN(data.total_borrowed), token.decimals);
                    const amount = parsed <= total.tokens ? e.target.value : total.uiTokens.replace(',', '');
                    setRepayAmount(amount);
                  }
                }}
              />
              <Button
                onClick={() =>
                  submitRepay(
                    marginAccount,
                    provider,
                    new BN(parseFloat(repayAmount) * 10 ** token.decimals),
                    data.loans,
                    wallet.publicKey,
                    pools.tokenPools,
                    markets.map(m => m.market),
                    markets[selectedMarket],
                    cluster,
                    blockExplorer,
                  )
                }>
                Repay Now
              </Button>
            </div>
          ) : (
            <div>There are no outstanding actions on this market.</div>
          )}
        </div>
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
