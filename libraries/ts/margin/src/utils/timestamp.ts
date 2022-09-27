import { BN } from "@project-serum/anchor"

export function getTimestamp() {
  return new BN(Math.floor(Date.now() / 1000))
}
