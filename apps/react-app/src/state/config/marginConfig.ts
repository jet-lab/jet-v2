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
  const { cluster, updateLookupTableAddresses } = useJetStore(state => ({
    cluster: state.settings.cluster,
    updateLookupTableAddresses: state.updateLookupTableAddresses
  }));
  const setMainConfig = useSetRecoilState(MainConfig);

  async function getLocalnetConfig() {
    let response = await axios.get('/localnet.config.legacy.json');
    return await response.data;
  }

  async function getAuthorityLookupTables(authority: string): Promise<string[]> {
    return (await axios.get<{
      authority: string,
      addresses: string[]
    }>(`http://localhost:3006/lookup/authority_addresses/${authority}`)).data.addresses
  }

  useEffect(() => {
    if (cluster == 'localnet') {
      getLocalnetConfig().then(config => {
        setMainConfig(config);
        return getAuthorityLookupTables(config.airspaces[0].lookupRegistryAuthority)
      }).then(addresses => {
        updateLookupTableAddresses(addresses);
      });
    } else {
      MarginClient.getConfig(cluster).then(config => setMainConfig(config));
      // TODO: update authority here too
    }
  }, [cluster, setMainConfig]);
}
