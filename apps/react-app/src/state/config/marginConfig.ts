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
  const { cluster, updateAirspaceLookupTables } = useJetStore(state => ({
    cluster: state.settings.cluster,
    updateAirspaceLookupTables: state.updateAirspaceLookupTables
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
        const airspaces = (await axios.get('/localnet.config.json')).data.airspaces;
        const addresses = await getAuthorityLookupTables(airspaces[0].lookupRegistryAuthority)
        updateAirspaceLookupTables(addresses);
      });
    } else {
      const configs = Promise.all([MarginClient.getConfig(cluster), MarginClient.getLegacyConfig(cluster)]);
      configs.then(async ([config, legacyConfig]) => {
        // Merge airspaces info from new to legacy format
        legacyConfig.airspaces = config.airspaces;
        setMainConfig(legacyConfig);
        const addresses = await getAuthorityLookupTables(legacyConfig.airspaces[0].lookupRegistryAuthority);
        updateAirspaceLookupTables(addresses);
      })
    }
  }, [cluster, setMainConfig]);
}
