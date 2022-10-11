import { useEffect } from 'react';
import axios from 'axios';
import { atom, useRecoilValue, useSetRecoilState } from 'recoil';
import { MarginClient, MarginConfig as JetMarginConfig } from '@jet-lab/margin';
import { Cluster } from '../settings/settings';

// Pool config instantiation at app init
export const MarginConfig = atom({
  key: 'marginConfig',
  default: undefined as JetMarginConfig | undefined
});

// A syncer to be called so that we can have dependent atom state
export function useMarginConfigSyncer() {
  const cluster = useRecoilValue(Cluster);
  const setMarginConfig = useSetRecoilState(MarginConfig);

  async function getLocalnetConfig() {
    let response = await axios.get('/localnet.config.json');
    return await response.data;
  }

  useEffect(() => {
    if (cluster == 'localnet') {
      getLocalnetConfig().then(config => setMarginConfig(config));
    } else {
      MarginClient.getConfig(cluster).then(config => setMarginConfig(config));
    }
  }, [cluster, setMarginConfig]);
}
