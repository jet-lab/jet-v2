import { useRecoilState, useRecoilValue } from 'recoil';
('../misc/ReorderArrows');
import {
  AllFixedTermMarketsAtom,
  MarketAndconfig,
  SelectedFixedTermMarketAtom
} from '@state/fixed-term/fixed-term-market-sync';
import { FixedBorrowViewOrder, FixedLendViewOrder } from '@state/views/fixed-term';
import { ReorderArrows } from '@components/misc/ReorderArrows';
import { Button, Select } from 'antd';
import AngleDown from '@assets/icons/arrow-angle-down.svg';
import { marketToString } from '@utils/jet/fixed-term-utils';
import { CurrentAccount } from '@state/user/accounts';
import { Dispatch, SetStateAction, useEffect, useState } from 'react';
import { MarginAccount, TokenAmount } from '@jet-lab/margin';
import { getExplorerUrl } from '@utils/ui';
import { notify } from '@utils/notify';
import { AnchorProvider } from '@project-serum/anchor';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { settle } from '@jet-lab/fixed-term';
import { useProvider } from '@utils/jet/provider';
import { JetMarginPools, Pools } from '@state/pools/pools';

const { Option } = Select;

interface FixedTermMarketSelectorProps {
  type: 'asks' | 'bids';
}

const fetchOwedTokens = async (market: MarketAndconfig, marginAccount: MarginAccount): Promise<number> => {
  const user = await market.market.fetchMarginUser(marginAccount);
  const owedTokens = user?.assets.entitledTokens;
  return owedTokens ? new TokenAmount(owedTokens, market.token.decimals).tokens : 0;
};

const settleNow = async (
  marginAccount: MarginAccount,
  markets: MarketAndconfig[],
  selectedMarket: number,
  provider: AnchorProvider,
  setOwedTokens: Dispatch<SetStateAction<number>>,
  cluster: 'mainnet-beta' | 'localnet' | 'devnet',
  blockExplorer: 'solscan' | 'solanaExplorer' | 'solanaBeach',
  pools: JetMarginPools
) => {
  if (!marginAccount) return;
  let tx = 'failed_before_tx';
  try {
    tx = await settle({
      markets: markets.map(m => m.market),
      selectedMarket,
      marginAccount,
      provider,
      pools: pools.tokenPools
    });
    notify(
      'Settle Successful',
      `Your assets have been sent to your margin account`,
      'success',
      getExplorerUrl(tx, cluster, blockExplorer)
    );
    setOwedTokens(0);
  } catch (e: any) {
    notify(
      'Settle Failed',
      `There was an issue settling your funds, please try again.`,
      'error',
      getExplorerUrl(e.signature, cluster, blockExplorer)
    );
  }
};

export const FixedTermMarketSelector = ({ type }: FixedTermMarketSelectorProps) => {
  const [order, setOrder] = useRecoilState(type === 'asks' ? FixedLendViewOrder : FixedBorrowViewOrder);
  const markets = useRecoilValue(AllFixedTermMarketsAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();
  const [selectedMarket, setSelectedMarket] = useRecoilState(SelectedFixedTermMarketAtom);
  const [owedTokens, setOwedTokens] = useState(0);
  const pools = useRecoilValue(Pools);

  useEffect(() => {
    if (marginAccount) {
      fetchOwedTokens(markets[selectedMarket], marginAccount).then(owed => setOwedTokens(owed));
    }
  }, [selectedMarket, marginAccount]);

  if (!marginAccount || !pools) return;

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
          {owedTokens > 0 && (
            <div className="assets-to-settle">
              <span>
                There are {owedTokens} {markets[selectedMarket].token.symbol} currently pending settment on this market.
              </span>
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
                    pools
                  )
                }>
                Settle Now
              </Button>
            </div>
          )}
        </div>
      </div>

      <ReorderArrows component="marketSelector" order={order} setOrder={setOrder} vertical />
    </div>
  );
};
