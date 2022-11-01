import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { AssociatedToken, MarginAccount, MarginConfig, Pool, sendAll } from "@jet-lab/margin"
import { BondMarket } from "./bondMarket"
import { AnchorProvider, BN } from "@project-serum/anchor"

const createRandomSeed = (byteLength: number) => {
  const max = 127
  const min = 0
  return Uint8Array.from(new Array(byteLength).fill(0).map(() => Math.ceil(Math.random() * (max - min) + min)))
}

interface IWithCreateFixedMarketAccount {
  market: BondMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
  instructions: TransactionInstruction[]
  marketAccount: PublicKey
}
export const withCreateFixedMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
  instructions,
  marketAccount
}: IWithCreateFixedMarketAccount) => {
  const tokenMint = market.addresses.underlyingTokenMint
  const ticketMint = market.addresses.bondTicketMint
  await AssociatedToken.withCreate(instructions, provider, marginAccount.address, tokenMint)
  await AssociatedToken.withCreate(instructions, provider, marginAccount.address, ticketMint)
  const info = await provider.connection.getAccountInfo(marketAccount)
  if (!info) {
    const createAccountIx = await market.registerAccountWithMarket(marginAccount, walletAddress)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: createAccountIx
    })
  }
  return { tokenMint, ticketMint }
}

interface ICreateLendOrder {
  market: BondMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  walletAddress: PublicKey
  amount: BN
  basisPoints: BN
  pools: Record<string, Pool>
  currentPool: Pool
  marketAccount?: string
}
export const createFixedLendOrder = async ({
  market,
  provider,
  marginAccount,
  marginConfig,
  walletAddress,
  amount,
  basisPoints,
  pools,
  currentPool
}: ICreateLendOrder) => {
  // Fail if there is no active bonds program id in the config
  if (!marginConfig.bondsProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const lenderAccount = await market.deriveMarginUserAddress(marginAccount)
  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  const { tokenMint } = await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions,
    marketAccount: lenderAccount
  })
  if (accountInstructions.length > 0) {
    instructions.push(accountInstructions)
  }

  const lendInstructions: TransactionInstruction[] = []

  AssociatedToken.withTransfer(lendInstructions, tokenMint, walletAddress, marginAccount.address, amount)

  // refresh pool positions
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions: lendInstructions,
    pools,
    marginAccount
  })

  // refresh market instruction
  const refreshIx = await market.program.methods
    .refreshPosition(true)
    .accounts({
      marginUser: lenderAccount,
      marginAccount: marginAccount.address,
      bondManager: market.addresses.bondManager,
      underlyingOracle: market.addresses.underlyingOracle,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .instruction()

  await marginAccount.withAdapterInvoke({
    instructions: lendInstructions,
    adapterInstruction: refreshIx
  })

  // create lend instruction
  const loanOffer = await market.offerLoanIx(marginAccount, amount, basisPoints, walletAddress, createRandomSeed(4))
  await marginAccount.withAdapterInvoke({
    instructions: lendInstructions,
    adapterInstruction: loanOffer
  })

  instructions.push(lendInstructions)
  return sendAll(provider, [instructions])
}

interface ICreateBorrowOrder {
  market: BondMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  basisPoints: BN
}

export const createFixedBorrowOrder = async ({
  market,
  marginAccount,
  marginConfig,
  provider,
  walletAddress,
  pools,
  currentPool,
  amount,
  basisPoints
}: ICreateBorrowOrder): Promise<string> => {
  // Fail if there is no active bonds program id in the config
  if (!marginConfig.bondsProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const borrowerAccount = await market.deriveMarginUserAddress(marginAccount)

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions,
    marketAccount: borrowerAccount
  })
  if (accountInstructions.length > 0) {
    instructions.push(accountInstructions)
  }

  // refresh pools positions
  const borrowInstructions: TransactionInstruction[] = []
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions: borrowInstructions,
    pools,
    marginAccount
  })

  // refresh market instruction
  const refreshIx = await market.program.methods
    .refreshPosition(true)
    .accounts({
      marginUser: borrowerAccount,
      marginAccount: marginAccount.address,
      bondManager: market.addresses.bondManager,
      underlyingOracle: market.addresses.underlyingOracle,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .instruction()

  await marginAccount.withAdapterInvoke({
    instructions: borrowInstructions,
    adapterInstruction: refreshIx
  })

  // Create borrow instruction
  const borrowOffer = await market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
    createRandomSeed(4)
  )

  await marginAccount.withAdapterInvoke({
    instructions: borrowInstructions,
    adapterInstruction: borrowOffer
  })

  instructions.push(borrowInstructions)
  return sendAll(provider, [instructions])
}

interface ICancelOrder {
  market: BondMarket
  marginAccount: MarginAccount
  provider: AnchorProvider
  orderId: Uint8Array
  pools: Record<string, Pool>
  currentPool: Pool
}
export const cancelOrder = async ({
  market,
  marginAccount,
  provider,
  orderId,
  pools,
  currentPool
}: ICancelOrder): Promise<string> => {
  let instructions: TransactionInstruction[] = []
  const borrowerAccount = await market.deriveMarginUserAddress(marginAccount)

  // refresh pools positions
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions,
    pools,
    marginAccount
  })

  // refresh market instruction
  const refreshIx = await market.program.methods
    .refreshPosition(true)
    .accounts({
      marginUser: borrowerAccount,
      marginAccount: marginAccount.address,
      bondManager: market.addresses.bondManager,
      underlyingOracle: market.addresses.underlyingOracle,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .instruction()

  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: refreshIx
  })
  const cancelLoan = await market.cancelOrderIx(marginAccount, orderId)
  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: cancelLoan
  })
  return sendAll(provider, [instructions])
}
