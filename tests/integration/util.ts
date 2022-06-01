import { AnchorProvider, InstructionNamespace } from "@project-serum/anchor"
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet"
import {
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
import {
  Account,
  Commitment,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionSignature
} from "@solana/web3.js"

import MARGIN_CONFIG from "../../libraries/ts/src/margin/config.json"

import { IDL as JetControlIDL, JetControl } from "../../libraries/ts/src/types/jetControl"
import { buildInstructions } from "../../libraries/ts/src/utils/idlBuilder"

const controlProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.controlProgramId)
const marginMetadataProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.metadataProgramId)

const controlInstructions = buildInstructions(JetControlIDL, controlProgramId) as InstructionNamespace<JetControl>

export async function createAuthority(provider: AnchorProvider, payer: Keypair): Promise<void> {
  const [authority] = await PublicKey.findProgramAddress([], controlProgramId)

  const accountInfo = await provider.connection.getAccountInfo(authority, "processed" as Commitment)
  if (!accountInfo) {
    const lamports = 1 * LAMPORTS_PER_SOL
    const airdropSignature = await provider.connection.requestAirdrop(authority, lamports)
    await provider.connection.confirmTransaction(airdropSignature)

    const ix = controlInstructions.createAuthority({
      accounts: {
        authority: authority,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId
      }
    })

    const tx = new Transaction().add(ix)

    await provider.sendAndConfirm(tx, [payer])
  }
}

export async function registerAdapter(
  provider: AnchorProvider,
  requester: Keypair,
  adapterProgramId: PublicKey,
  payer: Keypair
): Promise<void> {
  const [metadataAccount] = await PublicKey.findProgramAddress([adapterProgramId.toBuffer()], marginMetadataProgramId)

  const accountInfo = await provider.connection.getAccountInfo(metadataAccount, "processed" as Commitment)
  if (!accountInfo) {
    const [authority] = await PublicKey.findProgramAddress([], controlProgramId)

    const ix = controlInstructions.registerAdapter({
      accounts: {
        requester: requester.publicKey,
        authority,
        adapter: adapterProgramId,
        metadataAccount: metadataAccount,
        payer: payer.publicKey,
        metadataProgram: marginMetadataProgramId,
        systemProgram: SystemProgram.programId
      }
    })
    const tx = new Transaction().add(ix)
    await provider.sendAndConfirm(tx, [payer])
  }
}

async function sendTransaction(
  provider: AnchorProvider,
  transaction: Transaction,
  signers: Array<Account>
): Promise<TransactionSignature> {
  const signature = await provider.connection.sendTransaction(transaction, signers, {
    skipPreflight: false
  })
  const { value } = await provider.connection.confirmTransaction(signature, "recent")
  if (value?.err) {
    throw new Error(JSON.stringify(value.err))
  }
  return signature
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

export async function createTokenAccount(provider: AnchorProvider, mint: PublicKey, owner: PublicKey) {
  const tokenAddress = await getAssociatedTokenAddress(mint, owner, true)
  const transaction = new Transaction().add(
    createAssociatedTokenAccountInstruction(provider.wallet.publicKey, tokenAddress, owner, mint)
  )
  await provider.sendAndConfirm(transaction)
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
  const mintInfo = MintLayout.decode(Buffer.from(mintAccount!.data))
  return Number(mintInfo.supply) / pow10(decimals)
}

export async function getTokenBalance(provider: AnchorProvider, commitment: Commitment, tokenAddress: PublicKey) {
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
  fromTokenAccount: PublicKey,
  toTokenAccount: PublicKey
) {
  const transaction = new Transaction().add(
    createTransferCheckedInstruction(
      fromTokenAccount,
      mint,
      toTokenAccount,
      provider.wallet.publicKey,
      amount * pow10(decimals),
      decimals
    )
  )
  await provider.sendAndConfirm(transaction)
}
