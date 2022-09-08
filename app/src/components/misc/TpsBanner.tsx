import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { Connection } from '@solana/web3.js';
import { Dictionary } from '../../state/settings/localization/localization';
import { MS_PER_MINUTE } from '../../utils/time';
import { Alert } from 'antd';

// Banner to show user that the Solana network is running slowly
export function TpsBanner(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [tps, setTps] = useState<number | undefined>(undefined);
  const unusuallySlow = tps && tps < 1200;
  const criticallySlow = tps && tps < 800;

  // On mount, initiate an interval of checking Solana TPS
  useEffect(() => {
    async function getSolanaTps() {
      try {
        // Create a new connection at main solana endpoint
        const connection = new Connection('https://api.mainnet-beta.solana.com/');
        // Get performance samples
        const samples = await connection.getRecentPerformanceSamples(15);
        // Reduce to the total transactions-per-second
        const totalTps = samples.reduce((acc, val) => {
          return acc + val.numTransactions / val.samplePeriodSecs;
        }, 0);
        // Calculate the average tps from the amount of samples
        const aveTps = Math.round(totalTps / samples.length);
        setTps(aveTps);
      } catch {
        return;
      }
    }

    getSolanaTps();
    // Check TPS every 30 seconds
    const tpsInterval = setInterval(getSolanaTps, MS_PER_MINUTE / 2);
    return () => clearInterval(tpsInterval);
  }, []);

  // Render the TPS banner (if TPS is slow enough)
  if (unusuallySlow) {
    return (
      <Alert
        closable
        className="tps-banner"
        type={criticallySlow ? 'error' : 'warning'}
        message={
          criticallySlow
            ? dictionary.notifications.tpsSevere.replaceAll('{{TPS}}', tps.toString())
            : dictionary.notifications.tpsDegraded.replaceAll('{{TPS}}', tps.toString())
        }
      />
    );
  } else {
    return <></>;
  }
}
