import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useWallet } from '@solana/wallet-adapter-react';
import { Dictionary } from '@state/settings/localization/localization';
import { BlockExplorer } from '@state/settings/settings';
import { WalletModal } from '@state/modals/modals';
import { SendingTransaction } from '@state/actions/actions';
import { CurrentPool } from '@state/pools/pools';
import { ActionResponse, useMarginActions } from '@utils/jet/marginActions';
import { getExplorerUrl } from '@utils/ui';
import { notify } from '@utils/notify';
import { Button } from 'antd';
import { CloudFilled } from '@ant-design/icons';
import { useJetStore } from '@jet-lab/store';

// Button for airdropping a token to the user's Solana wallet (if on devnet)
export function AirdropButton(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const blockExplorer = useRecoilValue(BlockExplorer);
  const cluster = useJetStore(state => state.settings.cluster);
  const { connected } = useWallet();
  const currentPool = useRecoilValue(CurrentPool);
  const { airdrop } = useMarginActions();
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const [sendingTransaction, setSendingTransaction] = useRecoilState(SendingTransaction);

  // Airdrop token to user's Solana wallet
  async function doAirdrop() {
    if (!currentPool) {
      return;
    }

    setSendingTransaction(true);
    const [amount, txId, resp] = await airdrop(currentPool);
    if (resp === ActionResponse.Success) {
      notify(
        dictionary.notifications.actions.successTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.successDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', amount)
          .replace('{{ASSET}}', currentPool.symbol),
        'success',
        txId ? getExplorerUrl(txId, cluster, blockExplorer) : undefined
      );
    } else if (resp === ActionResponse.Cancelled) {
      notify(
        dictionary.notifications.actions.cancelledTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.cancelledDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', amount)
          .replace('{{ASSET}}', currentPool.symbol),
        'warning'
      );
    } else {
      notify(
        dictionary.notifications.actions.failedTitle.replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop),
        dictionary.notifications.actions.failedDescription
          .replace('{{ACTION}}', dictionary.poolsView.poolDetail.airdrop)
          .replace('{{AMOUNT}}', amount)
          .replace('{{ASSET}}', currentPool.symbol),
        'error'
      );
    }
    setSendingTransaction(false);
  }

  if (cluster === 'mainnet-beta') {
    return <></>;
  }

  return (
    <Button
      type="dashed"
      style={{ marginLeft: 20 }}
      onClick={() => (connected ? doAirdrop() : setWalletModalOpen(true))}
      disabled={!currentPool || sendingTransaction}
      loading={sendingTransaction}
      icon={<CloudFilled />}>
      {dictionary.poolsView.poolDetail.airdrop}
    </Button>
  );
}
