import assert from "assert"
import { Address, AnchorProvider, BN, translateAddress } from "@project-serum/anchor"
import { Mint, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js"

import { MarginAccount } from "../marginAccount"
import { MarginPrograms } from "../marginClient"
import { findDerivedAccount } from "../../utils/pda"
import { AssociatedToken } from "../../token"
import { MarginPoolData } from "./state"
import { MarginTokenConfig, MarginTokens } from "../config"
import { PoolAmount } from "./poolAmount"
import { parsePriceData, PriceData } from "@pythnetwork/client"

type TokenKindNonCollateral = { nonCollateral: Record<string, never> }
type TokenKindCollateral = { collateral: Record<string, never> }
type TokenKindClaim = { claim: Record<string, never> }

export type TokenKind = TokenKindNonCollateral | TokenKindCollateral | TokenKindClaim

export interface MarginPoolAddresses {
  /** The pool's token mint i.e. BTC or SOL mint address*/
  tokenMint: PublicKey
  marginPool: PublicKey
  vault: PublicKey
  depositNoteMint: PublicKey
  loanNoteMint: PublicKey
  marginPoolAdapterMetadata: PublicKey
  tokenMetadata: PublicKey
  depositNoteMetadata: PublicKey
  loanNoteMetadata: PublicKey
  controlAuthority: PublicKey
}

export interface TokenMetadataParams {
  tokenKind: TokenKind
  collateralWeight: number
  collateralMaxStaleness: BN
}

export interface MarginPoolParams {
  feeDestination: PublicKey
}

export interface MarginPoolConfig {
  flags: BN
  utilizationRate1: number
  utilizationRate2: number
  borrowRate0: number
  borrowRate1: number
  borrowRate2: number
  borrowRate3: number
  managementFeeRate: number
  managementFeeCollectThreshold: BN
}

export class MarginPool {
  public address: PublicKey
  public tokenConfig: MarginTokenConfig
  public info?: {
    marginPool: MarginPoolData
    tokenMint: Mint
    vault: AssociatedToken
    depositNoteMint: Mint
    loanNoteMint: Mint
    tokenPriceOracle: PriceData
  }

  get tokenPrice(): number | undefined {
    return this.info?.tokenPriceOracle.price
  }

  constructor(public programs: MarginPrograms, public addresses: MarginPoolAddresses) {
    assert(programs)
    assert(addresses)
    this.address = addresses.marginPool
    const mintAddress = addresses.tokenMint.toBase58()
    this.tokenConfig = Object.values(this.programs.config.tokens).find(token => token.mint === mintAddress)!
  }

  /**
   * Derive accounts from tokenMint
   * @param {MarginPrograms} programs
   * @param {Address} tokenMint
   * @returns {PublicKey} Margin Pool Address
   */
  static derive(programs: MarginPrograms, tokenMint: Address): MarginPoolAddresses {
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

  static async load(programs: MarginPrograms, tokenMint: Address): Promise<MarginPool> {
    assert(programs)
    assert(tokenMint)

    const addresses = this.derive(programs, tokenMint)
    const marginPool = new MarginPool(programs, addresses)
    await marginPool.refresh()
    return marginPool
  }

  /**
   * Load every Margin Pool in the config.
   * @param programs
   * @returns
   */
  static async loadAll(programs: MarginPrograms): Promise<Record<MarginTokens, MarginPool>> {
    // FIXME: This could be faster with fewer round trips to rpc
    const pools: Record<string, MarginPool> = {}
    for (const token of Object.values(programs.config.tokens)) {
      const pool = await this.load(programs, token.mint)
      pools[token.symbol.toString()] = pool
    }
    return pools
  }

  async refresh() {
    const [marginPoolInfo, poolTokenMintInfo, vaultMintInfo, depositNoteMintInfo, loanNoteMintInfo] =
      await this.programs.marginPool.provider.connection.getMultipleAccountsInfo([
        this.addresses.marginPool,
        this.addresses.tokenMint,
        this.addresses.vault,
        this.addresses.depositNoteMint,
        this.addresses.loanNoteMint
      ])

    if (!marginPoolInfo || !poolTokenMintInfo || !vaultMintInfo || !depositNoteMintInfo || !loanNoteMintInfo) {
      this.info = undefined
    } else {
      const marginPool = this.programs.marginPool.coder.accounts.decode<MarginPoolData>(
        "marginPool",
        marginPoolInfo.data
      )
      const oracleInfo = await this.programs.marginPool.provider.connection.getAccountInfo(marginPool.tokenPriceOracle)
      assert(
        oracleInfo,
        "Pyth oracle does not exist but a margin pool does. The margin pool is incorrectly configured."
      )
      this.info = {
        marginPool,
        tokenMint: AssociatedToken.decodeMint(poolTokenMintInfo, this.addresses.tokenMint),
        vault: AssociatedToken.decodeAccount(vaultMintInfo, this.addresses.vault, this.tokenConfig.decimals),
        depositNoteMint: AssociatedToken.decodeMint(depositNoteMintInfo, this.addresses.depositNoteMint),
        loanNoteMint: AssociatedToken.decodeMint(loanNoteMintInfo, this.addresses.loanNoteMint),
        tokenPriceOracle: parsePriceData(oracleInfo.data)
      }
    }
  }

  async create(
    provider: AnchorProvider,
    requester: Address,
    collateralWeight: number,
    collateralMaxStaleness: BN,
    feeDestination: Address,
    pythProduct: Address,
    pythPrice: Address,
    marginPoolConfig: MarginPoolConfig
  ) {
    const ix1: TransactionInstruction[] = []
    await this.withRegisterToken(ix1, requester)
    await provider.sendAndConfirm(new Transaction().add(...ix1))

    const ix2: TransactionInstruction[] = []
    await this.withConfigureToken(
      ix2,
      requester,
      collateralWeight,
      collateralMaxStaleness,
      feeDestination,
      pythProduct,
      pythPrice,
      marginPoolConfig
    )
    try {
      return await provider.sendAndConfirm(new Transaction().add(...ix2))
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async withRegisterToken(instructions: TransactionInstruction[], requester: Address): Promise<void> {
    const authority = findDerivedAccount(this.programs.config.controlProgramId)

    const ix = await this.programs.control.methods
      .registerToken()
      .accounts({
        requester,
        authority,
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        loanNoteMint: this.addresses.loanNoteMint,
        tokenMint: this.addresses.tokenMint,
        tokenMetadata: this.addresses.tokenMetadata,
        depositNoteMetadata: this.addresses.depositNoteMetadata,
        loanNoteMetadata: this.addresses.loanNoteMetadata,
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
   */
  async withConfigureToken(
    instructions: TransactionInstruction[],
    requester: Address,
    collateralWeight: number,
    collateralMaxStaleness: BN,
    feeDestination: Address,
    pythProduct: Address,
    pythPrice: Address,
    marginPoolConfig: MarginPoolConfig
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
        authority: this.addresses.controlAuthority,
        tokenMint: this.addresses.tokenMint,
        marginPool: this.address,
        tokenMetadata: this.addresses.tokenMetadata,
        depositMetadata: this.addresses.depositNoteMetadata,
        pythProduct: pythProduct,
        pythPrice: pythPrice,
        marginPoolProgram: this.programs.config.marginPoolProgramId,
        metadataProgram: this.programs.config.metadataProgramId
      })
      .instruction()
    instructions.push(ix)
  }

  /// Instruction to deposit tokens into the pool in exchange for deposit notes
  ///
  /// # Params
  ///
  /// `depositor` - The authority for the source tokens
  /// `source` - The token account that has the tokens to be deposited
  /// `destination` - The token account to send notes representing the deposit
  /// `amount` - The amount of tokens to be deposited
  async deposit(marginAccount: MarginAccount, source: Address, amount: number) {
    await marginAccount.refresh()
    const position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(position)

    const ix: TransactionInstruction[] = []

    await this.withDeposit(ix, marginAccount.address, source, position.address, new BN(amount))
    await marginAccount.withUpdatePositionBalance(ix, position.address)

    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  async withDeposit(
    instructions: TransactionInstruction[],
    depositor: Address,
    source: Address,
    destination: Address,
    amount: BN
  ): Promise<void> {
    const ix = await this.programs.marginPool.methods
      .deposit(amount)
      .accounts({
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        depositor,
        source,
        destination,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
    instructions.push(ix)
  }

  // async refreshAllPoolPositions(
  //   connection: Connection,
  //   marginAccount: MarginAccount,
  // ) {
  //   // we need to get the positions
  //   //
  // }

  async refreshPosition(marginAccount: MarginAccount) {
    const tokenMetadata = await marginAccount.getTokenMetadata(this.addresses.tokenMint)

    const ix: TransactionInstruction[] = []
    await this.withAdapterInvoke(
      ix,
      marginAccount.owner,
      marginAccount.address,
      this.programs.config.marginPoolProgramId,
      this.addresses.marginPoolAdapterMetadata,
      await this.makeMarginRefreshPositionInstruction(marginAccount.address, tokenMetadata.pythPrice)
    )
    try {
      return await marginAccount.provider.sendAndConfirm(new Transaction().add(...ix))
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async marginBorrow(marginAccount: MarginAccount, amount: BN) {
    await marginAccount.refresh()
    const deposit_position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(deposit_position)

    const loan_position = await marginAccount.getOrCreatePosition(this.addresses.loanNoteMint)
    assert(loan_position)

    const tokenMetadata = await marginAccount.getTokenMetadata(this.addresses.tokenMint)

    const data = Buffer.from(Uint8Array.of(0, ...new BN(500000).toArray("le", 4)))
    const additionalComputeBudgetInstruction = new TransactionInstruction({
      keys: [],
      programId: new PublicKey("ComputeBudget111111111111111111111111111111"),
      data
    })
    const ix: TransactionInstruction[] = [additionalComputeBudgetInstruction]
    await this.withAdapterInvoke(
      ix,
      marginAccount.owner,
      marginAccount.address,
      this.programs.config.marginPoolProgramId,
      this.addresses.marginPoolAdapterMetadata,
      await this.makeMarginRefreshPositionInstruction(marginAccount.address, tokenMetadata.pythPrice)
    )
    await this.withAdapterInvoke(
      ix,
      marginAccount.owner,
      marginAccount.address,
      this.programs.config.marginPoolProgramId,
      this.addresses.marginPoolAdapterMetadata,
      await this.makeMarginBorrowInstruction(
        marginAccount.address,
        deposit_position.address,
        loan_position.address,
        amount
      )
    )
    try {
      return await marginAccount.provider.sendAndConfirm(new Transaction().add(...ix))
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async makeMarginRefreshPositionInstruction(
    marginAccount: Address,
    tokenPriceOracle: Address
  ): Promise<TransactionInstruction> {
    assert(marginAccount)
    assert(tokenPriceOracle)
    return await this.programs.marginPool.methods
      .marginRefreshPosition()
      .accounts({
        marginAccount,
        marginPool: this.address,
        tokenPriceOracle
      })
      .instruction()
  }

  /// Instruction to borrow tokens using a margin account
  ///
  /// # Params
  ///
  /// `margin_scratch` - The scratch account for the margin system
  /// `margin_account` - The account being borrowed against
  /// `deposit_account` - The account to receive the notes for the borrowed tokens
  /// `loan_account` - The account to receive the notes representing the debt
  /// `amount` - The amount of tokens to be borrowed
  async makeMarginBorrowInstruction(
    marginAccount: Address,
    deposit_account: Address,
    loan_account: Address,
    amount: BN
  ): Promise<TransactionInstruction> {
    assert(marginAccount)
    assert(deposit_account)
    assert(loan_account)
    return await this.programs.marginPool.methods
      .marginBorrow(amount)
      .accounts({
        marginAccount,
        marginPool: this.address,
        loanNoteMint: this.addresses.loanNoteMint,
        depositNoteMint: this.addresses.depositNoteMint,
        loanAccount: loan_account,
        depositAccount: deposit_account,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  /// Instruction to repay tokens owed by a margin account
  ///
  /// # Params
  ///
  /// `margin_scratch` - The scratch account for the margin system
  /// `margin_account` - The account with the loan to be repaid
  /// `deposit_account` - The account with notes to repay the loan
  /// `loan_account` - The account with the loan debt to be reduced
  /// `amount` - The amount to be repaid
  async marginRepay(marginAccount: MarginAccount, amount: PoolAmount) {
    await marginAccount.refresh()
    const deposit_position = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(deposit_position)

    const loan_position = await marginAccount.getOrCreatePosition(this.addresses.loanNoteMint)
    assert(loan_position)

    const ix: TransactionInstruction[] = []
    await this.withAdapterInvoke(
      ix,
      marginAccount.owner,
      marginAccount.address,
      this.programs.config.marginPoolProgramId,
      this.addresses.marginPoolAdapterMetadata,
      await this.makeMarginRepayInstruction(
        marginAccount.address,
        deposit_position.address,
        loan_position.address,
        amount
      )
    )

    return await marginAccount.provider.sendAndConfirm(new Transaction().add(...ix))
  }

  async makeMarginRepayInstruction(
    marginAccount: Address,
    deposit_account: Address,
    loan_account: Address,
    amount: PoolAmount
  ): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .marginRepay(amount.toRpcArg())
      .accounts({
        marginAccount: marginAccount,
        marginPool: this.address,
        loanNoteMint: this.addresses.loanNoteMint,
        depositNoteMint: this.addresses.depositNoteMint,
        loanAccount: loan_account,
        depositAccount: deposit_account,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  /// Instruction to withdraw tokens from the pool in exchange for deposit notes
  /// (owned by a margin account)
  ///
  /// # Params
  ///
  /// `margin_scratch` - The scratch account for the margin system
  /// `margin_account` - The margin account with the deposit to be withdrawn
  /// `source` - The token account that has the deposit notes to be exchanged
  /// `destination` - The token account to send the withdrawn deposit
  /// `PoolAmount` - The amount of the deposit
  async marginWithdraw(marginAccount: MarginAccount, destination: Address, amount: PoolAmount) {
    const depositPosition = await marginAccount.getOrCreatePosition(this.addresses.depositNoteMint)
    assert(depositPosition)

    const tx = new Transaction()
    const ix: TransactionInstruction[] = []
    await this.withAdapterInvoke(
      ix,
      marginAccount.owner,
      marginAccount.address,
      this.programs.config.marginPoolProgramId,
      this.addresses.marginPoolAdapterMetadata,
      await this.makeMarginWithdrawInstruction(marginAccount.address, depositPosition.address, destination, amount)
    )
    tx.add(...ix)

    try {
      return await marginAccount.provider.sendAndConfirm(tx)
    } catch (err) {
      console.log(err)
      throw err
    }
  }

  async makeMarginWithdrawInstruction(
    marginAccount: Address,
    source: Address,
    destination: Address,
    amount: PoolAmount
  ): Promise<TransactionInstruction> {
    return await this.programs.marginPool.methods
      .marginWithdraw(amount.toRpcArg())
      .accounts({
        marginAccount: marginAccount,
        marginPool: this.address,
        vault: this.addresses.vault,
        depositNoteMint: this.addresses.depositNoteMint,
        source,
        destination,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction()
  }

  async withAdapterInvoke(
    instructions: TransactionInstruction[],
    owner: Address,
    marginAccount: Address,
    adapterProgram: Address,
    adapterMetadata: Address,
    adapterInstruction: TransactionInstruction
  ): Promise<void> {
    const ix = await this.programs.margin.methods
      .adapterInvoke(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return { isSigner: false, isWritable: accountMeta.isWritable }
        }),
        adapterInstruction.data
      )
      .accounts({
        owner,
        marginAccount,
        adapterProgram,
        adapterMetadata
      })
      .remainingAccounts(
        adapterInstruction.keys.slice(1).map(accountMeta => {
          return {
            pubkey: accountMeta.pubkey,
            isSigner: false,
            isWritable: accountMeta.isWritable
          }
        })
      )
      .instruction()
    instructions.push(ix)
  }
}
