import { atom } from "recoil"
import { localStorageEffect } from "../effects/localStorageEffect"

export const FixedBorrowViewOrder = atom({
  key: "FixedBorrowViewOrder",
  default: ["accountSnapshot", "marketSelector", "fixedRow", "fullAccountBalance"] as string[],
  effects: [localStorageEffect("jetAppFixedBorrowViewOrder")],
  dangerouslyAllowMutability: true
})

export const FixedBorrowRowOrder = atom({
  key: "FixedBorrowRowOrder",
  default: ["fixedBorrowEntry", "fixedBorrowChart"],
  effects: [localStorageEffect("jetAppFixedBorrowRowOrder")],
  dangerouslyAllowMutability: true
})

export const FixedLendViewOrder = atom({
  key: "FixedLendViewOrder",
  default: ["accountSnapshot", "marketSelector", "fixedRow", "fullAccountBalance"] as string[],
  effects: [localStorageEffect("jetAppFixedLendViewOrder")],
  dangerouslyAllowMutability: true
})

export const FixedLendRowOrder = atom({
  key: "FixedLendRowOrder",
  default: ["fixedLendEntry", "fixedLendChart"],
  effects: [localStorageEffect("jetAppFixedLendRowOrder")],
  dangerouslyAllowMutability: true
})
