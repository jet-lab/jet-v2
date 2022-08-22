import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { Connection } from '@solana/web3.js';
import { Dictionary } from '../../state/settings/localization/localization';
import { Alert } from 'antd';

export function TpsBanner(): JSX.Element {
  const dictionary = useRecoilValue(Dictionary);
  const [tps, setTps] = useState<number | undefined>(undefined);

  useEffect(() => {
    async function getSolanaTps() {
      try {
        const connection = new Connection('https://api.mainnet-beta.solana.com/');
        const samples = await connection.getRecentPerformanceSamples(15);
        const totalTps = samples.reduce((acc, val) => {
          return acc + val.numTransactions / val.samplePeriodSecs;
        }, 0);
        const aveTps = Math.round(totalTps / samples.length);
        setTps(aveTps);
      } catch {
        return;
      }
    }

    getSolanaTps();
    const tpsInterval = setInterval(getSolanaTps, 30000);
    return () => clearInterval(tpsInterval);
  }, []);

  return (
    <>
      {tps && tps < 1200 && (
        <Alert
          closable
          className="tps-banner"
          type={tps > 800 ? 'warning' : 'error'}
          message={
            tps > 800
              ? dictionary.notifications.tpsDegraded.replaceAll('{{TPS}}', tps.toString())
              : dictionary.notifications.tpsSevere.replaceAll('{{TPS}}', tps.toString())
          }
        />
      )}
    </>
  );
}
