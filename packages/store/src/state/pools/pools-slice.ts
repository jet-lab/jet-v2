import { StateCreator } from "zustand"
import { JetStore } from "../../state"
import { updatePool, setPools } from './pools-actions'
import { PoolDetails, PoolsSlice } from "./types"

export const createPoolsSlice: StateCreator<JetStore, [], [], PoolsSlice> = (set) => ({
    pools: {},
    updatePool: (address: string, update: Partial<PoolDetails>) => set((state) => updatePool(state, address, update)),
    setPools: (pools: Record<string, PoolDetails>) => set(() => setPools(pools))
})