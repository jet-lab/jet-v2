import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { AssociatedToken, FixedTermMarketConfig, MarginAccount, Pool, PoolTokenChange, sendAll } from "@jet-lab/margin"
import { FixedTermMarket } from "./fixedTerm"
import { AnchorProvider, BN } from "@project-serum/anchor"

const createRandomSeed = (byteLength: number) => {
  const max = 127
  const min = 0
  return Uint8Array.from(new Array(byteLength).fill(0).map(() => Math.ceil(Math.random() * (max - min) + min)))
}

const refreshAllMarkets = async (
  markets: FixedTermMarket[],
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
            market: market.addresses.market,
            underlyingOracle: market.addresses.underlyingOracle,
            ticketOracle: market.addresses.ticketOracle,
            tokenProgram: TOKEN_PROGRAM_ID
          })
          .instruction()

        await marginAccount.withAccountingInvoke({
          instructions: ixs,
          adapterInstruction: refreshIx
        })
      }
    })
  )
}

// CREATE MARKET ACCOUNT
interface IWithCreateFixedTermMarketAccount {
  market: FixedTermMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
  markets: FixedTermMarket[]
  refreshPools: boolean
  pools: Record<string, Pool>
  pool: Pool
}
export const withCreateFixedTermMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
  markets,
  refreshPools,
  pools,
  pool
}: IWithCreateFixedTermMarketAccount) => {
  const tokenMint = market.addresses.underlyingTokenMint
  const ticketMint = market.addresses.ticketMint
  const marketIXS: TransactionInstruction[] = []
  await AssociatedToken.withCreate(marketIXS, provider, marginAccount.address, tokenMint)
  await AssociatedToken.withCreate(marketIXS, provider, marginAccount.address, ticketMint)
  const marginUserInfo = await market.fetchMarginUser(marginAccount)
  if (!marginUserInfo) {
    const createAccountIx = await market.registerAccountWithMarket(marginAccount, walletAddress)
    await marginAccount.withAdapterInvoke({
      instructions: marketIXS,
      adapterInstruction: createAccountIx
    })
  }
  await refreshAllMarkets(markets, marketIXS, marginAccount, market.address)

  if (refreshPools) {
    await pool.withPrioritisedPositionRefresh({
      instructions: marketIXS,
      pools,
      marginAccount
    })
  }
  return { tokenMint, ticketMint, marketIXS }
}

// MARKET MAKER ORDERS
interface ICreateLendOrder {
  market: {
    market: FixedTermMarket
    config: FixedTermMarketConfig
  }
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
  amount: BN
  basisPoints: BN
  pools: Record<string, Pool>
  marketAccount?: string
  marketConfig: FixedTermMarketConfig
  markets: FixedTermMarket[]
}
export const offerLoan = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
  amount,
  basisPoints,
  pools,
  marketConfig,
  markets
}: ICreateLendOrder) => {
  const pool = pools[market.config.symbol]
  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    pool,
    refreshPools: true
  })
  instructions.push(marketIXS)
  const orderIXS: TransactionInstruction[] = []

  // refresh pool positions
  await pool.withPrioritisedPositionRefresh({
    instructions: orderIXS,
    pools,
    marginAccount
  })

  // create lend instruction
  await pool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  const loanOffer = await market.market.offerLoanIx(
    marginAccount,
    amount,
    basisPoints,
    walletAddress,
    createRandomSeed(8),
    marketConfig.borrowTenor
  )
  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: loanOffer
  })
  instructions.push(orderIXS)
  return sendAll(provider, [instructions])
}

interface ICreateBorrowOrder {
  market: {
    market: FixedTermMarket
    config: FixedTermMarketConfig
  }
  marginAccount: MarginAccount
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  amount: BN
  basisPoints: BN
  marketConfig: FixedTermMarketConfig
  markets: FixedTermMarket[]
}

export const requestLoan = async ({
  market,
  marginAccount,
  provider,
  walletAddress,
  pools,
  amount,
  basisPoints,
  marketConfig,
  markets
}: ICreateBorrowOrder): Promise<string> => {
  const pool = pools[market.config.symbol]

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    pool,
    refreshPools: true
  })
  instructions.push(marketIXS)

  const orderIXS: TransactionInstruction[] = []
  // refresh pools positions
  await pool.withPrioritisedPositionRefresh({
    instructions: orderIXS,
    pools,
    marginAccount
  })

  // Create borrow instruction
  const borrowOffer = await market.market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
    createRandomSeed(8),
    marketConfig.borrowTenor
  )

  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: borrowOffer
  })
  instructions.push(orderIXS)
  return sendAll(provider, [instructions])
}

