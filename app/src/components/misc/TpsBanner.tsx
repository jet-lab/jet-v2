import { useEffect, useState } from 'react';
import { Cluster, PreferredRpcNode, rpcNodes } from '@state/settings/settings';
import { Dictionary } from '@state/settings/localization/localization';
import { useProvider } from '@utils/jet/provider';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Alert } from 'antd';
import { NetworkStateAtom } from '@state/network/network-state';

// Banner to show user that the Solana network is running slowly
export function TpsBanner(): JSX.Element {
  const cluster = useRecoilValue(Cluster);
  const { provider } = useProvider();
  const rpcNode = useRecoilValue(PreferredRpcNode);
  const dictionary = useRecoilValue(Dictionary);
  const nodeIndexer = cluster === 'mainnet-beta' ? 'mainnetBeta' : 'devnet';
  const ping = rpcNodes[rpcNode][`${nodeIndexer}Ping`];
  const [tps, setTps] = useState<number | undefined>(undefined);
  const unusuallySlow = (tps && tps < 1500) || ping > 750;
  const criticallySlow = (tps && tps < 1000) || ping > 1500;
  const [networkStatus, setNetworkStatus] = useRecoilState(NetworkStateAtom);

  // Returns the conditional TPS warning message
  function getTpsMessage() {
    let message = dictionary.notifications.tpsDegraded;
    if (criticallySlow) {
      message = dictionary.notifications.tpsSevere;
    }

    // Add dynamic values
    message = message.replaceAll('{{TPS}}', tps?.toString() ?? '').replaceAll('{{PING}}', ping.toString() + 'ms');
    return message;
  }

  // On mount, initiate an interval of checking Solana TPS
  useEffect(() => {
    async function getSolanaTps() {
      try {
        // Get performance samples
        const samples = await provider.connection.getRecentPerformanceSamples(15);
        if (networkStatus) setNetworkStatus('connected');
        // Reduce to the total transactions-per-second
        const totalTps = samples.reduce((acc, val) => {
          return acc + val.numTransactions / val.samplePeriodSecs;
        }, 0);
        // Calculate the average tps from the amount of samples
        const aveTps = Math.round(totalTps / samples.length);
        setTps(aveTps);
      } catch {
        setNetworkStatus('error');
        return;
      }
    }

    getSolanaTps();
    // Check TPS every 30 seconds
    const tpsInterval = setInterval(getSolanaTps, 60_000);
    return () => clearInterval(tpsInterval);
  }, [provider.connection]);

  // Render the TPS banner (if TPS is slow enough)
  if (unusuallySlow) {
    return (
      <Alert closable className="tps-banner" type={criticallySlow ? 'error' : 'warning'} message={getTpsMessage()} />
    );
  } else {
    return <></>;
  }
}
