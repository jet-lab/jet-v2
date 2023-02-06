import { LoadingOutlined } from '@ant-design/icons';
import { useJetStore } from '@jet-lab/store';
import { NetworkState } from '../state/network/network-state';

interface WaitingForNetwork {
  networkState: NetworkState;
}

export const WaitingForNetworkView = ({ networkState }: WaitingForNetwork) => {
  const cluster = useJetStore(state => state.settings.cluster);

  return networkState === 'loading' ? (
    <div className="centered-loading-container">
      <LoadingOutlined />
    </div>
  ) : (
    <div className="connection-failed-container">
      <span>There was an error connecting to the selected network.</span>
      <span>You are currently connected to {cluster} but the application did not get a response from the network.</span>
    </div>
  );
};
