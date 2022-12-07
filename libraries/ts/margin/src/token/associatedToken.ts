import { BN, Address, translateAddress, AnchorProvider } from "@project-serum/anchor"
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  createAssociatedTokenAccountInstruction,
  createCloseAccountInstruction,
  createSyncNativeInstruction,
  Mint,
  Account,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError,
  TokenInvalidAccountSizeError,
  ACCOUNT_SIZE,
  AccountLayout,
  AccountState,
  MINT_SIZE,
  MintLayout,
  TokenInvalidOwnerError,
  createInitializeAccountInstruction,
  getMinimumBalanceForRentExemptAccount,
  TokenInvalidMintError,
  createTransferInstruction,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token"
import { Connection, PublicKey, TransactionInstruction, SystemProgram, AccountInfo } from "@solana/web3.js"
import { chunks } from "../utils"
import { findDerivedAccount } from "../utils/pda"
import { TokenAmount } from "./tokenAmount"

export type TokenAddress = Address | TokenFormat

export enum TokenFormat {
  /** The users associated token account will be used, and sol will be unwrapped. */
  unwrappedSol,
  /** The users associated token account will be used, and sol will be wrapped. */
  wrappedSol
}

export class AssociatedToken {
  static readonly NATIVE_DECIMALS = 9
  exists: boolean
  /**
   * Get the address for the associated token account
   * @static
   * @param {Address} mint Token mint account
   * @param {Address} owner Owner of the new account
   * @returns {Promise<PublicKey>} Public key of the associated token account
   * @memberof AssociatedToken
   */
  static derive(mint: Address, owner: Address): PublicKey {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)
    return findDerivedAccount(ASSOCIATED_TOKEN_PROGRAM_ID, ownerAddress, TOKEN_PROGRAM_ID, mintAddress)
  }

  /**
   * TODO:
   * @static
   * @param {Connection} connection
   * @param {Address} mint
   * @param {Address} owner
   * @param {number} decimals
   * @returns {(Promise<AssociatedToken>)}
   * @memberof AssociatedToken
   */
  static async load({
    connection,
    mint,
    owner,
    decimals
  }: {
    connection: Connection
    mint: Address
    owner: Address
    decimals: number
  }): Promise<AssociatedToken> {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)
    const address = this.derive(mintAddress, ownerAddress)
    const token = await this.loadAux(connection, address, decimals)
    if (token.info && !token.info.owner.equals(ownerAddress)) {
      throw new TokenInvalidOwnerError("The owner of a token account doesn't match the expected owner")
    }
    return token
  }

  static async exists(connection: Connection, mint: Address, owner: Address) {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)
    const address = this.derive(mintAddress, ownerAddress)
    return await AssociatedToken.existsAux(connection, mint, owner, address)
  }

  static async existsAux(connection: Connection, mint: Address, owner: Address, address: Address) {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)
    const tokenAddress = translateAddress(address)
    const info = await connection.getAccountInfo(tokenAddress)

    if (info) {
      const fakeDecimals = 0
      const account = this.decodeAccount(info, address, fakeDecimals)
      if (!account.info) {
        throw new TokenInvalidAccountSizeError()
      }
      if (!account.info.owner.equals(ownerAddress)) {
        throw new TokenInvalidOwnerError("The owner of a token account doesn't match the expected owner")
      }
      if (!account.info.mint.equals(mintAddress)) {
        throw new TokenInvalidMintError("The mint of a token account doesn't match the expected mint")
      }
      return true
    }
    return false
  }

  static async loadAux(connection: Connection, address: Address, decimals: number) {
    const pubkey = translateAddress(address)
    const account = await connection.getAccountInfo(pubkey)
    return AssociatedToken.decodeAccount(account, pubkey, decimals)
  }

  static zero(mint: Address, owner: Address, decimals: number) {
    const address = this.derive(mint, owner)
    return this.zeroAux(address, decimals)
  }

  static zeroAux(address: Address, decimals: number) {
    const pubkey = translateAddress(address)
    const info = null
    const amount = TokenAmount.zero(decimals)
    return new AssociatedToken(pubkey, info, amount)
  }

  /** Loads multiple token accounts, loads wrapped SOL. Batches by 100 (RPC limit) */
  static async loadMultiple({
    connection,
    mints,
    decimals,
    owner
  }: {
    connection: Connection
    mints: Address[]
    decimals: number | number[]
    owner: Address
  }): Promise<AssociatedToken[]> {
    const addresses: PublicKey[] = []
    for (let i = 0; i < mints.length; i++) {
      const mint = mints[i]
      addresses.push(AssociatedToken.derive(mint, owner))
    }

    return await this.loadMultipleAux({ connection, addresses, decimals })
  }

  /**
   * Loads multiple associated token accounts by owner.
   * If the native mint is provided, loads the native SOL balance of the owner instead.
   * If a mints array is not provided, loads all associated token accounts and the SOL balance of the owner.
   * Batches by 100 (RPC limit) */
  static async loadMultipleOrNative({
    connection,
    owner,
    mints,
    decimals
  }: {
    connection: Connection
    owner: Address
    mints?: Address[]
    decimals?: number | number[]
  }): Promise<AssociatedToken[]> {
    if (Array.isArray(decimals) && mints !== undefined && decimals.length !== mints.length) {
      throw new Error("Decimals array length does not equal mints array length")
    }
    const ownerAddress = translateAddress(owner)

    let addresses: PublicKey[]
    let accountInfos: (AccountInfo<Buffer> | null)[] | undefined
    if (mints) {
      addresses = []
      const mintAddresses = mints.map(translateAddress)
      for (let i = 0; i < mintAddresses.length; i++) {
        const mint = mintAddresses[i]
        if (mint.equals(NATIVE_MINT)) {
          // Load the owner and read their SOL balance
          addresses.push(ownerAddress)
        } else {
          // Load the token account
          addresses.push(AssociatedToken.derive(mint, ownerAddress))
        }
      }
      accountInfos = await AssociatedToken.loadMultipleAccountsInfoBatched(connection, addresses)
    } else {
      const { value } = await connection.getTokenAccountsByOwner(ownerAddress, { programId: TOKEN_PROGRAM_ID })
      accountInfos = value.map(acc => acc.account)
      addresses = value.map(acc => acc.pubkey)
      mints = accountInfos.map(acc => (acc && AccountLayout.decode(acc.data).mint) ?? PublicKey.default)

      // Add the users native SOL account
      const emptyOwnerNativeAccount = {
        data: Buffer.alloc(0),
        executable: false,
        owner: SystemProgram.programId,
        lamports: 0
      }
      const ownerAccount = (await connection.getAccountInfo(ownerAddress)) ?? emptyOwnerNativeAccount
      accountInfos.push(ownerAccount)
      addresses.push(ownerAddress)
      mints.push(NATIVE_MINT)
    }

    if (decimals === undefined) {
      decimals = []
      const mintInfos = await AssociatedToken.loadMultipleAccountsInfoBatched(connection, mints.map(translateAddress))
      for (let i = 0; i < mintInfos.length; i++) {
        const mintInfo = mintInfos[i]
        if (translateAddress(mints[i]).equals(NATIVE_MINT)) {
          decimals.push(this.NATIVE_DECIMALS)
        } else if (translateAddress(mints[i]).equals(PublicKey.default)) {
          decimals.push(0)
        } else if (mintInfo === null) {
          decimals.push(0)
        } else {
          const mint = MintLayout.decode(mintInfo.data)
          decimals.push(mint.decimals)
        }
      }
    }

    const accounts: AssociatedToken[] = []
    for (let i = 0; i < mints.length; i++) {
      const mint = translateAddress(mints[i])
      const address = addresses[i]
      const decimal = Array.isArray(decimals) ? decimals[i] : decimals
      const info = accountInfos[i]
      const associatedTokenAddress = AssociatedToken.derive(mint, ownerAddress)

      const isAssociatedtoken = associatedTokenAddress.equals(address)
      const isNative = mint.equals(NATIVE_MINT) && address.equals(ownerAddress)

      // Exlude non-associated token accounts and unwrapped wallet balances
      if (!isAssociatedtoken && !isNative) {
        continue
      }

      if (isNative) {
        // Load the owner and read their SOL balance
        accounts.push(AssociatedToken.decodeNative(info, address))
      } else {
        // Load the token account
        accounts.push(AssociatedToken.decodeAccount(info, address, decimal))
      }
    }
    return accounts
  }

  /**
   * Loads multiple token accounts and their mints by address.
   * Batches by 100 (RPC limit) */
  static async loadMultipleAux({
    connection,
    addresses,
    decimals
  }: {
    connection: Connection
    addresses: Address[]
    decimals?: number | number[]
  }): Promise<AssociatedToken[]> {
    if (Array.isArray(decimals) && decimals.length !== addresses.length) {
      throw new Error("Decimals array length does not equal addresses array length")
    }

    const pubkeys = addresses.map(address => translateAddress(address))

    const accountInfos = await AssociatedToken.loadMultipleAccountsInfoBatched(connection, pubkeys)
    if (decimals === undefined) {
      decimals = []
      const mintPubkeys = accountInfos.map(acc => {
        return (acc && AccountLayout.decode(acc.data).mint) ?? PublicKey.default
      })
      const mintInfos = await AssociatedToken.loadMultipleAccountsInfoBatched(connection, mintPubkeys)
      for (let i = 0; i < mintInfos.length; i++) {
        const mintInfo = mintInfos[i]
        if (mintPubkeys[i].equals(PublicKey.default)) {
          decimals.push(0)
        } else if (mintInfo === null) {
          decimals.push(0)
        } else {
          const mint = MintLayout.decode(mintInfo.data)
          decimals.push(mint.decimals)
        }
      }
    }

    const accounts: AssociatedToken[] = []
    for (let i = 0; i < pubkeys.length; i++) {
      const decimal = Array.isArray(decimals) ? decimals[i] : decimals
      const account = AssociatedToken.decodeAccount(accountInfos[i], pubkeys[i], decimal)
      accounts.push(account)
    }
    return accounts
  }

  /**
   * Fetch all the account info for multiple accounts specified by an array of public keys.
   *
   * @private
   * @static
   * @param {Connection} connection The connection used to fetch.
   * @param {PublicKey[]} publicKeys The accounts to fetch.
   * @param {number} [batchSize=100] The maximum batch size. Default 100, which is an RPC limit.
   * @return {(Promise<(AccountInfo<Buffer> | null)[]>)} An array of accounts returned in the same order as the publicKeys array
   * @memberof AssociatedToken
   */
  private static async loadMultipleAccountsInfoBatched(
    connection: Connection,
    publicKeys: PublicKey[],
    batchSize: number = 100
  ): Promise<(AccountInfo<Buffer> | null)[]> {
    const batches = chunks(batchSize, publicKeys)
    const promises = batches.map(batch => connection.getMultipleAccountsInfo(batch))
    const infos = await Promise.all(promises)
    return infos.flat(1)
  }

  /** TODO:
   * Get mint info
   * @static
   * @param {Provider} connection
   * @param {Address} mint
   * @returns {(Promise<Mint | undefined>)}
   * @memberof AssociatedToken
   */
  static async loadMint(connection: Connection, mint: Address): Promise<Mint | undefined> {
    const mintAddress = translateAddress(mint)
    const mintInfo = await connection.getAccountInfo(mintAddress)
    if (!mintInfo) {
      return undefined
    }
    return AssociatedToken.decodeMint(mintInfo, mintAddress)
  }

  /**
   * Creates an instance of AssociatedToken.
   *
   * @param {PublicKey} address
   * @param {Account | null} info
   * @param {TokenAmount} amount
   * @memberof AssociatedToken
   */
  constructor(public address: PublicKey, public info: Account | null, public amount: TokenAmount) {
    this.exists = !!info
  }

  /**
   * Decode a token account. From @solana/spl-token
   * @param {AccountInfo<Buffer>} inifo
   * @param {PublicKey} address
   * @returns
   */
  static decodeAccount(data: AccountInfo<Buffer> | null, address: Address, decimals: number) {
    const publicKey = translateAddress(address)
    if (!data) {
      return AssociatedToken.zeroAux(publicKey, decimals)
    }

    if (data && !data.owner.equals(TOKEN_PROGRAM_ID)) throw new TokenInvalidAccountOwnerError()
    if (data && data.data.length != ACCOUNT_SIZE) throw new TokenInvalidAccountSizeError()

    const rawAccount = AccountLayout.decode(data.data)

    const info = {
      address: publicKey,
      mint: rawAccount.mint,
      owner: rawAccount.owner,
      amount: rawAccount.amount,
      delegate: rawAccount.delegateOption ? rawAccount.delegate : null,
      delegatedAmount: rawAccount.delegatedAmount,
      isInitialized: rawAccount.state !== AccountState.Uninitialized,
      isFrozen: rawAccount.state === AccountState.Frozen,
      isNative: !!rawAccount.isNativeOption,
      rentExemptReserve: rawAccount.isNativeOption ? rawAccount.isNative : null,
      closeAuthority: rawAccount.closeAuthorityOption ? rawAccount.closeAuthority : null
    }
    return new AssociatedToken(publicKey, info, TokenAmount.account(info, decimals))
  }

  /**
   * Decode a mint account
   * @param {AccountInfo<Buffer>} info
   * @param {PublicKey} address
   * @returns {Mint}
   */
  static decodeMint(info: AccountInfo<Buffer>, address: PublicKey): Mint {
    if (!info) throw new TokenAccountNotFoundError()
    if (!info.owner.equals(TOKEN_PROGRAM_ID)) throw new TokenInvalidAccountOwnerError()
    if (info.data.length != MINT_SIZE) throw new TokenInvalidAccountSizeError()

    const rawMint = MintLayout.decode(info.data)

    return {
      address,
      mintAuthority: rawMint.mintAuthorityOption ? rawMint.mintAuthority : null,
      supply: rawMint.supply,
      decimals: rawMint.decimals,
      isInitialized: rawMint.isInitialized,
      freezeAuthority: rawMint.freezeAuthorityOption ? rawMint.freezeAuthority : null
    }
  }

  /**
   * Decode a token account. From @solana/spl-token
   * @param {AccountInfo<Buffer>} inifo
   * @param {PublicKey} address
   * @returns
   */
  static decodeNative(info: AccountInfo<Buffer> | null, address: Address) {
    const publicKey = translateAddress(address)
    if (info && info.data.length != 0) throw new TokenInvalidAccountSizeError()

    return new AssociatedToken(
      publicKey,
      null,
      TokenAmount.lamports(new BN(info?.lamports.toString() ?? "0"), this.NATIVE_DECIMALS)
    )
  }

  /**
   * Returns true when the mint is native and the token account is actually the native wallet
   *
   * @static
   * @param {Address} owner
   * @param {Address} mint
   * @param {Address} tokenAccountOrNative
   * @return {boolean}
   * @memberof AssociatedToken
   */
  static isNative(owner: Address, mint: Address, tokenAccountOrNative: Address): boolean {
    const ownerPubkey = translateAddress(owner)
    const mintPubkey = translateAddress(mint)
    const tokenAccountOrNativePubkey = translateAddress(tokenAccountOrNative)

    return mintPubkey.equals(NATIVE_MINT) && tokenAccountOrNativePubkey.equals(ownerPubkey)
  }

  /**
   * If the associated token account does not exist for this mint, add instruction to create the token account.If ATA exists, do nothing.
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {Provider} provider
   * @param {Address} owner
   * @param {Address} mint
   * @returns {Promise<PublicKey>} returns the public key of the token account
   * @memberof AssociatedToken
   */
  static async withCreate(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: Address,
    mint: Address
  ): Promise<PublicKey> {
    const ownerAddress = translateAddress(owner)
    const mintAddress = translateAddress(mint)
    const tokenAddress = this.derive(mintAddress, ownerAddress)

    if (!(await AssociatedToken.exists(provider.connection, mintAddress, ownerAddress))) {
      const ix = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        tokenAddress,
        ownerAddress,
        mintAddress
      )
      instructions.push(ix)
    }
    return tokenAddress
  }

  /**
   * If the token account does not exist, add instructions to create and initialize the token account. If the account exists do nothing.
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {Provider} provider
   * @param {Address} owner
   * @param {Address} mint
   * @returns {Promise<PublicKey>} returns the public key of the token account
   * @memberof AssociatedToken
   */
  static async withCreateAux(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: Address,
    mint: Address,
    address: Address
  ): Promise<void> {
    const ownerAddress = translateAddress(owner)
    const mintAddress = translateAddress(mint)
    const tokenAddress = translateAddress(address)

    if (!(await AssociatedToken.existsAux(provider.connection, mintAddress, ownerAddress, address))) {
      let rent = await getMinimumBalanceForRentExemptAccount(provider.connection)
      let createIx = SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: tokenAddress,
        lamports: rent,
        space: ACCOUNT_SIZE,
        programId: TOKEN_PROGRAM_ID
      })
      let initIx = createInitializeAccountInstruction(tokenAddress, mintAddress, ownerAddress)
      instructions.push(createIx, initIx)
    }
  }

  /**
   * Add close associated token account IX
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {Address} owner
   * @param {Address} mint
   * @param {Address} rentDestination
   * @memberof AssociatedToken
   */
  static withClose(instructions: TransactionInstruction[], owner: Address, mint: Address, rentDestination: Address) {
    const ownerPubkey = translateAddress(owner)
    const mintPubkey = translateAddress(mint)
    const rentDestinationPubkey = translateAddress(rentDestination)

    const tokenAddress = this.derive(mintPubkey, ownerPubkey)
    const ix = createCloseAccountInstruction(tokenAddress, rentDestinationPubkey, ownerPubkey)
    instructions.push(ix)
  }

  /**
   * Transfer tokens from one owner to another
   * @param instructions
   * @param mint
   * @param sourceOwner
   * @param destinationOwner
   * @param amount
   */
  static withTransfer(
    instructions: TransactionInstruction[],
    mint: Address,
    sourceOwner: Address,
    destinationOwner: Address,
    amount: BN
  ) {
    const authority = translateAddress(sourceOwner)
    const source = this.derive(mint, sourceOwner)
    const destination = this.derive(mint, destinationOwner)

    instructions.push(createTransferInstruction(source, destination, authority, amount.toNumber()))
  }

  /** Wraps SOL in an associated token account. The account will only be created if it doesn't exist.
   * @param instructions
   * @param provider
   * @param {BN} feesBuffer How much tokens should remain unwrapped to pay for fees
   */
  static async withWrapNative(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    feesBuffer: BN
  ): Promise<PublicKey> {
    const owner = translateAddress(provider.wallet.publicKey)
    const ownerInfo = await provider.connection.getAccountInfo(owner)
    const ownerLamports = BN.max(new BN(ownerInfo?.lamports ?? 0).sub(feesBuffer), new BN(0))

    //this will add instructions to create ata if ata does not exist, if exist, we will get the ata address
    const associatedToken = await this.withCreate(instructions, provider, owner, NATIVE_MINT)
    //IX to transfer sol to ATA

    const transferIx = SystemProgram.transfer({
      fromPubkey: owner,
      lamports: BigInt(ownerLamports.toString()),
      toPubkey: associatedToken
    })

    const syncNativeIX = createSyncNativeInstruction(associatedToken)
    instructions.push(transferIx, syncNativeIX)
    return associatedToken
  }

  /**
   * Unwraps all SOL in the associated token account.
   *
   * @param {TransactionInstruction[]} instructions
   * @param {owner} owner
   */
  static withUnwrapNative(instructions: TransactionInstruction[], owner: Address): void {
    //add close account IX
    this.withClose(instructions, owner, NATIVE_MINT, owner)
  }

  /** Add wrap SOL IX
   * @param instructions
   * @param provider
   * @param mint
   * @param feesBuffer How much tokens should remain unwrapped to pay for fees
   */
  static async withWrapIfNativeMint(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    mint: Address,
    feesBuffer: BN
  ): Promise<PublicKey> {
    const mintPubkey = translateAddress(mint)

    if (mintPubkey.equals(NATIVE_MINT)) {
      //only run if mint is wrapped sol mint
      return this.withWrapNative(instructions, provider, feesBuffer)
    }
    return AssociatedToken.derive(mint, provider.wallet.publicKey)
  }

  /**
   * Unwraps all SOL if the mint is native and the tokenAccount is the owner
   *
   * @param {TransactionInstruction[]} instructions
   * @param {owner} owner
   * @param {mint} mint
   * @param {tokenAccount} tokenAccountOrNative
   */
  static withUnwrapIfNativeMint(instructions: TransactionInstruction[], owner: Address, mint: Address): void {
    const mintPubkey = translateAddress(mint)

    if (mintPubkey.equals(NATIVE_MINT)) {
      //add close account IX
      this.withUnwrapNative(instructions, owner)
    }
  }

  /**
   * Create the associated token account. Funds it if native.
   *
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {AnchorProvider} provider
   * @param {Address} mint
   * @param {number} feesBuffer How much tokens should remain unwrapped to pay for fees
   * @memberof AssociatedToken
   */
  static async withCreateOrWrapIfNativeMint(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    mint: Address,
    feesBuffer: BN
  ): Promise<PublicKey> {
    const owner = provider.wallet.publicKey
    const mintPubkey = translateAddress(mint)

    if (mintPubkey.equals(NATIVE_MINT)) {
      // Only run if mint is wrapped sol mint. Create the wrapped sol account and return its pubkey
      return await this.withWrapNative(instructions, provider, feesBuffer)
    } else {
      // Return the associated token
      return await this.withCreate(instructions, provider, owner, mint)
    }
  }

  /**
   * Create the associated token account as a pre-instruction.
   * Unwraps sol as a post-instruction.
   *
   * @static
   * @param {TransactionInstruction[]} preInstructions
   * @param {TransactionInstruction[]} postInstructions
   * @param {AnchorProvider} provider
   * @param {Address} mint
   * @memberof AssociatedToken
   */
  static async withCreateOrUnwrapIfNativeMint(
    preInstructions: TransactionInstruction[],
    postInstructions: TransactionInstruction[],
    provider: AnchorProvider,
    mint: Address
  ): Promise<PublicKey> {
    const owner = provider.wallet.publicKey
    const mintPubkey = translateAddress(mint)

    const associatedToken = await this.withCreate(preInstructions, provider, owner, mintPubkey)

    if (mintPubkey.equals(NATIVE_MINT)) {
      // Only run if mint is wrapped sol mint. Create the wrapped sol account and return its pubkey
      this.withUnwrapNative(postInstructions, owner)
    }

    return associatedToken
  }

  static async withBeginTransferFromSource({
    instructions,
    provider,
    mint,
    feesBuffer,
    source = TokenFormat.unwrappedSol
  }: {
    instructions: TransactionInstruction[]
    provider: AnchorProvider
    mint: Address
    feesBuffer: BN
    source: Address | TokenFormat
  }): Promise<PublicKey> {
    let sourceAddress: PublicKey | undefined
    // Is destination a PublicKey or string
    if (typeof source === "string" || (typeof source === "object" && "_bn" in source)) {
      sourceAddress = translateAddress(source)
    }
    let owner = provider.wallet.publicKey
    let isSourceOwner = sourceAddress && sourceAddress.equals(owner)
    let isSourceAssociatedAddress = sourceAddress && AssociatedToken.derive(mint, owner).equals(sourceAddress)

    if (source === TokenFormat.unwrappedSol || isSourceOwner || isSourceAssociatedAddress) {
      return await AssociatedToken.withCreateOrWrapIfNativeMint(instructions, provider, mint, feesBuffer)
    } else if (source === TokenFormat.wrappedSol) {
      return await AssociatedToken.withCreate(instructions, provider, owner, mint)
    } else if (sourceAddress) {
      await AssociatedToken.withCreateAux(instructions, provider, owner, mint, sourceAddress)
      return sourceAddress
    }
    throw new Error(
      "Unexpected argument 'source' or there are multiple versions of @solana/web3.js PublicKey installed"
    )
  }

  static async withBeginTransferToDestination({
    instructions,
    provider,
    mint,
    destination = TokenFormat.unwrappedSol
  }: {
    instructions: TransactionInstruction[]
    provider: AnchorProvider
    mint: Address
    destination: Address | TokenFormat
  }): Promise<PublicKey> {
    let destinationAddress: PublicKey | undefined
    // Is destination a PublicKey or string
    if (typeof destination === "string" || (typeof destination === "object" && "_bn" in destination)) {
      destinationAddress = translateAddress(destination)
    }
    let owner = provider.wallet.publicKey
    let isDestinationOwner = destinationAddress && destinationAddress.equals(owner)
    let isDestinationAssociatedAddress =
      destinationAddress && AssociatedToken.derive(mint, owner).equals(destinationAddress)

    if (
      destination === TokenFormat.wrappedSol ||
      destination === TokenFormat.unwrappedSol ||
      isDestinationOwner ||
      isDestinationAssociatedAddress
    ) {
      return await AssociatedToken.withCreate(instructions, provider, owner, mint)
    } else if (destinationAddress) {
      await AssociatedToken.withCreateAux(instructions, provider, owner, mint, destinationAddress)
      return destinationAddress
    }
    throw new Error(
      "Unexpected argument 'destination' or there are multiple versions of @solana/web3.js PublicKey installed"
    )
  }

  /** Ends the transfer by unwraps the token if it is the native mint. */
  static withEndTransfer({
    instructions,
    provider,
    mint,
    destination = TokenFormat.unwrappedSol
  }: {
    instructions: TransactionInstruction[]
    provider: AnchorProvider
    mint: Address
    destination: Address | TokenFormat
  }) {
    let destinationAddress: PublicKey | undefined
    // Is destination a PublicKey or string
    if (typeof destination === "string" || (typeof destination === "object" && "_bn" in destination)) {
      destinationAddress = translateAddress(destination)
    }
    let owner = provider.wallet.publicKey
    let isDestinationOwner = destinationAddress && destinationAddress.equals(owner)

    if (
      translateAddress(mint).equals(NATIVE_MINT) &&
      (destination === TokenFormat.unwrappedSol || isDestinationOwner)
    ) {
      AssociatedToken.withUnwrapNative(instructions, owner)
    }
  }
}

