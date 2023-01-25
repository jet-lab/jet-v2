import { JetStore } from "../../state"
import { PoolDetails } from "./types"

export const updatePool = (state: JetStore, address: string, update: Partial<PoolDetails>) => {
    const pool = {
        ...state.pools[address],
        ...update
    }
    return ({ pools: { [address]: pool } })
}

export const setPools = (pools: Record<string, PoolDetails>) => {
    return ({ pools })
}