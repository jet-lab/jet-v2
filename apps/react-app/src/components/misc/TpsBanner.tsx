import { useEffect, useState } from 'react';
import { Dictionary } from '@state/settings/localization/localization';
import { useProvider } from '@utils/jet/provider';
import { useRecoilState, useRecoilValue } from 'recoil';
import { Alert } from 'antd';
import { NetworkStateAtom } from '@state/network/network-state';
import { useJetStore } from '@jet-lab/store';

// Banner to show user that the Solana network is running slowly
export function TpsBanner(): JSX.Element {
  const { cluster, rpc } = useJetStore(state => ({ cluster: state.settings.cluster, rpc: state.settings.rpc }));
  const { provider } = useProvider();
  const dictionary = useRecoilValue(Dictionary);
  const ping = rpc.pings[cluster];
  const [tps, setTps] = useState<number | undefined>(undefined);
  const unusuallySlow = (tps && tps < 1500) || ping > 750;
  const criticallySlow = (tps && tps < 1000) || ping > 1500;
  const [networkStatus, setNetworkStatus] = useRecoilState(NetworkStateAtom);
  const isMainnet = cluster === 'mainnet-beta';

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
  }, [provider.connection]);

  // Render the TPS banner (if TPS is slow enough)
  if (isMainnet && unusuallySlow) {
    return (
      <Alert closable className="tps-banner" type={criticallySlow ? 'error' : 'warning'} message={getTpsMessage()} />
    );
  } else {
    return <></>;
  }
}