/**
 * Convert number to BN. This never throws for large numbers, unlike the BN constructor.
 * @param {Number} [number]
 * @returns {BN}
 */
export function numberToBn(number: number | null | undefined): BN {
  return new BN(numberToBigInt(number).toString())
}

/**
 * Convert BN to number. This never throws for large numbers, unlike BN.toNumber().
 * @param {BN} [bn]
 * @returns {number}
 */
export function bnToNumber(bn: BN | null | undefined): number {
  return bn ? parseFloat(bn.toString()) : 0
}

/**
 * Convert BigInt (SPL Token) to BN. (Anchor)
 * @param {bigint} [bigInt]
 * @returns {BN}
 */
export const bigIntToBn = (bigInt: bigint | null | undefined): BN => {
  return bigInt ? new BN(bigInt.toString()) : new BN(0)
}

export const bnToBigInt = (bn: BN | null | undefined): bigint => {
  return bn ? BigInt(bn.toString()) : 0n
}

/** Convert BigInt (SPL Token) to BN. */
export const bigIntToNumber = (bigint: bigint | null | undefined): number => {
  return bigint ? Number(bigint) : 0
}

export function numberToBigInt(number: number | null | undefined) {
  // Stomp out any fraction component of the number
  return number !== null && number !== undefined
    ? BigInt(number.toLocaleString("fullwide", { useGrouping: false, maximumFractionDigits: 0 }))
    : 0n
}
