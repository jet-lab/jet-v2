import { Address } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { AccountMeta, Connection, PublicKey, TransactionInstruction } from "@solana/web3.js"
import { MarginAccount } from "margin"
import { FixedTermMarket } from "./fixedTerm"
import { WasmAccountMeta, WasmTransactionInstruction } from "wasm"

export class DerivedAccount {
  public address: PublicKey
  public bumpSeed: number

  constructor(address: PublicKey, bumpSeed: number) {
    this.address = address
    this.bumpSeed = bumpSeed
  }
}
interface ToBytes {
  toBytes(): Uint8Array
}

interface HasPublicKey {
  publicKey: PublicKey
}

type DerivedAccountSeed = HasPublicKey | ToBytes | Uint8Array | string

export async function findFixedTermDerivedAccount(
  seeds: DerivedAccountSeed[],
  programId: PublicKey
): Promise<PublicKey> {
  const seedBytes = seeds.map(s => {
    if (typeof s == "string") {
      return Buffer.from(s)
    } else if ("publicKey" in s) {
      return s.publicKey.toBytes()
    } else if ("toBytes" in s) {
      return s.toBytes()
    } else {
      return s
    }
  })
  const [address, bumpSeed] = await PublicKey.findProgramAddress(seedBytes, programId)
  return new DerivedAccount(address, bumpSeed).address
}

export const fetchData = async (connection: Connection, address: Address): Promise<Buffer> => {
  let data = (await connection.getAccountInfo(new PublicKey(address)))?.data
  if (!data) {
    throw "could not fetch account"
  }

  return data
}

export const logAccounts = ({ ...accounts }) => {
  for (let name in accounts) {
    console.log(name + ": " + accounts[name])
  }
}

export const refreshAllMarkets = async (
  markets: FixedTermMarket[],
  ixs: TransactionInstruction[],
  marginAccount: MarginAccount,
  marketAddres?: PublicKey
) => {
  await Promise.all(
    markets.map(async market => {
      const marketUserInfo = await market.fetchMarginUser(marginAccount)
      const marketUser = await market.deriveMarginUserAddress(marginAccount)
      // We need to refresh the currnet market being created
      // as the market gets created with an existing position, but the user will not yet be found
      if (marketUserInfo || marketAddres?.equals(market.address)) {
        const refreshIx = await market.program.methods
          .refreshPosition(true)
          .accounts({
            marginUser: marketUser,
            marginAccount: marginAccount.address,
            market: market.addresses.market,
            underlyingOracle: market.addresses.underlyingOracle,
            ticketOracle: market.addresses.ticketOracle,
            tokenProgram: TOKEN_PROGRAM_ID
          })
          .instruction()

        await marginAccount.withAccountingInvoke({
          instructions: ixs,
          adapterInstruction: refreshIx
        })
      }
    })
  )
}

/**
 * Translates the deserialized instruction to the native `TransactionInstruction` type
 * @param ix the deserialized instruction to translate
 * @returns
 */
export function translateWasmInstruction(ix: WasmTransactionInstruction): TransactionInstruction {
  return new TransactionInstruction({
    keys: ix.accounts.map((a: WasmAccountMeta): AccountMeta => {
      return {
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.is_signer,
        isWritable: a.is_writable
      }
    }),
    programId: new PublicKey(ix.program_id),
    data: Buffer.from(ix.data)
  })
}
