import { createContext, useContext, useState } from 'react';

// Cluster setting context
export type Cluster = 'mainnet-beta' | 'devnet';
interface ClusterSetting {
  clusterSetting: Cluster;
  setClusterSetting: (clusterSetting: Cluster) => void;
}
export const ClusterSettingContext = createContext<ClusterSetting>({
  clusterSetting: 'mainnet-beta',
  setClusterSetting: () => null
});

export function ClusterSettingProvider(props: { children: JSX.Element }): JSX.Element {
  const preference = localStorage.getItem('jetCluster');
  const [clusterSetting, setClusterSetting] = useState<Cluster>(
    preference === 'mainnet-beta' || preference === 'devnet' ? preference : 'mainnet-beta'
  );

  return (
    <ClusterSettingContext.Provider
      value={{
        clusterSetting,
        setClusterSetting
      }}>
      {props.children}
    </ClusterSettingContext.Provider>
  );
}

// Cluster setting hook
export const useClusterSetting = () => {
  const context = useContext(ClusterSettingContext);
  return context;
};
