import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  AssociatedToken,
  FixedMarketConfig,
  MarginAccount,
  MarginConfig,
  Pool,
  PoolTokenChange,
  sendAll
} from "@jet-lab/margin"
import { FixedMarket } from "./fixedMarket"
import { Address, AnchorProvider, BN } from "@project-serum/anchor"

const createRandomSeed = (byteLength: number) => {
  const max = 127
  const min = 0
  return Uint8Array.from(new Array(byteLength).fill(0).map(() => Math.ceil(Math.random() * (max - min) + min)))
}

const refreshAllMarkets = async (
  markets: FixedMarket[],
  ixs: TransactionInstruction[],
  marginAccount: MarginAccount,
  marketAddress: PublicKey
) => {
  await Promise.all(
    markets.map(async market => {
      const marketUserInfo = await market.fetchMarginUser(marginAccount)
      const marketUser = await market.deriveMarginUserAddress(marginAccount)
      if (marketUserInfo || market.address.equals(marketAddress)) {
        const refreshIx = await market.program.methods
          .refreshPosition(true)
          .accounts({
            marginUser: marketUser,
            marginAccount: marginAccount.address,
            marketManager: market.addresses.marketManager,
            underlyingOracle: market.addresses.underlyingOracle,
            ticketOracle: market.addresses.ticketOracle,
            tokenProgram: TOKEN_PROGRAM_ID
          })
          .instruction()

        await marginAccount.withAdapterInvoke({
          instructions: ixs,
          adapterInstruction: refreshIx
        })
      }
    })
  )
}

// CREATE MARKET ACCOUNT
interface IWithCreateFixedMarketAccount {
  market: FixedMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
  instructions: TransactionInstruction[]
}
export const withCreateFixedMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
  instructions
}: IWithCreateFixedMarketAccount) => {
  const tokenMint = market.addresses.underlyingTokenMint
  const ticketMint = market.addresses.marketTicketMint
  await AssociatedToken.withCreate(instructions, provider, marginAccount.address, tokenMint)
  await AssociatedToken.withCreate(instructions, provider, marginAccount.address, ticketMint)
  const marginUserInfo = await market.fetchMarginUser(marginAccount)
  if (!marginUserInfo) {
    const createAccountIx = await market.registerAccountWithMarket(marginAccount, walletAddress)
    await marginAccount.withAdapterInvoke({
      instructions,
      adapterInstruction: createAccountIx
    })
  }
  return { tokenMint, ticketMint }
}

// MARKET MAKER ORDERS
interface ICreateLendOrder {
  market: FixedMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  walletAddress: PublicKey
  amount: BN
  basisPoints: BN
  pools: Record<string, Pool>
  currentPool: Pool
  marketAccount?: string
  marketConfig: FixedMarketConfig
  markets: FixedMarket[]
}
export const offerLoan = async ({
  market,
  provider,
  marginAccount,
  marginConfig,
  walletAddress,
  amount,
  basisPoints,
  pools,
  currentPool,
  marketConfig,
  markets
}: ICreateLendOrder) => {
  // Fail if there is no active fixed market program id in the config
  if (!marginConfig.fixedMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  const { tokenMint } = await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions
  })
  if (accountInstructions.length > 0) {
    instructions.push(accountInstructions)
  }
  const refreshInstructions: TransactionInstruction[] = []

  // refresh pool positions
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions: refreshInstructions,
    pools,
    marginAccount
  })

  await refreshAllMarkets(markets, refreshInstructions, marginAccount, market.address)
  instructions.push(refreshInstructions)

  const withdrawInstructions: TransactionInstruction[] = []
  // create lend instruction
  await currentPool.withWithdrawToMargin({
    instructions: withdrawInstructions,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })
  instructions.push(withdrawInstructions)

  const lendInstructions: TransactionInstruction[] = []

  const loanOffer = await market.offerLoanIx(
    marginAccount,
    amount,
    basisPoints,
    walletAddress,
    createRandomSeed(8),
    marketConfig.borrowDuration
  )
  await marginAccount.withAdapterInvoke({
    instructions: lendInstructions,
    adapterInstruction: loanOffer
  })

  instructions.push(lendInstructions)
  return sendAll(provider, [instructions])
}

