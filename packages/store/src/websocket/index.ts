import { useJetStore } from '../store';

let connectionRetryTimeout: NodeJS.Timeout;
let pendingTimeoutType: string | undefined;

export let ws: WebSocket | undefined;
export const initWebsocket = (cluster?: Cluster, wallet?: string | null) => {
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
  console.log('init websocket', ws, cluster, wallet);
  if (ws && ws.url === endpoint) {
    // We terminate this connection as it's likely a duplicate
    return;
  } else if (ws && ws.url !== endpoint) {
    // We use a private code to indicate the reason why the socket is being closed.
    // If the socket is closed due to this code, we don't reconnect.
    // See https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent/code#value for more info.
    ws!.close(4321);
    // ws = undefined;
  }

  if (cluster !== pendingTimeoutType) {
    clearTimeout(connectionRetryTimeout);
    pendingTimeoutType = undefined;
  }

  try {
    // console.log('initialising websocket for ', cluster, endpoint);
    if (!endpoint) throw `No websocket environment variable set up.`;

    ws = new WebSocket(endpoint);
    setTimeout(() => {
      // Check if the socket is still conneecting
      if (ws && ws.readyState === 0) {
        ws.close(4322);
        ws = undefined;
      }
    }, 2000)

    ws.onopen = () => {
      if (!wallet) {
        return;
      }
      const subscriptionEvent: APPLICATION_WS_EVENTS = {
        type: 'SUBSCRIBE',
        payload: {
          wallet,
          margin_accounts: []
        }
      };
      ws?.send(JSON.stringify(subscriptionEvent));
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
      } else if (data.type === 'MARGIN-ACCOUNT-UPDATE') {
        useJetStore.getState().updateMarginAccount(data.payload);
      } else if (data.type === 'MARGIN-ACCOUNT-LIST') {
        const map = {};
        for (const el of data.payload.accounts) {
          map[el.address] = el;
        }
        useJetStore.getState().initAllMarginAccounts(map);
      } else if (data.type === 'OPEN-ORDER-UPDATE') {
        useJetStore.getState().updateOpenOrders(data.payload);
      } else if (data.type === 'FIXED-TERM-POSITION-UPDATE') {
        useJetStore.getState().updateOpenPositions(data.payload);
      }
    };

    ws.onclose = (e: CloseEvent) => {
      // 1006 = Abnormal closure, the browser closes the connection during negotiation
      // 4321 = Our custom code to signal that we don't want to recreate the ws 
      // 4322 = Our custom code to signal a change in ws.url
      if (e.code === 4321 || e.code === 4322) {
        return;
      }
      if (ws?.url !== endpoint) {
        // If a socket exists but is pointing to the wrong endpoint, don't continue.
        // There is another connection attempt in progress (due to racy conditions).
        return;
      }
      ws = undefined;
      connectionRetryTimeout = setTimeout(() => {
        pendingTimeoutType = cluster;
        initWebsocket(cluster, wallet);
      }, 1000);
    }

    ws.onerror = (_: Event) => {
      ws = undefined;;
      connectionRetryTimeout = setTimeout(() => {
        pendingTimeoutType = cluster;
        initWebsocket(cluster, wallet);
      }, 1000);
    };
  } catch (e) {
    console.log(e);
  }
};
