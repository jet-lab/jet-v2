import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js"
import { findDerivedAccount } from "../../utils/pda"
import { MarginPoolConfig, MarginPools, MarginTokenConfig } from "../config"
import { MarginPrograms } from "../marginClient"
import { MarginPoolAddresses, Pool, TokenKind } from "./pool"
import { MarginPoolConfigData } from "./state"

interface TokenMetadataParams {
  tokenKind: TokenKind
  collateralWeight: number
  maxLeverage: number
}

interface MarginPoolParams {
  feeDestination: PublicKey
}

interface IPoolCreationParams {
  tokenMint: Address
  collateralWeight: number
  maxLeverage: number
  feeDestination: Address
  pythProduct: Address
  pythPrice: Address
  marginPoolConfig: MarginPoolConfigData
  provider?: AnchorProvider
  programs?: MarginPrograms
}

/**
 * Class that allows the creation and management of margin pools.
 */
export class PoolManager {
  owner: PublicKey

  constructor(public programs: MarginPrograms, public provider: AnchorProvider) {
    this.owner = provider.wallet.publicKey
  }

  /**
   * Load a margin pool
   *
   * @param {{
   *     tokenMint: Address
   *     poolConfig?: MarginPoolConfig
   *     tokenConfig?: MarginTokenConfig
   *     programs?: MarginPrograms
   *   }}
   * @return {Promise<Pool>}
   * @memberof PoolManager
   */
  async load({
    tokenMint,
    poolConfig,
    tokenConfig,
    programs = this.programs
  }: {
    tokenMint: Address
    poolConfig?: MarginPoolConfig
    tokenConfig?: MarginTokenConfig
    programs?: MarginPrograms
  }): Promise<Pool> {
    const addresses = this._derive({ programs: programs, tokenMint })
    const marginPool = new Pool(programs, tokenMint, addresses, poolConfig, tokenConfig)
    await marginPool.refresh()
    return marginPool
  }

  /**
   * Loads all margin pools bases on the config provided to the manager
   *
   * @param {MarginPrograms} [programs=this.programs]
   * @return {Promise<Record<MarginPools, Pool>>}
   * @memberof PoolManager
   */
  async loadAll(programs: MarginPrograms = this.programs): Promise<Record<MarginPools, Pool>> {
    // FIXME: This could be faster with fewer round trips to rpc
    const pools: Record<string, Pool> = {}
    for (const poolConfig of Object.values(programs.config.pools)) {
      const tokenConfig: MarginTokenConfig | undefined = programs.config.tokens[poolConfig.symbol]
      if (tokenConfig) {
        const pool = await this.load({
          tokenMint: poolConfig.tokenMint,
          poolConfig,
          tokenConfig
        })
        pools[poolConfig.symbol] = pool
      }
    }
    return pools
  }

  setProvider(provider: AnchorProvider) {
    this.provider = provider
  }

  setPrograms(programs: MarginPrograms) {
    this.programs = programs
  }

  /**
   * Creates a margin pool
   * @param args  // TODO document interface
   * @returns
   */
  async create({
    tokenMint,
    collateralWeight,
    maxLeverage,
    feeDestination,
    pythProduct,
    pythPrice,
    marginPoolConfig,
    provider = this.provider,
    programs = this.programs
  }: IPoolCreationParams) {
    const addresses = this._derive({ programs: programs, tokenMint })
    const address = addresses.marginPool
    const ix1: TransactionInstruction[] = []
    if (this.owner) {
      try {
        await this.withRegisterToken({
          instructions: ix1,
          requester: this.owner,
          addresses,
          address
        })
        await provider.sendAndConfirm(new Transaction().add(...ix1))
        const ix2: TransactionInstruction[] = []
        await this.withConfigureToken({
          instructions: ix2,
          requester: this.owner,
          collateralWeight,
          maxLeverage,
          feeDestination,
          pythProduct,
          pythPrice,
          marginPoolConfig,
          addresses,
          address: addresses.marginPool
        })

        return await provider.sendAndConfirm(new Transaction().add(...ix2))
      } catch (err) {
        console.log(err)
        throw err
      }
    } else {
      throw new Error("No owner keypair provided")
    }
  }
  /**
   * // TODO add description
   * @param instructions
   * @param requester
   * @param addresses
   * @param address
   */
  async withRegisterToken({
    instructions,
    requester,
    addresses,
    address,
    programs = this.programs
  }: {
    instructions: TransactionInstruction[]
    requester: Address
    addresses: MarginPoolAddresses
    address: PublicKey
    programs?: MarginPrograms
  }): Promise<void> {
    const authority = findDerivedAccount(programs.config.controlProgramId)

    const ix = await programs.control.methods
      .registerToken()
      .accounts({
        requester,
        authority,
        marginPool: address,
        vault: addresses.vault,
        depositNoteMint: addresses.depositNoteMint,
        loanNoteMint: addresses.loanNoteMint,
        tokenMint: addresses.tokenMint,
        tokenMetadata: addresses.tokenMetadata,
        depositNoteMetadata: addresses.depositNoteMetadata,
        loanNoteMetadata: addresses.loanNoteMetadata,
        marginPoolProgram: programs.config.marginPoolProgramId,
        metadataProgram: programs.config.metadataProgramId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      })
      .instruction()
    instructions.push(ix)
  }

