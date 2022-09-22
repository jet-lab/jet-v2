import { AnchorProvider, Provider } from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
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
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Commitment,
  ConfirmOptions,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

export class TestMint {
  keypair: Keypair;
  provider: AnchorProvider;
  decimals: number;

  constructor(decimals: number, keypair: Keypair, provider: AnchorProvider) {
    this.keypair = keypair;
    this.provider = provider;
    this.decimals = decimals;
  }

  get address() {
    return this.keypair.publicKey;
  }

  async createAndMintTo(
    amount: number,
    owner: PublicKey,
    payer: Keypair
  ): Promise<PublicKey> {
    const tokenAddress = await getAssociatedTokenAddress(
      this.keypair.publicKey,
      owner
    );
    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        tokenAddress,
        owner,
        this.keypair.publicKey
      ),
      createMintToCheckedInstruction(
        this.keypair.publicKey,
        tokenAddress,
        this.keypair.publicKey,
        amount,
        this.decimals
      )
    );
    await this.provider.connection.confirmTransaction(
      await this.provider.sendAndConfirm(transaction, [this.keypair, payer])
    );

    return tokenAddress;
  }
}

export class Transactor {
  private signers: Signer[];
  private provider: Provider;

  constructor(signers: Signer[], provider: Provider) {
    this.signers = signers;
    this.provider = provider;
  }

  async addSigner(signer: Signer) {
    this.signers.push(signer);
  }

  async signSendInstructions(
    instructions: TransactionInstruction[],
    opts?: ConfirmOptions
  ): Promise<string> {
    let tx = new Transaction().add(...instructions);
    let signers: Signer[] = [];

    // ha ha for loops go BRRRRRRRRRRRRR
    for (let ix in tx.instructions) {
      let ixn = tx.instructions[ix];
      for (let k in ixn.keys) {
        let meta = ixn.keys[k];
        for (let kp in this.signers) {
          let sgnr = this.signers[kp];
          if (
            sgnr.publicKey.toString() === meta.pubkey.toString() &&
            meta.isSigner
          ) {
            signers.push(sgnr);
          }
        }
      }
    }

    return await this.provider.sendAndConfirm!(tx, signers, opts);
  }
}

export async function createToken(
  provider: AnchorProvider,
  owner: Keypair,
  decimals: number,
  supply: number
): Promise<[PublicKey, PublicKey]> {
  const mint = Keypair.generate();
  const vault = Keypair.generate();
  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: owner.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: await getMinimumBalanceForRentExemptMint(provider.connection),
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeMintInstruction(
      mint.publicKey,
      decimals,
      owner.publicKey,
      null
    ),
    SystemProgram.createAccount({
      fromPubkey: owner.publicKey,
      newAccountPubkey: vault.publicKey,
      space: ACCOUNT_SIZE,
      lamports: await getMinimumBalanceForRentExemptAccount(
        provider.connection
      ),
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeAccountInstruction(
      vault.publicKey,
      mint.publicKey,
      owner.publicKey
    ),
    createMintToCheckedInstruction(
      mint.publicKey,
      vault.publicKey,
      owner.publicKey,
      BigInt(supply) * BigInt(pow10(decimals)),
      decimals
    )
  );
  await provider.sendAndConfirm(tx, [owner, mint, vault]);
  return [mint.publicKey, vault.publicKey];
}

export async function createTokenAccount(
  provider: AnchorProvider,
  mint: PublicKey,
  owner: PublicKey,
  payer: Keypair
) {
  const tokenAddress = await getAssociatedTokenAddress(mint, owner, true);
  const transaction = new Transaction().add(
    createAssociatedTokenAccountInstruction(
      payer.publicKey,
      tokenAddress,
      owner,
      mint
    )
  );
  await provider.sendAndConfirm(transaction, [payer]);
  return tokenAddress;
}

export async function createUserWallet(
  provider: AnchorProvider,
  lamports: number
): Promise<NodeWallet> {
  const account = Keypair.generate();
  const wallet = new NodeWallet(account);
  const airdropSignature = await provider.connection.requestAirdrop(
    account.publicKey,
    lamports
  );
  await provider.connection.confirmTransaction(airdropSignature);
  return wallet;
}

export async function getMintSupply(
  provider: AnchorProvider,
  mintPublicKey: PublicKey,
  decimals: number
) {
  const mintAccount = await provider.connection.getAccountInfo(mintPublicKey);
  if (!mintAccount) {
    throw new Error("Mint does not exist");
  }
  const mintInfo = MintLayout.decode(Buffer.from(mintAccount.data));
  return Number(mintInfo.supply) / pow10(decimals);
}

export async function getTokenAccountInfo(
  provider: AnchorProvider,
  address: PublicKey
) {
  const info = await provider.connection.getAccountInfo(address);
  if (!info) {
    throw new Error("Account does not exist");
  }
  return AccountLayout.decode(Buffer.from(info.data));
}

export async function getTokenBalance(
  provider: AnchorProvider,
  commitment: Commitment = "processed",
  tokenAddress: PublicKey
) {
  const balance = await provider.connection.getTokenAccountBalance(
    tokenAddress,
    commitment
  );
  return balance.value.uiAmount;
}

export function pow10(decimals: number): number {
  switch (decimals) {
    case 6:
      return 1_000_000;
    case 7:
      return 10_000_000;
    case 8:
      return 100_000_000;
    case 9:
      return 1_000_000_000;
    default:
      throw new Error(`Unsupported number of decimals: ${decimals}.`);
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
  );
  await provider.sendAndConfirm(transaction, [owner]);
}
