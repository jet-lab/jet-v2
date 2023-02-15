import { Cluster } from 'slices/settings';
import { APPLICATION_WS_EVENTS, JET_WS_EVENTS } from '../events';
import { PoolDataUpdate } from '../slices/pools';
import { useJetStore } from '../store';

let ws: WebSocket;
export const initWebsocket = (cluster?: Cluster, wallet?: string | null) => {
  if (ws) {
    ws.close();
  }

  try {
    let endpoint: string | undefined;
    switch (cluster) {
      case 'devnet':
        endpoint = process.env.REACT_APP_DEV_WS_API;
        break;
      case 'localnet':
        endpoint = process.env.REACT_APP_LOCAL_WS_API;
        break;
      case 'mainnet-beta':
        endpoint = process.env.REACT_APP_WS_API;
        break;
    }
    
    console.log('initialising websocket for ', cluster, endpoint);
    if (!endpoint) throw `No websocket environment variable set up.`;

    ws = new WebSocket(endpoint);

    ws.onopen = () => {
      if (!wallet) {
        return;
      }
      const subscriptionEvent: APPLICATION_WS_EVENTS = {
        type: 'SUBSCRIBE',
        payload: {
          wallet,
          // It's safe not to pass in margin accounts
          margin_accounts: []
        }
      };
      ws.send(JSON.stringify(subscriptionEvent));
    };

    ws.onmessage = (msg: MessageEvent<string>) => {
      const data: JET_WS_EVENTS = JSON.parse(msg.data);

      if (data.type === 'MARGIN-POOL-UPDATE') {
        const update: PoolDataUpdate = {
          address: data.payload.address,
          borrowed_tokens: data.payload.borrowed_tokens,
          deposit_tokens: data.payload.deposit_tokens
          // TODO figure out how to fetch these last two datapoints from pool manager
          // deposit_notes: new BN(data.payload.deposit_notes).toNumber(),
          // accrued_until: new Date(data.payload.accrued_until * 1000)
        };
        useJetStore.getState().updatePool(update);
      } else if (data.type === 'PRICE-UPDATE') {
        useJetStore.getState().updatePrices(data);
      }
    };
  } catch (e) {
    console.log(e);
  }
};