  /**
   * Create a margin pool by configuring the token with the control program.
   *
   * # Instructions
   *
   * - jet_control::configure_token - configures an SPL token and creates its pool
   * @param instructions
   * @param requester
   * @param collateralWeight
   * @param maxLeverage
   * @param feeDestination
   * @param pythProduct
   * @param pythPrice
   * @param marginPoolConfig
   * @param addresses
   * @param address
   */
  async withConfigureToken({
    instructions,
    requester,
    collateralWeight,
    maxLeverage,
    feeDestination,
    pythProduct,
    pythPrice,
    marginPoolConfig,
    addresses,
    address,
    programs = this.programs
  }: {
    instructions: TransactionInstruction[]
    requester: Address
    collateralWeight: number
    maxLeverage: number
    feeDestination: Address
    pythProduct: Address
    pythPrice: Address
    marginPoolConfig: MarginPoolConfigData
    addresses: MarginPoolAddresses
    address: PublicKey
    programs?: MarginPrograms
  }): Promise<void> {
    // Set the token configuration, e.g. collateral weight
    const metadata: TokenMetadataParams = {
      tokenKind: { collateral: {} },
      collateralWeight: collateralWeight,
      maxLeverage: maxLeverage
    }
    const poolParam: MarginPoolParams = {
      feeDestination: translateAddress(feeDestination)
    }

    const ix = await programs.control.methods
      .configureToken(
        {
          tokenKind: metadata.tokenKind as never,
          collateralWeight: metadata.collateralWeight,
          maxLeverage
        },
        poolParam,
        marginPoolConfig
      )
      .accounts({
        requester,
        authority: addresses.controlAuthority,
        tokenMint: addresses.tokenMint,
        marginPool: address,
        tokenMetadata: addresses.tokenMetadata,
        depositMetadata: addresses.depositNoteMetadata,
        loanMetadata: addresses.loanNoteMetadata,
        pythProduct: pythProduct,
        pythPrice: pythPrice,
        marginPoolProgram: programs.config.marginPoolProgramId,
        metadataProgram: programs.config.metadataProgramId
      })
      .instruction()
    instructions.push(ix)
  }

  /**
   * Derive accounts from tokenMint
   * @param {MarginPrograms} programs
   * @param {Address} tokenMint
   * @returns {PublicKey} Margin Pool Address
   */
  private _derive({ programs, tokenMint }: { programs: MarginPrograms; tokenMint: Address }): MarginPoolAddresses {
    const tokenMintAddress = translateAddress(tokenMint)
    const programId = translateAddress(programs.config.marginPoolProgramId)
    const marginPool = findDerivedAccount(programId, tokenMintAddress)
    const vault = findDerivedAccount(programId, marginPool, "vault")
    const depositNoteMint = findDerivedAccount(programId, marginPool, "deposit-notes")
    const loanNoteMint = findDerivedAccount(programId, marginPool, "loan-notes")
    const marginPoolAdapterMetadata = findDerivedAccount(programs.config.metadataProgramId, programId)
    const tokenMetadata = findDerivedAccount(programs.config.metadataProgramId, tokenMint)
    const depositNoteMetadata = findDerivedAccount(programs.config.metadataProgramId, depositNoteMint)
    const loanNoteMetadata = findDerivedAccount(programs.config.metadataProgramId, loanNoteMint)
    const controlAuthority = findDerivedAccount(programs.config.controlProgramId)

    return {
      tokenMint: tokenMintAddress,
      marginPool,
      vault,
      depositNoteMint,
      loanNoteMint,
      marginPoolAdapterMetadata,
      tokenMetadata,
      depositNoteMetadata,
      loanNoteMetadata,
      controlAuthority
    }
  }
}
