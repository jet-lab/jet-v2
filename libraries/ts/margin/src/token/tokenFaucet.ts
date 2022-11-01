import {
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  getAssociatedTokenAddress,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import { Connection, LAMPORTS_PER_SOL, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js"
import { AssociatedToken } from "./associatedToken"
import { Address, BN, AnchorProvider, translateAddress, Program } from "@project-serum/anchor"
import { MarginPrograms, MarginTokenConfig } from "../margin"
import { IDL as JetTestServiceIdl } from "../types/jetTestService"

const TEST_SERVICE_ID = new PublicKey("JPTSApMSqCHBww7vDhpaSmzipTV3qPg6vxub4qneKoy")

export class TokenFaucet {
  static async tokenRequest(
    provider: AnchorProvider,
    mint: Address,
    user: Address,
    destination: Address,
    lamports: BN
  ): Promise<TransactionInstruction> {
    const testService = new Program(JetTestServiceIdl, TEST_SERVICE_ID, provider)

    mint = translateAddress(mint)
    user = translateAddress(user)
    destination = translateAddress(destination)

    const tokenInfoAddress = PublicKey.findProgramAddressSync(
      [Buffer.from("token-info"), mint.toBuffer()],
      TEST_SERVICE_ID
    )

    return testService.methods
      .tokenRequest(lamports)
      .accounts({
        mint,
        destination,
        info: tokenInfoAddress[0],
        requester: user,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
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
  static async airdropToken(provider: AnchorProvider, user: Address, mint: Address, lamports: BN): Promise<string> {
    const instructions: TransactionInstruction[] = []

    // Check for user token account
    // If it doesn't exist add instructions to create it
    const address = await AssociatedToken.withCreate(instructions, provider, user, mint)

    // Create airdrop instructions
    instructions.push(await this.tokenRequest(provider, mint, user, address, lamports))

    // Execute airdrop
    return provider.sendAndConfirm(new Transaction().add(...instructions))
  }

  /** Airdrops native SOL if the mint is the native mint. */
  static async airdrop(
    provider: AnchorProvider,
    cluster: "localnet" | "devnet",
    lamports: BN,
    token: MarginTokenConfig,
    owner: Address = provider.wallet.publicKey
  ): Promise<string> {
    const mintAddress = translateAddress(token.mint)
    const ownerAddress = translateAddress(owner)

    const ix: TransactionInstruction[] = []

    const destination = AssociatedToken.derive(token.mint, owner)

    // Optionally create a token account for wallet
    if (!mintAddress.equals(NATIVE_MINT) && !(await AssociatedToken.exists(provider.connection, token.mint, owner))) {
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
      const endpoint =
        cluster == "localnet"
          ? provider.connection
          : new Connection("https://api.devnet.solana.com", AnchorProvider.defaultOptions().commitment)

      const blockhash = await endpoint.getLatestBlockhash()
      const signature = await endpoint.requestAirdrop(ownerAddress, lamports.toNumber())
      await endpoint.confirmTransaction({ signature, ...blockhash })
      return signature
    } else {
      ix.push(await this.tokenRequest(provider, mintAddress, owner, destination, lamports))
      return await provider.sendAndConfirm(new Transaction().add(...ix))
    }
  }
}
