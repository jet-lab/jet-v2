import { MarketAndConfig } from '@jet-lab/margin';
import { Modal } from '@jet-lab/ui';
import { friendlyMarketName } from '@utils/jet/fixed-term-utils';
import { Button, InputNumber } from 'antd';
import { useState } from 'react';

interface AutorollModalProps {
  marketAndConfig: MarketAndConfig;
  open: boolean;
  onClose: () => void;
}
export const AutoRollModal = ({ marketAndConfig, open, onClose }: AutorollModalProps) => {
  const [minLendRate, setMinLendRate] = useState();
  const [maxBorrowRate, setMaxBorrowRate] = useState();
  return (
    <Modal
      open={open}
      onClose={onClose}
      title={`Configure ${friendlyMarketName(
        marketAndConfig.config.symbol,
        marketAndConfig.config.borrowTenor
      ).toUpperCase()} market`}>
      <div className="flex flex-col autoroll-modal">
        <div className="flex mx-2 my-4">
          <label>
            Minimum lend rate
            <InputNumber
              className="input-rate"
              value={minLendRate}
              placeholder={'6.50'}
              type="number"
              step={0.01}
              min={0}
              controls={false}
              addonAfter="%"
            />
          </label>
          <label>
            Maximum borrow rate
            <InputNumber
              className="input-rate"
              value={maxBorrowRate}
              placeholder={'6.50'}
              type="number"
              step={0.01}
              min={0}
              controls={false}
              addonAfter="%"
            />
          </label>
        </div>
        <Button>Save</Button>
      </div>
    </Modal>
  );
};
