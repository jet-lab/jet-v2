import { BondMarket, settle } from '@jet-lab/jet-bonds-client';
import { MarginAccount } from '@jet-lab/margin';
import { AllFixedMarketsAtom, MarketAndconfig } from '@state/fixed-market/fixed-term-market-sync';
import { BlockExplorer, Cluster } from '@state/settings/settings';
import { CurrentAccount } from '@state/user/accounts';
import { useProvider } from '@utils/jet/provider';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { Button } from 'antd';
import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';

interface Owed {
  tickets: number;
  tokens: number;
}
const defaultAssets: Owed = { tickets: 0, tokens: 0 };

const fetchAssets = async (market: BondMarket, marginAccount: MarginAccount): Promise<Owed> => {
  const user = await market.fetchMarginUser(marginAccount);
  const assets = { ...defaultAssets };
  if (user?.assets) {
    assets.tokens = user.assets.entitledTokens?.toNumber();
    assets.tickets = user.assets.entitledTickets?.toNumber();
  }
  return assets;
};

const fetchTotalOwed = async (markets: MarketAndconfig[], marginAccount: MarginAccount): Promise<Owed> => {
  const assets = await Promise.all(markets.map(market => fetchAssets(market.market, marginAccount)));
  const total = assets.reduce(
    (all, market) => {
      all.tokens += market.tokens;
      all.tickets += market.tickets;
      return all;
    },
    { ...defaultAssets }
  );
  return total;
};

export const Settle = () => {
  const markets = useRecoilValue(AllFixedMarketsAtom);
  const marginAccount = useRecoilValue(CurrentAccount);
  const [pendingAssets, setPendingAssets] = useState({ ...defaultAssets });
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();

  useEffect(() => {
    if (marginAccount) {
      fetchTotalOwed(markets, marginAccount).then(owed => setPendingAssets(owed));
    }
  }, [markets.length, marginAccount]);

  const settleNow = async () => {
    let tx: string;
    try {
      tx = await settle({ markets: markets.map(m => m.market), marginAccount, provider });
      notify(
        'Settle Successful',
        `Your assets have been sent to your margin account`,
        'success',
        getExplorerUrl(tx, cluster, blockExplorer)
      );
      setPendingAssets(defaultAssets);
    } catch {
      notify(
        'Settle Failed',
        `There was an issue settling your funds, please try again.`,
        'success',
        getExplorerUrl(tx, cluster, blockExplorer)
      );
    }
  };

  if (pendingAssets.tickets === 0 && pendingAssets.tokens === 0) {
    return null;
  }

  return (
    <div className="view-element assets-to-settle">
      <div>
        <span>
          You are entitled to {pendingAssets.tickets} tickets and {pendingAssets.tokens} tokens currently pending
          settment.
        </span>
        <Button onClick={settleNow}>Settle Now</Button>
      </div>
    </div>
  );
};
