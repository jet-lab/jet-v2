import { AnchorProvider } from "@project-serum/anchor"
import { PublicKey, AddressLookupTableProgram } from "@solana/web3.js"
import { Pool, SPLSwapPool } from "margin"
import { chunks } from "./array"
import { sendAndConfirm, sendAll } from "./sendAll"

/**
 * Create and populate a lookup table
 */
export const createMarginPoolLookupTable = async (
  provider: AnchorProvider,
  pools: Record<string, Pool>
): Promise<PublicKey> => {
  let addresses: PublicKey[] = []
  for (const pool in pools) {
    addresses = [...addresses, ...Object.values(pools[pool].addresses)]
  }

  return createLookupTable(provider, addresses)
}

export const crateSwapPoolLookupTable = async (
  provider: AnchorProvider,
  swapPools: SPLSwapPool[]
): Promise<PublicKey> => {
  let addresses: PublicKey[] = []
  for (const pool of swapPools) {
    addresses = [
      ...addresses,
      ...[
        new PublicKey(pool.authority),
        new PublicKey(pool.feeAccount),
        new PublicKey(pool.poolMint),
        new PublicKey(pool.tokenA),
        new PublicKey(pool.tokenB),
        new PublicKey(pool.tokenMintA),
        new PublicKey(pool.tokenMintB),
        new PublicKey(pool.authority)
      ]
    ]
  }

  return createLookupTable(provider, addresses)
}

export const createLookupTable = async (provider: AnchorProvider, accounts: PublicKey[]): Promise<PublicKey> => {
  // Craete lookup table
  const [lookupTableInst, lookupTableAddress] = AddressLookupTableProgram.createLookupTable({
    authority: provider.wallet.publicKey,
    payer: provider.wallet.publicKey,
    recentSlot: await provider.connection.getSlot()
  })

  await sendAndConfirm(provider, [lookupTableInst])

  // Remove duplicates
  const addresses = Array.from(new Set([...accounts.filter(a => typeof a.toBytes === "function")]))

  // Populate lookup table wth accounts
  const all = chunks(20, addresses).map(addresses => {
    return [
      AddressLookupTableProgram.extendLookupTable({
        authority: provider.wallet.publicKey,
        lookupTable: lookupTableAddress,
        addresses,
        payer: provider.wallet.publicKey
      })
    ]
  })

  await sendAll(provider, all)
  console.log("Lookup Table Address:", lookupTableAddress.toBase58())

  return lookupTableAddress
}
