import { useWallet } from '@solana/wallet-adapter-react';
import { Pool } from '@jet-lab/margin';
import { useSetRecoilState, useRecoilValue, useRecoilState } from 'recoil';
import { WalletModal } from '../../../state/modals/modals';
import { WalletInit } from '../../../state/user/walletTokens';
import { CurrentAction } from '../../../state/actions/actions';
import { CurrentMarketPair } from '../../../state/trade/market';
import { formatRate } from '../../../utils/format';
import { TokenLogo } from '../../misc/TokenLogo';
import { ReactComponent as BorrowIcon } from '../../../styles/icons/function-borrow.svg';
import { useCurrencyFormatting } from '../../../utils/currency';
import { FiatValues } from '../../../state/settings/settings';
import { Dictionary } from '../../../state/settings/localization/localization';
import { CurrentPoolSymbol } from '../../../state/borrow/pools';
import { Button, Skeleton, Typography } from 'antd';

export const PoolRow = (props: { pool: Pool }) => {
  const { pool } = props;
  const [currentPoolSymbol, setCurrentPoolSymbol] = useRecoilState(CurrentPoolSymbol);
  const dictionary = useRecoilValue(Dictionary);
  const setWalletModalOpen = useSetRecoilState(WalletModal);
  const setCurrentMarketPair = useSetRecoilState(CurrentMarketPair);
  const fiatValues = useRecoilValue(FiatValues);
  const { connected } = useWallet();
  const walletInit = useRecoilValue(WalletInit);
  const setCurrentAction = useSetRecoilState(CurrentAction);
  const { currencyFormatter, currencyAbbrev } = useCurrencyFormatting();
  const { Paragraph, Text } = Typography;

  return (
    <tr
      className={`ant-table-row ant-table-row-level-0 ${pool.symbol}-pools-table-row ${
        pool.symbol === currentPoolSymbol ? 'active' : ''
      }`}
      onClick={() => setCurrentPoolSymbol(pool.symbol)}
      key={pool.symbol}>
      <td className="ant-table-cell" style={{ textAlign: 'left' }}>
        {
          <div className="table-token">
            <TokenLogo height={22} symbol={pool?.symbol} />
            {pool?.name ? (
              <>
                <Text strong>{pool.name}</Text>
                <Text>{`${pool.symbol} ≈ ${currencyFormatter(pool.tokenPrice ?? 0, true)}`}</Text>
              </>
            ) : (
              ''
            )}
          </div>
        }
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        {pool.depositNoteMetadata?.valueModifier ? (
          formatRate(pool.depositNoteMetadata.valueModifier.toNumber())
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )}
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        {!isNaN(pool.utilizationRate) ? (
          formatRate(pool.utilizationRate)
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )}
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        {pool.borrowedTokens ? (
          `${currencyAbbrev(pool.borrowedTokens.tokens, fiatValues, pool.tokenPrice, pool.decimals / 2)} ${
            !fiatValues ? pool.symbol : ''
          }`
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )}
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        {pool.vault ? (
          `${currencyAbbrev(pool.vault.tokens, fiatValues, pool.tokenPrice, pool.decimals / 2)} ${
            !fiatValues ? pool.symbol : ''
          }`
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )}
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        {!isNaN(pool.depositApy) ? (
          <div className="flex align-center justify-end">
            <Paragraph type="success">{formatRate(pool.depositApy)}</Paragraph>&nbsp;/&nbsp;
            <Paragraph type="danger">{formatRate(pool.borrowApr)}</Paragraph>
          </div>
        ) : (
          <Skeleton className="align-right" paragraph={false} active />
        )}
      </td>
      <td className="ant-table-cell" style={{ textAlign: 'right' }}>
        <Button
          size="small"
          className="function-btn"
          disabled={!pool?.symbol}
          onClick={() => {
            if (!connected || !walletInit) {
              setWalletModalOpen(true);
              return;
            }

            setCurrentAction('borrow');
            setCurrentPoolSymbol(pool.symbol);
            if (pool.symbol !== 'USDC') {
              setCurrentMarketPair(`${pool.symbol}/USDC`);
            }
          }}>
          <BorrowIcon className="jet-icon" />
          {dictionary.actions.borrow.title}
        </Button>
      </td>
    </tr>
  );
};
