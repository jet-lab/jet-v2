import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {  PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js"
import { findDerivedAccount } from '../../utils/pda'
import { MarginPoolConfig, MarginTokenConfig, MarginTokens } from "../config"
import { MarginPrograms } from "../marginClient"
import { MarginPoolAddresses, Pool, TokenKind } from "./Pool"
import { MarginPoolConfigData } from "./state"

interface TokenMetadataParams {
  tokenKind: TokenKind
  collateralWeight: number
  collateralMaxStaleness: BN
}

interface MarginPoolParams {
  feeDestination: PublicKey
}

interface IPoolCreationParams {
  tokenMint: Address
  collateralWeight: number
  collateralMaxStaleness: BN
  feeDestination: Address
  pythProduct: Address
  pythPrice: Address
  marginPoolConfig: MarginPoolConfigData
}

/**
 * Class that allows the creation and management of margin pools.
 */
export class PoolManager {
  owner: PublicKey

  constructor(
    public programs: MarginPrograms,
    public provider: AnchorProvider,
  ) {
    this.owner = provider.wallet.publicKey
  }

  /**
   * Load a margin pool
   * @param tokenMint 
   * @param poolConfig 
   * @param tokenConfig 
   * @returns 
   */
  async load(
    tokenMint: Address,
    poolConfig?: MarginPoolConfig,
    tokenConfig?: MarginTokenConfig
  ): Promise<Pool> {
    const addresses = this._derive(this.programs, tokenMint)
    const marginPool = new Pool(this.programs, tokenMint, addresses, poolConfig, tokenConfig)
    await marginPool.refresh()
    return marginPool
  }

  /**
   * Loads all margin pools bases on the config provided to the manager
   * @returns
   */
  async loadAll(): Promise<Record<MarginTokens, Pool>> {
    // FIXME: This could be faster with fewer round trips to rpc
    const pools: Record<string, Pool> = {}
    for (const poolConfig of Object.values(this.programs.config.pools)) {
      const poolTokenMint = translateAddress(poolConfig.tokenMint)
      const tokenConfig = Object.values(this.programs.config.tokens).find(token =>
        translateAddress(token.mint).equals(poolTokenMint)
      )
      if (tokenConfig) {
        const pool = await this.load(poolConfig.tokenMint, poolConfig, tokenConfig)
        pools[poolConfig.symbol] = pool
      }
    }
    return pools
  }

  /**
   * Creates a margin pool
   * @param args  // TODO document interface
   * @returns 
   */
  async create({
    tokenMint,
    collateralWeight,
    collateralMaxStaleness,
    feeDestination,
    pythProduct,
    pythPrice,
    marginPoolConfig
  }: IPoolCreationParams) {
    const addresses = this._derive(this.programs, tokenMint)
    const address = addresses.marginPool
    const ix1: TransactionInstruction[] = []
    if (this.owner) {
      try {
        await this._withRegisterToken(ix1, this.owner, addresses, address)
        await this.provider.sendAndConfirm(new Transaction().add(...ix1))
        const ix2: TransactionInstruction[] = []
        await this._withConfigureToken(
          ix2,
          this.owner,
          collateralWeight,
          collateralMaxStaleness,
          feeDestination,
          pythProduct,
          pythPrice,
          marginPoolConfig,
          addresses,
          addresses.marginPool
        )

        return await this.provider.sendAndConfirm(new Transaction().add(...ix2))
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
  private async _withRegisterToken(
    instructions: TransactionInstruction[],
    requester: Address,
    addresses: MarginPoolAddresses,
    address: PublicKey
  ): Promise<void> {
    const authority = findDerivedAccount(this.programs.config.controlProgramId)

    const ix = await this.programs.control.methods
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
        marginPoolProgram: this.programs.config.marginPoolProgramId,
        metadataProgram: this.programs.config.metadataProgramId,
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
   * @param collateralMaxStaleness 
   * @param feeDestination 
   * @param pythProduct 
   * @param pythPrice 
   * @param marginPoolConfig 
   * @param addresses 
   * @param address 
   */
  private async _withConfigureToken(
    instructions: TransactionInstruction[],
    requester: Address,
    collateralWeight: number,
    collateralMaxStaleness: BN,
    feeDestination: Address,
    pythProduct: Address,
    pythPrice: Address,
    marginPoolConfig: MarginPoolConfigData,
    addresses: MarginPoolAddresses,
    address: PublicKey
  ): Promise<void> {
    // Set the token configuration, e.g. collateral weight
    const metadata: TokenMetadataParams = {
      tokenKind: { collateral: {} },
      collateralWeight: collateralWeight,
      collateralMaxStaleness: collateralMaxStaleness
    }
    const poolParam: MarginPoolParams = {
      feeDestination: translateAddress(feeDestination)
    }

    const ix = await this.programs.control.methods
      .configureToken(
        {
          tokenKind: metadata.tokenKind as never,
          collateralWeight: metadata.collateralWeight,
          collateralMaxStaleness: metadata.collateralMaxStaleness
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
        pythProduct: pythProduct,
        pythPrice: pythPrice,
        marginPoolProgram: this.programs.config.marginPoolProgramId,
        metadataProgram: this.programs.config.metadataProgramId
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
  private _derive(programs: MarginPrograms, tokenMint: Address): MarginPoolAddresses {
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
