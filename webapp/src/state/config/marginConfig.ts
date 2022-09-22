import { useEffect } from 'react';
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

  useEffect(() => {
    MarginClient.getConfig(cluster).then(config => setMarginConfig(config));
  }, [cluster, setMarginConfig]);
}
