import { BN, Address, translateAddress, AnchorProvider } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID } from "@project-serum/serum/lib/token-instructions"
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  createAssociatedTokenAccountInstruction,
  createCloseAccountInstruction,
  createTransferInstruction,
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
import { findDerivedAccount } from "../utils/pda"
import { TokenAmount } from "./tokenAmount"

export class AssociatedToken {
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
  static async load(connection: Connection, mint: Address, owner: Address, decimals: number): Promise<AssociatedToken> {
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
    if (account) {
      return AssociatedToken.decodeAccount(account, pubkey, decimals)
    } else {
      return AssociatedToken.zeroAux(address, decimals)
    }
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

  static async loadMultiple(
    connection: Connection,
    mints: Address[],
    decimals: number | number[],
    owners: Address | Address[]
  ): Promise<AssociatedToken[]> {
    if (Array.isArray(owners) && owners.length !== mints.length) {
      throw new Error("Owners array length does not equal mints array length")
    }
    if (Array.isArray(decimals) && decimals.length !== mints.length) {
      throw new Error("Decimals array length does not equal mints array length")
    }

    const addresses: PublicKey[] = []
    for (let i = 0; i < mints.length; i++) {
      const mint = mints[i]
      const owner = Array.isArray(owners) ? owners[i] : owners
      addresses.push(AssociatedToken.derive(mint, owner))
    }

    return await this.loadMultipleAux(connection, addresses, decimals)
  }

  static async loadMultipleAux(
    connection: Connection,
    addresses: Address[],
    decimals: number | number[]
  ): Promise<AssociatedToken[]> {
    if (Array.isArray(decimals) && decimals.length !== addresses.length) {
      throw new Error("Decimals array length does not equal addresses array length")
    }

    const pubkeys = addresses.map(address => translateAddress(address))

    const accounts = await connection.getMultipleAccountsInfo(pubkeys)
    return accounts.map((account, i) => {
      const decimal = Array.isArray(decimals) ? decimals[i] : decimals
      return account
        ? AssociatedToken.decodeAccount(account, pubkeys[i], decimal)
        : AssociatedToken.zeroAux(pubkeys[i], decimal)
    })
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
  static decodeAccount(data: AccountInfo<Buffer>, address: Address, decimals: number) {
    const publicKey = translateAddress(address)
    if (!data) throw new TokenAccountNotFoundError()
    if (!data.owner.equals(TOKEN_PROGRAM_ID)) throw new TokenInvalidAccountOwnerError()
    if (data.data.length != ACCOUNT_SIZE) throw new TokenInvalidAccountSizeError()

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
    return new AssociatedToken(publicKey, info, TokenAmount.tokenAccount(info, decimals))
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
   * If the associated token account does not exist for this mint, add instruction to create the token account.If ATA exists, do nothing.
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {Provider} provider
   * @param {PublicKey} owner
   * @param {PublicKey} mint
   * @returns {Promise<PublicKey>} returns the public key of the token account
   * @memberof AssociatedToken
   */
  static async withCreate(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: PublicKey,
    mint: PublicKey
  ): Promise<PublicKey> {
    const tokenAddress = this.derive(mint, owner)

    if (!(await AssociatedToken.exists(provider.connection, mint, owner))) {
      const ix = createAssociatedTokenAccountInstruction(provider.wallet.publicKey, tokenAddress, owner, mint)
      instructions.push(ix)
    }
    return tokenAddress
  }

  /**
   * Add close associated token account IX
   * @static
   * @param {TransactionInstruction[]} instructions
   * @param {PublicKey} owner
   * @param {PublicKey} mint
   * @param {PublicKey} rentDestination
   * @param {Signer[]} [multiSigner=[]]
   * @memberof AssociatedToken
   */
  static async withClose(
    instructions: TransactionInstruction[],
    owner: PublicKey,
    mint: PublicKey,
    rentDestination: PublicKey
  ) {
    const tokenAddress = this.derive(mint, owner)
    const ix = createCloseAccountInstruction(tokenAddress, rentDestination, owner)
    instructions.push(ix)
  }

  /** Add wrap SOL IX
   * @param instructions
   * @param provider
   * @param owner
   * @param mint
   * @param amount
   */
  static async withWrapIfNativeMint(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: PublicKey,
    mint: PublicKey,
    amount: BN
  ): Promise<void> {
    //only run if mint is wrapped sol mint
    if (mint.equals(NATIVE_MINT)) {
      //this will add instructions to create ata if ata does not exist, if exist, we will get the ata address
      const ata = await this.withCreate(instructions, provider, owner, mint)
      //IX to transfer sol to ATA
      const transferIx = SystemProgram.transfer({
        fromPubkey: owner,
        lamports: bnToNumber(amount),
        toPubkey: ata
      })
      const syncNativeIX = createSyncNativeInstruction(ata)
      instructions.push(transferIx, syncNativeIX)
    }
  }

  /**
   * add unWrap SOL IX
   * @param {TransactionInstruction[]} instructions
   * @param {Provider} provider
   * @param {owner} owner
   * @param {tokenAccount} tokenAccount
   * @param {mint} mint
   * @param {amount} amount
   */
  static async withUnwrapIfNative(
    instructions: TransactionInstruction[],
    provider: AnchorProvider,
    owner: PublicKey, //user pubkey
    tokenAccount: PublicKey,
    mint: PublicKey,
    amount: BN
  ): Promise<void> {
    if (mint.equals(NATIVE_MINT)) {
      //create a new ata if ata doesn't not exist
      const ata = await this.withCreate(instructions, provider, owner, mint)
      //IX to transfer wSOL to ATA
      const transferIx = createTransferInstruction(tokenAccount, ata, owner, BigInt(amount.toString()))
      //add transfer IX
      instructions.push(transferIx)
      //add close account IX
      await this.withClose(instructions, owner, mint, owner)
    }
  }
}

/**
 * Convert BN to precise number
 * @param {BN} [bn]
 * @returns {number}
 */
export const bnToNumber = (bn: BN | null | undefined): number => {
  return bn ? parseFloat(bn.toString()) : 0
}

/**
 * Convert BigInt (spl) to BN (Anchor)
 * @param {bigint} [bigInt]
 * @returns {BN}
 */
export const bigIntToBn = (bigInt: bigint | null | undefined): BN => {
  return bigInt ? new BN(bigInt.toString()) : new BN(0)
}

export const bigIntToNumber = (bigint: bigint | null | undefined): number => {
  return bigint ? Number(bigint.toString()) : 0
}
