import { Address, translateAddress } from "@project-serum/anchor"
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey"
import { PublicKey } from "@solana/web3.js"
import bs58 from "bs58"

export type AccountSeed = { toBytes(): Uint8Array } | { publicKey: PublicKey } | Uint8Array | string | Buffer

/**
 * Derive a PDA from the argued list of seeds.
 * @param {PublicKey} programId
 * @param {AccountSeed[]} seeds
 * @returns {Promise<PublicKey>}
 * @memberof JetClient
 */
 export function findDerivedAccount(programId: Address, ...seeds: AccountSeed[]): PublicKey {
    const seedBytes = seeds.map(s => {
      if (typeof s == "string") {
        const pubkeyBytes = bs58.decodeUnsafe(s)
        if (!pubkeyBytes || pubkeyBytes.length !== 32) {
          return Buffer.from(s)
        } else {
          return translateAddress(s).toBytes()
        }
      } else if ("publicKey" in s) {
        return s.publicKey.toBytes()
      } else if ("toBytes" in s) {
        return s.toBytes()
      } else {
        return s
      }
    })
  
    const [address] = findProgramAddressSync(seedBytes, translateAddress(programId))
    return address
  }