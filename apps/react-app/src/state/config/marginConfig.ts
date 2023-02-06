import { useEffect } from 'react';
import axios from 'axios';
import { atom, useSetRecoilState } from 'recoil';
import { MarginClient, MarginConfig } from '@jet-lab/margin';
import { useJetStore } from '@jet-lab/store';

// Pool config instantiation at app init
export const MainConfig = atom<MarginConfig | undefined>({
  key: 'mainConfig',
  default: undefined
});

// A syncer to be called so that we can have dependent atom state
export function useMainConfigSyncer() {
  const cluster = useJetStore(state => state.settings.cluster);
  const setMainConfig = useSetRecoilState(MainConfig);

  async function getLocalnetConfig() {
    let response = await axios.get('/localnet.config.legacy.json');
    return await response.data;
  }

  useEffect(() => {
    if (cluster == 'localnet') {
      getLocalnetConfig().then(config => {
        setMainConfig(config);
      });
    } else {
      MarginClient.getConfig(cluster).then(config => setMainConfig(config));
    }
  }, [cluster, setMainConfig]);
}
