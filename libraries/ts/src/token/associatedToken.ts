import { BN, Address, translateAddress, AnchorProvider, Provider } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID } from "@project-serum/serum/lib/token-instructions"
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
  MintLayout
} from "@solana/spl-token"
import { Connection, PublicKey, TransactionInstruction, SystemProgram, AccountInfo } from "@solana/web3.js"
import { Number192 } from "../utils/number192"
import { findDerivedAccount } from "../utils/pda"
import { TokenAmount } from "./tokenAmount"

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
      throw new Error("Unexpected owner of the associated token")
    }
    return token
  }

  static async exists(connection: Connection, mint: Address, owner: Address) {
    const mintAddress = translateAddress(mint)
    const ownerAddress = translateAddress(owner)
    const address = this.derive(mintAddress, ownerAddress)
    const account = await connection.getAccountInfo(address)
    return !!account
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

  /** Loads multiple token accounts, loads wrapped SOL. */
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
   * If a mints array is not provided, loads all associated token accounts and the SOL balance of the owner. */
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
      accountInfos = await connection.getMultipleAccountsInfo(addresses)
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
      const mintInfos = await connection.getMultipleAccountsInfo(mints.map(translateAddress))
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

    const accountInfos = await connection.getMultipleAccountsInfo(pubkeys)
    if (decimals === undefined) {
      decimals = []
      const mintPubkeys = accountInfos.map(acc => {
        return (acc && AccountLayout.decode(acc.data).mint) ?? PublicKey.default
      })
      const mintInfos = await connection.getMultipleAccountsInfo(mintPubkeys)
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
   * If the native wrapped token account does not exist, add instruction to create the token account. If ATA exists, do nothing.
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {Provider} provider
   * @param {Address} owner
   * @returns {Promise<PublicKey>} returns the public key of the token account
   * @memberof AssociatedToken
   */
  static async withCreateNative(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: Address
  ): Promise<PublicKey> {
    const ownerAddress = translateAddress(owner)
    const tokenAddress = this.derive(NATIVE_MINT, ownerAddress)

    if (!(await AssociatedToken.exists(provider.connection, NATIVE_MINT, ownerAddress))) {
      const ix = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        tokenAddress,
        ownerAddress,
        NATIVE_MINT
      )
      instructions.push(ix)
    }
    return tokenAddress
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

  /** Wraps SOL in an associated token account. The account will only be created if it doesn't exist.
   * @param instructions
   * @param provider
   * @param owner
   * @param amount
   */
  static async withWrapNative(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    amount: BN
  ): Promise<PublicKey> {
    const owner = translateAddress(provider.wallet.publicKey)

    //this will add instructions to create ata if ata does not exist, if exist, we will get the ata address
    const associatedToken = await this.withCreate(instructions, provider, owner, NATIVE_MINT)
    //IX to transfer sol to ATA
    const transferIx = SystemProgram.transfer({
      fromPubkey: owner,
      lamports: bnToNumber(amount),
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
   * @param tokenAccountOrNative
   * @param amount
   */
  static async withWrapIfNativeMint(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    mint: Address,
    tokenAccountOrNative: Address,
    amount: BN
  ): Promise<PublicKey> {
    const owner = provider.wallet.publicKey
    const mintPubkey = translateAddress(mint)
    const tokenAccountOrNativePubkey = translateAddress(tokenAccountOrNative)

    //only run if mint is wrapped sol mint, and the token account is actually the native wallet
    if (this.isNative(owner, mintPubkey, tokenAccountOrNativePubkey)) {
      return this.withWrapNative(instructions, provider, amount)
    }
    return tokenAccountOrNativePubkey
  }

  /**
   * Unwraps all SOL if the mint is native and the tokenAccount is the owner
   *
   * @param {TransactionInstruction[]} instructions
   * @param {owner} owner
   * @param {mint} mint
   * @param {tokenAccount} tokenAccountOrNative
   */
  static withUnwrapIfNative(
    instructions: TransactionInstruction[],
    owner: Address,
    mint: Address,
    tokenAccountOrNative: Address
  ): void {
    const ownerPubkey = translateAddress(owner)
    const mintPubkey = translateAddress(mint)
    const tokenAccountOrNativePubkey = translateAddress(tokenAccountOrNative)

    if (this.isNative(ownerPubkey, mintPubkey, tokenAccountOrNativePubkey)) {
      //add close account IX
      this.withUnwrapNative(instructions, owner)
    }
  }

  /**
   * Create the associated token account. Funds it if natve.
   *
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {AnchorProvider} provider
   * @param {Address} mint
   * @param {BN} initialAmount
   * @memberof AssociatedToken
   */
  static async withCreateOrWrapIfNativeMint(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    mint: Address,
    initialAmount: BN
  ): Promise<PublicKey> {
    const owner = provider.wallet.publicKey
    const mintPubkey = translateAddress(mint)

    if (mintPubkey.equals(NATIVE_MINT)) {
      // Only run if mint is wrapped sol mint. Create the wrapped sol account and return its pubkey
      return await this.withWrapNative(instructions, provider, initialAmount)
    } else {
      // Return the associated token
      return this.withCreate(instructions, provider, owner, mint)
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

    const associatedToken = await this.withCreate(preInstructions, provider, owner, NATIVE_MINT)

    if (mintPubkey.equals(NATIVE_MINT)) {
      // Only run if mint is wrapped sol mint. Create the wrapped sol account and return its pubkey
      this.withUnwrapNative(postInstructions, owner)
    }

    return associatedToken
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
  return bigInt ? new BN(bigInt.toString()) : Number192.ZERO
}

/** Convert BigInt (SPL Token) to BN. */
export const bigIntToNumber = (bigint: bigint | null | undefined): number => {
  return bigint ? Number(bigint) : 0
}

export function numberToBigInt(number: number | null | undefined) {
  // Stomp out any fraction component of the number
  return number !== null && number !== undefined
    ? BigInt(number.toLocaleString("fullwide", { useGrouping: false, maximumFractionDigits: 0 }))
    : BigInt(0)
}
