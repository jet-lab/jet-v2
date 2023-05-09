import { useEffect } from 'react';
import axios from 'axios';
import { atom, useSetRecoilState } from 'recoil';
import { MarginClient, MarginConfig } from '@jet-lab/margin';
import { useJetStore } from '@jet-lab/store';
import { getAuthorityLookupTables } from '@utils/lookupTables';

// Pool config instantiation at app init
export const MainConfig = atom<MarginConfig | undefined>({
  key: 'mainConfig',
  default: undefined
});

// A syncer to be called so that we can have dependent atom state
export function useMainConfigSyncer() {
  const { cluster, updateLookupTables } = useJetStore(state => ({
    cluster: state.settings.cluster,
    updateLookupTables: state.updateLookupTables
  }));
  const setMainConfig = useSetRecoilState(MainConfig);

  async function getLocalnetConfig() {
    let response = await axios.get('/localnet.config.legacy.json');
    return await response.data;
  }

  useEffect(() => {
    if (cluster == 'localnet') {
      getLocalnetConfig().then(async config => {
        setMainConfig(config);
        // This is temporary until we use the new config format
        let airspaces = (await axios.get('/localnet.config.json')).data.airspaces;
        return getAuthorityLookupTables(airspaces[0].lookupRegistryAuthority)
      }).then(addresses => {
        updateLookupTables(addresses);
      });
    } else {
      MarginClient.getConfig(cluster).then(config => setMainConfig(config));
      // TODO: update authority here too
    }
  }, [cluster, setMainConfig]);
}
