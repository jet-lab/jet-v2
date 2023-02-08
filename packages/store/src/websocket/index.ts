import { Cluster } from 'slices/settings';
import { APPLICATION_WS_EVENTS, JET_WS_EVENTS } from '../events';
import { PoolDataUpdate } from '../slices/pools';
import { useJetStore } from '../store';

let ws: WebSocket;
export const initWebsocket = (cluster?: Cluster) => {
  if (ws) {
    ws.close();
  }

  try {
    let endpoint: string | undefined;
    switch (cluster) {
      case 'devnet':
        endpoint = process.env.DEV_WS_API;
        break;
      case 'localnet':
        endpoint = process.env.LOCAL_WS_API;
        break;
      case 'mainnet-beta':
        endpoint = process.env.WS_API;
        break;
    }
    if (!endpoint) throw `No websocket environment variable set up.`;

    console.log('initialising websocket for ', cluster, endpoint);

    ws = new WebSocket(endpoint);

    ws.onopen = () => {
      const subscriptionEvent: APPLICATION_WS_EVENTS = {
        type: 'SUBSCRIBE',
        payload: {
          wallet: 'APhQTneeYjR8A5E3BuJBZFjHKpWdxHhTdiE1nuzoT553',
          margin_accounts: [
            'B8Tifsx1p22hto44FBo3sEt5nJmHtzTUNbM4f9UP42GV',
            'GT7eBGzue4e1Bq7N3Qox518nsCfyzEkEZeKwpD2vQMVM'
          ]
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
