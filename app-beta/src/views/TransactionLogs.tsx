import { useWallet } from '@solana/wallet-adapter-react';
import { useLanguage } from '../contexts/localization/localization';
import { useConnectWalletModal } from '../contexts/connectWalletModal';
import { useTransactionLogs } from '../contexts/transactionLogs';
import { useBlockExplorer } from '../contexts/blockExplorer';
import { shortenPubkey } from '../utils/utils';
import { ReactComponent as ArrowIcon } from '../styles/icons/arrow_icon.svg';
import { Button, Divider } from 'antd';
import { totalAbbrev } from '../utils/currency';
import { useEffect } from 'react';

export function TransactionLogs(): JSX.Element {
  const { dictionary } = useLanguage();
  const { connected, publicKey } = useWallet();
  const { setConnecting } = useConnectWalletModal();
  const { getExplorerUrl } = useBlockExplorer();
  const { logs, refreshLogs } = useTransactionLogs();

  useEffect(() => {
    refreshLogs();
  }, []);

  return (
    <div className="transaction-logs view">
      <div className="table-container">
        <div className="flex align-center justify-start">
          <h1>{dictionary.transactions.title}</h1>
          {connected && <span>{shortenPubkey(publicKey?.toString() ?? '')}</span>}
        </div>
        <Divider />
        <table>
          <thead>
            <tr>
              <th>{dictionary.transactions.date}</th>
              <th>{dictionary.transactions.time}</th>
              <th>{dictionary.transactions.signature}</th>
              <th>{dictionary.transactions.tradeAction}</th>
              <th>{dictionary.transactions.tradeAmount}</th>
            </tr>
          </thead>
          <tbody>
            {logs.map((log, i) => (
              <tr key={i} onClick={() => window.open(getExplorerUrl(log.signature), '_blank')}>
                <td>{log.blockDate}</td>
                <td>{log.blockTime}</td>
                <td style={{ color: 'var(--success)' }}>{shortenPubkey(log.signature, 4)}</td>
                <td className="reserve-detail">{log.tradeAction}</td>
                <td className="asset">
                  {totalAbbrev(Math.abs(log.tradeAmount.tokens), undefined, true, log.tokenDecimals)}
                  &nbsp;
                  {log.tokenSymbol}
                </td>
                <td>
                  <ArrowIcon className="jet-icon" />
                </td>
              </tr>
            ))}
            <tr className="no-interaction">
              <td></td>
              <td></td>
              <td></td>
              <td></td>
              <td></td>
            </tr>
            {!connected && (
              <tr className="no-interaction">
                <td>
                  <Button type="dashed" onClick={() => setConnecting(true)}>
                    {dictionary.settings.connect + ' ' + dictionary.settings.wallet}
                  </Button>
                </td>
                <td></td>
                <td></td>
                <td></td>
                <td></td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
