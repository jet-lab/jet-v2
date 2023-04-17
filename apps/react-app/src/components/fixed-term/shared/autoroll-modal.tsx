import { LoadingOutlined } from '@ant-design/icons';
import { MarginAccount, MarketAndConfig, configAutoroll, rate_to_price } from '@jet-lab/margin';
import { useJetStore } from '@jet-lab/store';
import { Modal } from '@jet-lab/ui';
import { useWallet } from '@solana/wallet-adapter-react';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';
import { useProvider } from '@utils/jet/provider';
import { notify } from '@utils/notify';
import { getExplorerUrl } from '@utils/ui';
import { Button, InputNumber } from 'antd';
import { useCallback, useEffect, useMemo, useState } from 'react';

interface AutorollModalProps {
  marketAndConfig: MarketAndConfig;
  marginAccount?: MarginAccount;
  borrowRate?: number;
  lendRate?: number;
  open: boolean;
  onClose: () => void;
  refresh: () => void;
}
export const AutoRollModal = ({
  marketAndConfig,
  marginAccount,
  open,
  borrowRate,
  lendRate,
  onClose,
  refresh
}: AutorollModalProps) => {
  const [minLendRate, setMinLendRate] = useState<number>();
  const [maxBorrowRate, setMaxBorrowRate] = useState<number>();
  const [pending, setPending] = useState(false);
  const { publicKey } = useWallet();
  const { provider } = useProvider();

  const { cluster, explorer } = useJetStore(state => state.settings);

  const marketName = useMemo(
    () => friendlyMarketName(marketAndConfig.config.symbol, marketAndConfig.config.borrowTenor).toUpperCase(),
    [marketAndConfig]
  );

  const preprocessInput = useCallback((e: number) => Math.round(e * 100) / 100, []);

  const processPayload = useCallback(() => {
    if (!minLendRate || minLendRate === 0 || !maxBorrowRate || maxBorrowRate === 0) return;

    const lendBps = Math.round(minLendRate * 100);
    const borrowBps = Math.round(maxBorrowRate * 100);

    const lendPrice = rate_to_price(BigInt(lendBps), BigInt(marketAndConfig.config.borrowTenor));
    const borrowPrice = rate_to_price(BigInt(borrowBps), BigInt(marketAndConfig.config.borrowTenor));
    return { lendPrice, borrowPrice };
  }, [minLendRate, maxBorrowRate]);

  const submitConfig = async () => {
    setPending(true);
    const payload = processPayload();
    if (!marginAccount || !publicKey || !payload) return;
    try {
      let signature = await configAutoroll({
        account: marginAccount,
        marketAndConfig,
        walletAddress: publicKey,
        provider,
        payload
      });
      console.log('Processed ', signature);
      notify(
        'Autoroll Configured',
        `You successfully configured autoroll for the ${marketName} market`,
        'success',
        getExplorerUrl(signature, cluster, explorer)
      );
      setPending(false);
    } catch (e: any) {
      notify(
        'Market Configuration Failed',
        `Please contact support`,
        'error',
        getExplorerUrl(e.signature, cluster, explorer)
      );
      setPending(false);
      console.error(e);
    } finally {
      refresh();
      onClose();
    }
  };

  useEffect(() => {
    if (lendRate) {
      setMinLendRate(lendRate / 100);
    } else {
      setMinLendRate(undefined);
    }
  }, [lendRate]);

  useEffect(() => {
    if (borrowRate) {
      setMaxBorrowRate(borrowRate / 100);
    } else {
      setMaxBorrowRate(undefined);
    }
  }, [borrowRate]);

  return (
    <Modal open={open} onClose={onClose} title={`Configure ${marketName} market`}>
      <div className="flex flex-col autoroll-modal">
        <div className="flex mx-2 my-4">
          <label>
            Minimum lend rate
            <InputNumber
              className="input-rate"
              value={minLendRate && minLendRate > 0 ? minLendRate : undefined}
              onChange={e => setMinLendRate(preprocessInput(e || 0))}
              formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
              placeholder={'3.50'}
              controls={false}
              addonAfter="%"
            />
          </label>
          <label>
            Maximum borrow rate
            <InputNumber
              className="input-rate"
              value={maxBorrowRate && maxBorrowRate > 0 ? maxBorrowRate : undefined}
              onChange={e => setMaxBorrowRate(preprocessInput(e || 0))}
              formatter={value => `${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')}
              placeholder={'6.50'}
              controls={false}
              addonAfter="%"
            />
          </label>
        </div>
        <Button disabled={minLendRate === 0 || maxBorrowRate === 0} onClick={submitConfig}>
          {pending ? (
            <>
              <LoadingOutlined />
              Sending transaction
            </>
          ) : (
            'Save'
          )}
        </Button>
      </div>
    </Modal>
  );
};
