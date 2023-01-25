import BN from 'bn.js'
import { create } from 'zustand'
import { createPoolsSlice } from './state/pools/pools-slice'
import { PoolsSlice } from './state/pools/types'
import { createPricesSlice } from './state/prices/prices-slice'
import { PricesSlice } from './state/prices/types'

export type JetStore = PoolsSlice & PricesSlice

export const useJetStore = create<JetStore>()((...a) => ({
    ...createPoolsSlice(...a),
    ...createPricesSlice(...a),
}))

const ws = new WebSocket(`${process.env.WS_API}`)

ws.onmessage = (msg: MessageEvent<string>) => {
    const data = JSON.parse(msg.data);
    if (data.key === 'MarginPool') {
        const update = {
            address: data.payload.address,
            borrowed_tokens: new BN(data.payload.borrowed_tokens).toNumber(),
            deposit_tokens: new BN(data.payload.deposit_tokens).toNumber(),
            deposit_notes: new BN(data.payload.deposit_notes).toNumber(),
            accrued_until: new Date(data.payload.accrued_until * 1000)
        }
        useJetStore.getState().updatePool(update.address, update)
    }
}