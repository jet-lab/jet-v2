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
import { Pool } from "./pool/Pool"
import { AccountPositionList, AccountPositionListLayout, MarginAccountData } from "./state"
import { MarginPrograms } from "./marginClient"
import { findDerivedAccount } from "../utils/pda"
import { AssociatedToken, MarginTokens } from ".."

export interface MarginAccountAddresses {
  marginAccount: PublicKey
  owner: PublicKey
  positions: Record<string, MarginPositionAddresses>
}

export interface MarginPositionAddresses {
  account: PublicKey
  tokenAccount: PublicKey
  tokenMint: PublicKey
  tokenMetadata: PublicKey
}

export class MarginAccount {
  static readonly SEED_MAX_VALUE = 65535
  get address() {
    return this.addresses.marginAccount
  }
  get owner() {
    return this.addresses.owner
  }

  /**
   * Creates an instance of margin account.
   * @param {MarginPrograms} programs
   * @param {Provider} provider The provider and wallet that can sign for this margin account
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
    public addresses: MarginAccountAddresses,
    public seed: number,
    public info: MarginAccountData | null,
    public positions: AccountPositionList | null
  ) {}

  static async loadTokens(programs: MarginPrograms, owner: Address): Promise<Record<MarginTokens, AssociatedToken>> {
    const tokenConfigs = Object.values(programs.config.tokens)

    const mints = tokenConfigs.map(token => token.mint)
    const decimals = tokenConfigs.map(token => token.decimals)

    const tokens = await AssociatedToken.loadMultipleOrNative(
      programs.margin.provider.connection,
      mints,
      decimals,
      owner
    )

    const tokensMap: Record<string, AssociatedToken> = {}
    for (let i = 0; i < tokens.length; i++) {
      tokensMap[tokenConfigs[i].symbol] = tokens[i]
    }
    return tokensMap
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
  static derive(programs: MarginPrograms, owner: Address, seed: number): MarginAccountAddresses {
    if (seed > this.SEED_MAX_VALUE || seed < 0) {
      console.log(`Seed is not within the range: 0 <= seed <= ${this.SEED_MAX_VALUE}.`)
    }
    const ownerAddress = translateAddress(owner)
    const buffer = Buffer.alloc(2)
    buffer.writeUInt16LE(seed)
    const marginAccount = findDerivedAccount(programs.config.marginProgramId, owner, buffer)

    const tokenConfigs = Object.values(programs.config.tokens)
    const positionAddressesList: MarginPositionAddresses[] = tokenConfigs.map(tokenConfig => {
      const tokenMint = translateAddress(tokenConfig.mint)
      const account = findDerivedAccount(programs.config.marginProgramId, marginAccount, tokenMint)
      const tokenAccount = findDerivedAccount(programs.config.marginProgramId, marginAccount, tokenMint)
      const tokenMetadata = findDerivedAccount(programs.config.metadataProgramId, tokenMint)
      return {
        tokenMint,
        account,
        tokenAccount,
        tokenMetadata
      }
    })

    const positions: Record<string, MarginPositionAddresses> = {}
    for (let i = 0; i < positionAddressesList.length; i++) {
      positions[translateAddress(tokenConfigs[i].mint).toBase58()] = positionAddressesList[i]
    }
    return { marginAccount, owner: ownerAddress, positions }
  }

  static deriveLiquidation(programs: MarginPrograms, marginAccount: MarginAccount, liquidator: Address) {
    return findDerivedAccount(programs.config.marginProgramId, marginAccount.address, liquidator)
  }

  static deriveTokenMetadata(programs: MarginPrograms, tokenMint: Address) {
    const tokenMintAddress = translateAddress(tokenMint)
    return findDerivedAccount(programs.config.metadataProgramId, tokenMintAddress)
  }

  /**
   *
   * @param {MarginPrograms} programs
   * @param {AnchorProvider} provider The provider and wallet that can sign for this margin account
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
    const addresses = this.derive(programs, ownerPubkey, seed)
    const marginAccount = new MarginAccount(programs, provider, addresses, seed, null, null)

    await marginAccount.refresh()

    return marginAccount
  }

  async refresh() {
    this.info = await this.programs.margin.account.marginAccount.fetchNullable(this.address)
    this.positions = this.info ? AccountPositionListLayout.decode(new Uint8Array(this.info.positions)) : null
  }

  static async exists(programs: MarginPrograms, owner: Address, seed: number): Promise<boolean> {
    const ownerPubkey = translateAddress(owner)
    const { marginAccount } = this.derive(programs, ownerPubkey, seed)
    const info = await programs.margin.provider.connection.getAccountInfo(marginAccount)
    return !!info
  }

  async exists(): Promise<boolean> {
    return await MarginAccount.exists(this.programs, this.owner, this.seed)
  }

  async createAccount() {
    const ix: TransactionInstruction[] = []
    await this.withCreateAccount(ix)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to create the account
  async withCreateAccount(instructions: TransactionInstruction[]): Promise<void> {
    if (!(await this.exists())) {
      const ix = await this.programs.margin.methods
        .createAccount(this.seed)
        .accounts({
          owner: this.owner,
          payer: this.provider.wallet.publicKey,
          marginAccount: this.address,
          systemProgram: SystemProgram.programId
        })
        .instruction()
      instructions.push(ix)
    }
  }

  //Deposit
  /// Transaction to deposit tokens into a margin account
  ///
  /// # Params
  ///
  /// `token_mint` - The address of the mint for the tokens being deposited
  /// `source` - The token account that the deposit will be transfered from
  /// `amount` - The amount of tokens to deposit
  async deposit(marginPool: Pool, source: Address, amount: BN) {
    await this.refresh()
    const position = await this.getOrCreatePosition(marginPool.addresses.depositNoteMint)
    assert(position)

    const ix: TransactionInstruction[] = []
    await marginPool.withDeposit({
      instructions: ix,
      depositor: this.owner,
      source,
      destination: position.address,
      amount
    })
    await this.withUpdatePositionBalance(ix, position.address)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
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
    const ix: TransactionInstruction[] = []
    await this.withUpdatePositionBalance(ix, account)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to update the accounting for assets in
  /// the custody of the margin account.
  ///
  /// # Params
  ///
  /// `account` - The account address that has had a balance change
  async withUpdatePositionBalance(instructions: TransactionInstruction[], account: PublicKey): Promise<void> {
    const ix = await this.programs.margin.methods
      .updatePositionBalance()
      .accounts({
        marginAccount: this.address,
        tokenAccount: account
      })
      .instruction()
    instructions.push(ix)
  }

  async registerPosition(tokenMint: Address): Promise<TransactionSignature> {
    const tokenMintAddress = translateAddress(tokenMint)
    const ix: TransactionInstruction[] = []
    const tokenAccount = await this.withRegisterPosition(ix, tokenMintAddress)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
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
  async withRegisterPosition(instructions: TransactionInstruction[], tokenMint: Address): Promise<PublicKey> {
    const tokenAccount = findDerivedAccount(this.programs.config.marginProgramId, this.address, tokenMint)
    const metadata = findDerivedAccount(this.programs.config.metadataProgramId, tokenMint)

    const ix = await this.programs.margin.methods
      .registerPosition()
      .accounts({
        authority: this.owner,
        payer: this.provider.wallet.publicKey,
        marginAccount: this.address,
        positionTokenMint: tokenMint,
        metadata,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId
      })
      .instruction()
    instructions.push(ix)
    return tokenAccount
  }

  async closePosition(tokenAccount: PublicKey) {
    const ix: TransactionInstruction[] = []
    await this.withClosePosition(ix, tokenAccount)
    return await this.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  /// Get instruction to close a position
  ///
  /// # Params
  ///
  /// `token_account` - The address of the token account for the position being closed
  async withClosePosition(instructions: TransactionInstruction[], tokenAccount: PublicKey): Promise<void> {
    const authority = findDerivedAccount(this.programs.config.controlProgramId)

    const ix = await this.programs.margin.methods
      .closePosition()
      .accounts({
        authority: authority,
        receiver: this.provider.wallet.publicKey,
        marginAccount: this.address,
        tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)
  }

  static async getTokenAccountInfo(connection: Connection, address: PublicKey) {
    const info = await connection.getAccountInfo(address)
    return AccountLayout.decode(Buffer.from(info!.data))
  }
}
