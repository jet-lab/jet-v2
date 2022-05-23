import assert from "assert"
import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { AccountLayout, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  TransactionSignature
} from "@solana/web3.js"
import { MarginPool } from "./pool"
import { AccountPositionList, AccountPositionListLayout, MarginAccountData } from "./state"
import { MarginPrograms } from "./marginClient"
import { findDerivedAccount } from "../utils/pda"

export class MarginAccount {
  static readonly SEED_MAX_VALUE = 65535
  public address: PublicKey;
  public owner: PublicKey;

  /**
   * Creates an instance of margin account.
   * @param {MarginPrograms} programs
   * @param {Provider} provider
   * @param {PublicKey} address The address of the margin account
   * @param {Address} owner
   * @param {number} seed
   * @param {(MarginAccountData | null)} info
   * @param {(AccountPositionList | null)} positions
   * @memberof MarginAccount
   */
  constructor(
    public programs: MarginPrograms,
    public provider: AnchorProvider,
    address: Address,
    owner: Address,
    public seed: number,
    public info: MarginAccountData | null,
    private positions: AccountPositionList | null
  ) {
    this.address = translateAddress(address);
    this.owner = translateAddress(owner);
  }

  /**
   * Derive margin account PDA from owner address and seed
   *
   * @private
   * @static
   * @param {Address} marginProgramId
   * @param {Address} owner
   * @param {number} seed
   * @return {PublicKey}
   * @memberof MarginAccount
   */
  static derive(programs: MarginPrograms, owner: Address, seed: number): PublicKey {
    if (seed > this.SEED_MAX_VALUE || seed < 0) {
      console.log(`Seed is not within the range: 0 <= seed <= ${this.SEED_MAX_VALUE}.`)
    }
    const buffer = Buffer.alloc(2)
    buffer.writeUInt16LE(seed)
    return findDerivedAccount(programs.config.marginProgramId, owner, buffer)
  }

  static deriveTokenMetadata(programs: MarginPrograms, tokenMint: Address) {
    const tokenMintAddress = translateAddress(tokenMint)
    return findDerivedAccount(programs.config.metadataProgramId, tokenMintAddress)
  }

  /**
   *
   * @param {MarginPrograms} programs
   * @param {Address} owner
   * @param {number} seed
   * @returns {Promise<MarginAccount>}
   */
  static async load(
    programs: MarginPrograms,
    provider: AnchorProvider,
    owner: Address,
    seed: number
  ): Promise<MarginAccount> {
    const ownerPubkey = translateAddress(owner)
    const address = this.derive(programs, ownerPubkey, seed)
    const marginAccount = new MarginAccount(programs, provider, address, ownerPubkey, seed, null, null)

    await marginAccount.refresh()

    return marginAccount
  }

  async refresh() {
    this.info = await this.programs.margin.account.marginAccount.fetchNullable(this.address)

    this.positions = this.info ? AccountPositionListLayout.decode(new Uint8Array(this.info.positions)) : null
  }

  static async exists(programs: MarginPrograms, owner: Address, seed: number): Promise<boolean> {
    const ownerPubkey = translateAddress(owner)
    const address = this.derive(programs, ownerPubkey, seed)
    const info = await programs.margin.provider.connection.getAccountInfo(address)
    return !!info
  }

  async createAccount() {
    const tx = new Transaction()
    tx.add(await this.makeCreateAccountInstruction())
    return await this.provider.sendAndConfirm!(tx)
  }

  /// Get instruction to create the account
  async makeCreateAccountInstruction(): Promise<TransactionInstruction> {
    const ownerAddress = translateAddress(this.owner)
    const marginAccount = MarginAccount.derive(this.programs, this.owner, this.seed)
    return await this.programs.margin.methods
      .createAccount(this.seed)
      .accounts({
        owner: this.owner,
        payer: this.provider.wallet.publicKey,
        marginAccount: marginAccount,
        systemProgram: SystemProgram.programId
      })
      .instruction()
  }

