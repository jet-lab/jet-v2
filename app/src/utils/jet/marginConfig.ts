import { MarginClient, MarginConfig } from '@jet-lab/margin';
import { useEffect, useState } from 'react';
import { useRecoilValue } from 'recoil';
import { Cluster } from '../../state/settings/settings';

export function useMarginConfig() {
  const cluster = useRecoilValue(Cluster);
  const [config, setConfig] = useState<MarginConfig | undefined>(undefined);

  useEffect(() => {
    const getConfig = async () => {
      setConfig(await MarginClient.getConfig(cluster));
    };

    getConfig();
  }, [cluster]);

  return config;
}
