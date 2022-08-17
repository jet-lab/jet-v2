import {
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import { Connection, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js"
import { AssociatedToken } from "./associatedToken"
import { Address, BN, AnchorProvider, translateAddress } from "@project-serum/anchor"
import { MarginPrograms } from "../margin"

export class TokenFaucet {
  /**
   * TODO:
   * @private
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {PublicKey} tokenMint
   * @param {PublicKey} tokenFaucet
   * @param {PublicKey} tokenAccount
   * @memberof TokenFaucet
   */
  private static async withAirdrop(
    instructions: TransactionInstruction[],
    programs: MarginPrograms,
    tokenMint: Address,
    tokenFaucet: Address,
    tokenAccount: Address,
    lamports: BN
  ) {
    if (!programs.config.faucetProgramId) {
      throw new Error("No spl token faucet program id")
    }

    const pubkeyNonce = await PublicKey.findProgramAddress(
      [Buffer.from("faucet", "utf8")],
      translateAddress(programs.config.faucetProgramId)
    )

    const keys = [
      { pubkey: pubkeyNonce[0], isSigner: false, isWritable: false },
      {
        pubkey: translateAddress(tokenMint),
        isSigner: false,
        isWritable: true
      },
      { pubkey: translateAddress(tokenAccount), isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: translateAddress(tokenFaucet), isSigner: false, isWritable: false }
    ]

    const faucetIx = new TransactionInstruction({
      programId: translateAddress(programs.config.faucetProgramId),
      data: Buffer.from([1, ...lamports.toArray("le", 8)]),
      keys
    })

    instructions.push(faucetIx)
  }

  /**
   * TODO:
   * @static
   * @param {AnchorProvider} provider
   * @param {Address} faucet
   * @param {Address} user
   * @param {Address} mint
   * @returns {Promise<string>}
   * @memberof TokenFaucet
   */
  static async airdropToken(
    programs: MarginPrograms,
    provider: AnchorProvider,
    faucet: Address,
    user: Address,
    mint: Address,
    lamports: BN
  ): Promise<string> {
    const instructions: TransactionInstruction[] = []

    // Check for user token account
    // If it doesn't exist add instructions to create it
    const address = await AssociatedToken.withCreate(instructions, provider, user, mint)

    // Create airdrop instructions
    await this.withAirdrop(instructions, programs, mint, faucet, address, lamports)

    // Execute airdrop
    return provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  /** Airdrops native SOL if the mint is the native mint. */
  static async airdrop(
    programs: MarginPrograms,
    provider: AnchorProvider,
    lamports: BN,
    mint: Address,
    owner: Address,
    faucet?: Address
  ): Promise<string> {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)

    const ix: TransactionInstruction[] = []

    const destination = AssociatedToken.derive(mint, owner)

    // Optionally create a token account for wallet
    if (!mintAddress.equals(NATIVE_MINT) && !(await AssociatedToken.exists(provider.connection, mint, owner))) {
      const createTokenAccountIx = createAssociatedTokenAccountInstruction(
        ownerAddress,
        destination,
        ownerAddress,
        mintAddress
      )
      ix.push(createTokenAccountIx)
    }

    if (mintAddress.equals(NATIVE_MINT)) {
      // Sol airdrop
      // Use a specific endpoint. A hack because some devnet endpoints are unable to airdrop
      const endpoint = new Connection("https://api.devnet.solana.com", AnchorProvider.defaultOptions().commitment)
      const airdropTxnId = await endpoint.requestAirdrop(ownerAddress, parseInt(lamports.toString()))
      await endpoint.confirmTransaction(airdropTxnId)
      return airdropTxnId
    } else if (faucet) {
      // Faucet airdrop
      await this.withAirdrop(ix, programs, mintAddress, translateAddress(faucet), destination, lamports)
      return await provider.sendAndConfirm(new Transaction().add(...ix))
    } else {
      // Mint to the destination token account
      const mintToIx = createMintToInstruction(mintAddress, destination, ownerAddress, BigInt(lamports.toString()))
      ix.push(mintToIx)
      return await provider.sendAndConfirm(new Transaction().add(...ix))
    }
  }
}