interface ICreateBorrowOrder {
  market: FixedMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  basisPoints: BN
  marketConfig: FixedMarketConfig
  markets: FixedMarket[]
}

export const requestLoan = async ({
  market,
  marginAccount,
  marginConfig,
  provider,
  walletAddress,
  pools,
  currentPool,
  amount,
  basisPoints,
  marketConfig,
  markets
}: ICreateBorrowOrder): Promise<string> => {
  // Fail if there is no active fixed market program id in the config
  if (!marginConfig.fixedMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions
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

  await refreshAllMarkets(markets, borrowInstructions, marginAccount, market.address)

  // Create borrow instruction
  const borrowOffer = await market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
    createRandomSeed(8),
    marketConfig.borrowDuration
  )

  await marginAccount.withAdapterInvoke({
    instructions: borrowInstructions,
    adapterInstruction: borrowOffer
  })

  instructions.push(borrowInstructions)
  return sendAll(provider, [instructions])
}

interface ICancelOrder {
  market: FixedMarket
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
      marketManager: market.addresses.marketManager,
      underlyingOracle: market.addresses.underlyingOracle,
      ticketOracle: market.addresses.ticketOracle,
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

// MARKET TAKER ORDERS

interface IBorrowNow {
  market: FixedMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  markets: FixedMarket[]
}

export const borrowNow = async ({
  marginConfig,
  market,
  marginAccount,
  provider,
  walletAddress,
  currentPool,
  pools,
  amount,
  markets
}: IBorrowNow): Promise<string> => {
  // Fail if there is no active fixed market program id in the config
  if (!marginConfig.fixedMarketProgramId) {
    throw new Error("There is no fixed market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions
  })
  if (accountInstructions.length > 0) {
    instructions.push(accountInstructions)
  }

  // refresh pools positions
  const refreshInstructions: TransactionInstruction[] = []
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions: refreshInstructions,
    pools,
    marginAccount
  })

  await refreshAllMarkets(markets, refreshInstructions, marginAccount, market.address)
  instructions.push(refreshInstructions)

  // Create borrow instruction
  const borrowInstructions: TransactionInstruction[] = []
  const borrowNow = await market.borrowNowIx(marginAccount, walletAddress, amount, createRandomSeed(8))

  await marginAccount.withAdapterInvoke({
    instructions: borrowInstructions,
    adapterInstruction: borrowNow
  })

  instructions.push(borrowInstructions)
  return sendAll(provider, [instructions])
}

interface ILendNow {
  market: FixedMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  markets: FixedMarket[]
}

export const lendNow = async ({
  marginConfig,
  market,
  marginAccount,
  provider,
  walletAddress,
  currentPool,
  pools,
  amount,
  markets
}: ILendNow): Promise<string> => {
  // Fail if there is no active fixed market program id in the config
  if (!marginConfig.fixedMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const accountInstructions: TransactionInstruction[] = []
  const { tokenMint } = await withCreateFixedMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions: accountInstructions
  })
  if (accountInstructions.length > 0) {
    instructions.push(accountInstructions)
  }

  const refreshInstructions: TransactionInstruction[] = []
  await currentPool.withMarginRefreshAllPositionPrices({
    instructions: refreshInstructions,
    pools,
    marginAccount
  })

  await refreshAllMarkets(markets, refreshInstructions, marginAccount, market.address)
  instructions.push(refreshInstructions)

  const withdrawInstructions: TransactionInstruction[] = []
  // create lend instruction
  await currentPool.withWithdrawToMargin({
    instructions: withdrawInstructions,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })
  instructions.push(withdrawInstructions)

  // Create borrow instruction
  const lendInstructions: TransactionInstruction[] = []
  const lendNow = await market.lendNowIx(marginAccount, amount, walletAddress, createRandomSeed(8))

  await marginAccount.withAdapterInvoke({
    instructions: lendInstructions,
    adapterInstruction: lendNow
  })

  instructions.push(lendInstructions)
  return sendAll(provider, [instructions])
}
