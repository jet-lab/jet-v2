import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { PriceInfo } from "./accountPosition"

export interface IAdapter {
  adapterProgramId: PublicKey
  getPrice(mint: PublicKey): PriceInfo | undefined
  withRefreshPosition(instructions: TransactionInstruction[], positionTokenMint: PublicKey): Promise<void>
}