interface ICancelOrder {
  market: {
    market: FixedTermMarket
    config: FixedTermMarketConfig
  }
  marginAccount: MarginAccount
  provider: AnchorProvider
  orderId: Uint8Array
  pools: Record<string, Pool>
}
export const cancelOrder = async ({
  market,
  marginAccount,
  provider,
  orderId,
  pools
}: ICancelOrder): Promise<string> => {
  let instructions: TransactionInstruction[] = []
  const borrowerAccount = await market.market.deriveMarginUserAddress(marginAccount)
  const pool = pools[market.config.symbol]
  // refresh pools positions
  await pool.withPrioritisedPositionRefresh({
    instructions,
    pools,
    marginAccount
  })

  // refresh market instruction
  const refreshIx = await market.market.program.methods
    .refreshPosition(true)
    .accounts({
      marginUser: borrowerAccount,
      marginAccount: marginAccount.address,
      market: market.market.addresses.market,
      underlyingOracle: market.market.addresses.underlyingOracle,
      ticketOracle: market.market.addresses.ticketOracle,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .instruction()

  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: refreshIx
  })
  const cancelLoan = await market.market.cancelOrderIx(marginAccount, orderId)
  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: cancelLoan
  })
  return sendAll(provider, [instructions])
}

// MARKET TAKER ORDERS

interface IBorrowNow {
  market: {
    market: FixedTermMarket
    config: FixedTermMarketConfig
  }
  marginAccount: MarginAccount
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  amount: BN
  markets: FixedTermMarket[]
}

export const borrowNow = async ({
  market,
  marginAccount,
  provider,
  walletAddress,
  pools,
  amount,
  markets
}: IBorrowNow): Promise<string> => {
  const pool = pools[market.config.symbol]

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS, tokenMint } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    pool,
    refreshPools: true
  })

  instructions.push(marketIXS)
  // refresh pools positions
  const orderIXS: TransactionInstruction[] = []

  // Create borrow instruction
  const seed = createRandomSeed(8)
  const borrowNow = await market.market.borrowNowIx(marginAccount, walletAddress, amount, seed)

  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: borrowNow
  })

  const change = PoolTokenChange.shiftBy(amount.sub(new BN(1)))
  const source = AssociatedToken.derive(tokenMint, marginAccount.address)
  const position = await pool.withGetOrRegisterDepositPosition({ instructions: orderIXS, marginAccount })

  const depositIx = await pool.programs.marginPool.methods
    .deposit(change.changeKind.asParam(), change.value)
    .accounts({
      marginPool: pool.address,
      vault: pool.addresses.vault,
      depositNoteMint: pool.addresses.depositNoteMint,
      depositor: marginAccount.address,
      source,
      destination: position,
      tokenProgram: TOKEN_PROGRAM_ID
    })
    .instruction()
  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: depositIx
  })
  instructions.push(orderIXS)
  return sendAll(provider, [instructions])
}

interface ILendNow {
  market: {
    market: FixedTermMarket
    config: FixedTermMarketConfig
  }
  marginAccount: MarginAccount
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  amount: BN
  markets: FixedTermMarket[]
}

export const lendNow = async ({
  market,
  marginAccount,
  provider,
  walletAddress,
  pools,
  amount,
  markets
}: ILendNow): Promise<string> => {
  const pool = pools[market.config.symbol]
  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    pool,
    refreshPools: true
  })
  instructions.push(marketIXS)
  const orderIXS: TransactionInstruction[] = []

  // create lend instruction
  await pool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  // Create borrow instruction
  const lendNow = await market.market.lendNowIx(marginAccount, amount, walletAddress, createRandomSeed(8))

  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: lendNow
  })

  instructions.push(orderIXS)
  const updateIXS: TransactionInstruction[] = []
  await marginAccount.withUpdateAllPositionBalances({ instructions: updateIXS })
  instructions.push(updateIXS)

  return sendAll(provider, [instructions])
}

interface ISettle {
  markets: FixedTermMarket[]
  marginAccount: MarginAccount
  provider: AnchorProvider
}

export const settle = async ({ markets, marginAccount, provider }: ISettle) => {
  const instructions: TransactionInstruction[] = []
  await Promise.all(
    markets.map(async market => {
      const user = await market.fetchMarginUser(marginAccount)
      if (user && (user.assets.entitledTickets.gtn(0) || user.assets.entitledTokens.gtn(0))) {
        const settleIX = await market.settle(marginAccount)
        marginAccount.withAdapterInvoke({
          instructions,
          adapterInstruction: settleIX
        })
      }
    })
  )
  return sendAll(provider, [instructions])
}
