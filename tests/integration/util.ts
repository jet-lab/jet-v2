import { InstructionNamespace } from "@project-serum/anchor"
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
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionSignature
} from "@solana/web3.js"
import assert from 'assert'

import MARGIN_CONFIG from "../../libraries/ts/src/margin/config.json"

import { IDL as JetControlIDL, JetControl } from "../../libraries/ts/src/types/jetControl"
import { buildInstructions } from "../../libraries/ts/src/utils/idlBuilder"

const controlProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.controlProgramId)
const marginMetadataProgramId: PublicKey = new PublicKey(MARGIN_CONFIG.localnet.metadataProgramId)

const controlInstructions = buildInstructions(JetControlIDL, controlProgramId) as InstructionNamespace<JetControl>

export async function createAuthority(connection: Connection, payer: Keypair): Promise<void> {
  const [authority] = await PublicKey.findProgramAddress([], controlProgramId)

  const accountInfo = await connection.getAccountInfo(authority, "processed" as Commitment);
  if (!accountInfo) {
    const lamports = 1 * LAMPORTS_PER_SOL
    const airdropSignature = await connection.requestAirdrop(authority, lamports)
    await connection.confirmTransaction(airdropSignature)

    const ix = controlInstructions.createAuthority({
      accounts: {
        authority: authority,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId
      }
    })

    const tx = new Transaction().add(ix)

    await sendTransaction(connection, tx, [new Account(payer.secretKey)])
  }
}

export async function registerAdapter(
  connection: Connection,
  requester: Keypair,
  adapterProgramId: PublicKey,
  payer: Keypair
): Promise<void> {
  const [metadataAccount] = await PublicKey.findProgramAddress([adapterProgramId.toBuffer()], marginMetadataProgramId)

  const accountInfo = await connection.getAccountInfo(metadataAccount, "processed" as Commitment);
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
    await sendTransaction(connection, tx, [new Account(payer.secretKey)])
  }
}

async function sendTransaction(
  connection: Connection,
  transaction: Transaction,
  signers: Array<Account>
): Promise<TransactionSignature> {
  const signature = await connection.sendTransaction(transaction, signers, {
    skipPreflight: false
  })
  const { value } = await connection.confirmTransaction(signature, "recent")
  if (value?.err) {
    throw new Error(JSON.stringify(value.err))
  }
  return signature
}

export async function createToken(
  connection: Connection,
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
      lamports: await getMinimumBalanceForRentExemptMint(connection),
      programId: TOKEN_PROGRAM_ID
    }),
    createInitializeMintInstruction(mint.publicKey, decimals, owner.publicKey, null),
    SystemProgram.createAccount({
      fromPubkey: owner.publicKey,
      newAccountPubkey: vault.publicKey,
      space: ACCOUNT_SIZE,
      lamports: await getMinimumBalanceForRentExemptAccount(connection),
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
  await sendAndConfirmTransaction(connection, transaction, [owner, mint, vault])
  return [mint.publicKey, vault.publicKey]
}

export async function createTokenAccount(connection: Connection, mint: PublicKey, owner: PublicKey, payer: Keypair) {
  const tokenAddress = await getAssociatedTokenAddress(mint, owner, true)
  const transaction = new Transaction().add(
    createAssociatedTokenAccountInstruction(payer.publicKey, tokenAddress, owner, mint)
  )
  await sendAndConfirmTransaction(connection, transaction, [payer])
  return tokenAddress
}

export async function createUserWallet(connection: Connection, lamports: number): Promise<NodeWallet> {
  const account = Keypair.generate()
  const wallet = new NodeWallet(account)
  const airdropSignature = await connection.requestAirdrop(account.publicKey, lamports)
  await connection.confirmTransaction(airdropSignature)
  return wallet
}

export async function getMintSupply(connection: Connection, mintPublicKey: PublicKey, decimals: number) {
  const mintAccount = await connection.getAccountInfo(mintPublicKey)
  const mintInfo = MintLayout.decode(Buffer.from(mintAccount!.data))
  return Number(mintInfo.supply) / pow10(decimals)
}

export async function getTokenBalance(connection: Connection, commitment: Commitment, tokenAddress: PublicKey) {
  const balance = await connection.getTokenAccountBalance(tokenAddress, commitment)
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
  connection: Connection,
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
  await sendAndConfirmTransaction(connection, transaction, [owner])
}
