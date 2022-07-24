import { AnchorProvider } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import {
  AccountLayout,
  ACCOUNT_SIZE,
  createAssociatedTokenAccountInstruction,
  createInitializeAccountInstruction,
  createInitializeMintInstruction,
  createMintToCheckedInstruction,
  createTransferCheckedInstruction,
  getAssociatedTokenAddress,
  getMinimumBalanceForRentExemptAccount,
  getMinimumBalanceForRentExemptMint,
  MintLayout,
  MINT_SIZE,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import { Commitment, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js"
import { MarginPrograms } from "../../libraries/ts/src"

import MARGIN_CONFIG from "../../libraries/ts/src/margin/config.json"

const controlProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.mainnet.controlProgramId)
const marginMetadataProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.mainnet.metadataProgramId)

export async function createAuthority(programs: MarginPrograms, provider: AnchorProvider): Promise<void> {
  const [authority] = await PublicKey.findProgramAddress([], controlProgramId)

  const accountInfo = await provider.connection.getAccountInfo(authority, "processed" as Commitment)
  if (!accountInfo) {
    const lamports = 1 * LAMPORTS_PER_SOL
    const airdropSignature = await provider.connection.requestAirdrop(authority, lamports)
    await provider.connection.confirmTransaction(airdropSignature)

    const tx = await programs.control.methods
      .createAuthority()
      .accounts({
        authority: authority,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .transaction()

    await provider.sendAndConfirm(tx)
  }
}

export async function registerAdapter(
  programs: MarginPrograms,
  provider: AnchorProvider,
  requester: Keypair,
  adapterProgramId: PublicKey,
  payer: Keypair
): Promise<void> {
  const [metadataAccount] = await PublicKey.findProgramAddress([adapterProgramId.toBuffer()], marginMetadataProgramId)

  const accountInfo = await provider.connection.getAccountInfo(metadataAccount, "processed" as Commitment)
  if (!accountInfo) {
    const [authority] = await PublicKey.findProgramAddress([], controlProgramId)

    const tx = await programs.control.methods
      .registerAdapter()
      .accounts({
        requester: requester.publicKey,
        authority,
        adapter: adapterProgramId,
        metadataAccount: metadataAccount,
        payer: payer.publicKey,
        metadataProgram: marginMetadataProgramId,
        systemProgram: SystemProgram.programId
      })
      .transaction()
    try {
      await provider.sendAndConfirm(tx)
    } catch (err) {
      console.log(err)
      throw err
    }
  }
}

export async function createToken(
  provider: AnchorProvider,
  owner: Keypair,
  decimals: number,
  supply: number
): Promise<[PublicKey, PublicKey]> {
  const mint = Keypair.generate()
  const vault = Keypair.generate()
  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: owner.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: await getMinimumBalanceForRentExemptMint(provider.connection),
      programId: TOKEN_PROGRAM_ID
    }),
    createInitializeMintInstruction(mint.publicKey, decimals, owner.publicKey, null),
    SystemProgram.createAccount({
      fromPubkey: owner.publicKey,
      newAccountPubkey: vault.publicKey,
      space: ACCOUNT_SIZE,
      lamports: await getMinimumBalanceForRentExemptAccount(provider.connection),
      programId: TOKEN_PROGRAM_ID
    }),
    createInitializeAccountInstruction(vault.publicKey, mint.publicKey, owner.publicKey),
    createMintToCheckedInstruction(
      mint.publicKey,
      vault.publicKey,
      owner.publicKey,
      BigInt(supply) * BigInt(pow10(decimals)),
      decimals
    )
  )
  await provider.sendAndConfirm(transaction, [owner, mint, vault])
  return [mint.publicKey, vault.publicKey]
}

export async function createTokenAccount(provider: AnchorProvider, mint: PublicKey, owner: PublicKey, payer: Keypair) {
  const tokenAddress = await getAssociatedTokenAddress(mint, owner, true)
  const transaction = new Transaction().add(
    createAssociatedTokenAccountInstruction(payer.publicKey, tokenAddress, owner, mint)
  )
  await provider.sendAndConfirm(transaction, [payer])
  return tokenAddress
}

export async function createUserWallet(provider: AnchorProvider, lamports: number): Promise<NodeWallet> {
  const account = Keypair.generate()
  const wallet = new NodeWallet(account)
  const airdropSignature = await provider.connection.requestAirdrop(account.publicKey, lamports)
  await provider.connection.confirmTransaction(airdropSignature)
  return wallet
}

export async function getMintSupply(provider: AnchorProvider, mintPublicKey: PublicKey, decimals: number) {
  const mintAccount = await provider.connection.getAccountInfo(mintPublicKey)
  if (!mintAccount) {
    throw new Error("Mint does not exist")
  }
  const mintInfo = MintLayout.decode(Buffer.from(mintAccount.data))
  return Number(mintInfo.supply) / pow10(decimals)
}

export async function getTokenAccountInfo(provider: AnchorProvider, address: PublicKey) {
  const info = await provider.connection.getAccountInfo(address)
  if (!info) {
    throw new Error("Account does not exist")
  }
  return AccountLayout.decode(Buffer.from(info.data))
}

export async function getTokenBalance(
  provider: AnchorProvider,
  commitment: Commitment = "processed",
  tokenAddress: PublicKey
) {
  const balance = await provider.connection.getTokenAccountBalance(tokenAddress, commitment)
  return balance.value.uiAmount
}

export function pow10(decimals: number): number {
  switch (decimals) {
    case 6:
      return 1_000_000
    case 7:
      return 10_000_000
    case 8:
      return 100_000_000
    case 9:
      return 1_000_000_000
    default:
      throw new Error(`Unsupported number of decimals: ${decimals}.`)
  }
}

export async function sendToken(
  provider: AnchorProvider,
  mint: PublicKey,
  amount: number,
  decimals: number,
  owner: Keypair,
  fromTokenAccount: PublicKey,
  toTokenAccount: PublicKey
) {
  const transaction = new Transaction().add(
    createTransferCheckedInstruction(
      fromTokenAccount,
      mint,
      toTokenAccount,
      owner.publicKey,
      amount * pow10(decimals),
      decimals
    )
  )
  await provider.sendAndConfirm(transaction, [owner])
}
