import { LoadingOutlined } from "@ant-design/icons"
import { useRecoilValue } from "recoil"
import { NetworkState } from "../state/network/network-state"
import { Cluster, PreferredRpcNode, RpcNodes } from "../state/settings/settings"

interface WaitingForNetwork {
    networkState: NetworkState
}

export const WaitingForNetworkView = ({
    networkState
}: WaitingForNetwork) => {
    const cluster = useRecoilValue(Cluster)

    return networkState === 'loading' ?
    <div className='centered-loading-container'><LoadingOutlined /></div> :
    <div className='connection-failed-container'>
        <span>There was an error connecting to the selected network.</span>
        <span>You are currently connected to {cluster} but the application did not get a response from the network.</span>
    </div>
}