  //Deposit
  /// Transaction to deposit tokens into a margin account
  ///
  /// # Params
  ///
  /// `token_mint` - The address of the mint for the tokens being deposited
  /// `source` - The token account that the deposit will be transfered from
  /// `amount` - The amount of tokens to deposit
  async deposit(marginPool: MarginPool, source: Address, amount: BN) {
    await this.refresh()
    const position = await this.getOrCreatePosition(marginPool.addresses.depositNoteMint)
    assert(position)

    const tx = new Transaction()
    tx.add(
      await marginPool.makeDepositInstruction(this.owner, source, position.address, amount),
      await this.makeUpdatePositionBalanceInstruction(position.address)
    )
    return await this.provider.sendAndConfirm!(tx)
  }

  //TODO Withdraw

  async getOrCreatePosition(tokenMint: Address) {
    assert(this.positions)
    const tokenMintAddress = translateAddress(tokenMint)

    for (let i = 0; i < this.positions.length.toNumber(); i++) {
      const position = this.positions.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    await this.registerPosition(tokenMintAddress)
    await this.refresh()

    for (let i = 0; i < this.positions.length.toNumber(); i++) {
      const position = this.positions.positions[i]
      if (position.token.equals(tokenMintAddress)) {
        return position
      }
    }

    throw new Error("Unable to register position.")
  }

  async getTokenMetadata(tokenMint: Address) {
    const metadataAddress = MarginAccount.deriveTokenMetadata(this.programs, tokenMint)
    return await this.programs.metadata.account.tokenMetadata.fetch(metadataAddress)
  }

  async updatePositionBalance(account: PublicKey) {
    const tx = new Transaction()
    tx.add(await this.makeUpdatePositionBalanceInstruction(account))
    return await this.provider.sendAndConfirm!(tx)
  }

  /// Get instruction to update the accounting for assets in
  /// the custody of the margin account.
  ///
  /// # Params
  ///
  /// `account` - The account address that has had a balance change
  async makeUpdatePositionBalanceInstruction(account: PublicKey): Promise<TransactionInstruction> {
    return await this.programs.margin.methods
      .updatePositionBalance()
      .accounts({
        marginAccount: this.address,
        tokenAccount: account
      })
      .instruction()
  }

  async registerPosition(token_mint: PublicKey): Promise<TransactionSignature> {
    let tx = new Transaction()
    const [tokenAccount, ix] = await this.makeRegisterPositionInstruction(token_mint)
    tx.add(ix)
    return this.provider.sendAndConfirm!(tx)
  }

  /// Get instruction to register new position
  ///
  /// # Params
  ///
  /// `token_mint` - The mint for the relevant token for the position
  /// `token_oracle` - The oracle account with price information on the token
  ///
  /// # Returns
  ///
  /// Returns the instruction, and the address of the token account to be
  /// created for the position.
  async makeRegisterPositionInstruction(token_mint: PublicKey): Promise<[PublicKey, TransactionInstruction]> {
    const token_account = findDerivedAccount(this.programs.config.marginProgramId, this.address, token_mint)

    const metadata = findDerivedAccount(this.programs.config.metadataProgramId, token_mint)

    const ix = await this.programs.margin.methods
      .registerPosition()
      .accounts({
        authority: this.owner, //this.authority,
        payer: this.provider.wallet.publicKey,
        marginAccount: this.address,
        positionTokenMint: token_mint,
        metadata,
        tokenAccount: token_account,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId
      })
      .instruction()
    return [token_account, ix]
  }

  async closePosition(tokenAccount: PublicKey) {
    const tx = new Transaction()
    const ix = await this.makeClosePositionInstruction(tokenAccount)
    tx.add(ix)
    return await this.provider.sendAndConfirm(tx)
  }

  /// Get instruction to close a position
  ///
  /// # Params
  ///
  /// `token_account` - The address of the token account for the position being closed
  async makeClosePositionInstruction(tokenAccount: PublicKey): Promise<TransactionInstruction> {
    const authority = findDerivedAccount(this.programs.config.controlProgramId)

    return await this.programs.margin.methods
      .closePosition()
      .accounts({
        authority: authority,
        receiver: this.provider.wallet.publicKey,
        marginAccount: this.address,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  static async getTokenAccountInfo(connection: Connection, address: PublicKey) {
    const info = await connection.getAccountInfo(address)
    return AccountLayout.decode(Buffer.from(info!.data))
  }
}
