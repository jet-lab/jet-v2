import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { PriceInfo } from "./accountPosition"

export interface IAdapter {
  programId: PublicKey
  getPrice(mint: PublicKey): PriceInfo | undefined
  withRefreshPosition(instructions: TransactionInstruction[], positionTokenMint: PublicKey): Promise<void>
}
