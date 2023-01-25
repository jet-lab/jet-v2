import { StateCreator } from "zustand";
import { JetStore } from "../../state";
import { PricesSlice } from "./types";

export const createPricesSlice: StateCreator<JetStore, [], [], PricesSlice > = (_set) => ({
    prices: {}
})