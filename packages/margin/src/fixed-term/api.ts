import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { AssociatedToken, FixedTermMarketConfig, MarginAccount, Pool, PoolTokenChange, sendAll } from "../index"
import { FixedTermMarket, MarketAndconfig } from "./fixedTerm"
import { AnchorProvider, BN } from "@project-serum/anchor"

const createRandomSeed = (byteLength: number) => {
  const max = 127
  const min = 0
  return Uint8Array.from(new Array(byteLength).fill(0).map(() => Math.ceil(Math.random() * (max - min) + min)))
}

// CREATE MARKET ACCOUNT
interface IWithCreateFixedTermMarketAccount {
  market: FixedTermMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
  markets: FixedTermMarket[]
}
export const withCreateFixedTermMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
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
  return { tokenMint, ticketMint, marketIXS }
}

// MARKET MAKER ORDERS
interface ICreateLendOrder {
  market: MarketAndconfig
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
    markets
  })
  instructions.push(marketIXS)

  const poolIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: poolIXS,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(poolIXS)

  const orderIXS: TransactionInstruction[] = []

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
  market: MarketAndconfig
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
  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets
  })
  instructions.push(marketIXS)

  const poolIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: poolIXS,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(poolIXS)

  const orderIXS: TransactionInstruction[] = []
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
  market: MarketAndconfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  orderId: BN
  amount: BN
}
export const cancelOrder = async ({ market, marginAccount, provider, orderId }: ICancelOrder): Promise<string> => {
  let instructions: TransactionInstruction[] = []
  const cancelLoan = await market.market.cancelOrderIx(marginAccount, orderId)
  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: cancelLoan
  })
  return sendAll(provider, [instructions])
}

// MARKET TAKER ORDERS

interface IBorrowNow {
  market: MarketAndconfig
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
    markets
  })
  instructions.push(marketIXS)
  // refresh pools positions

  const poolIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: poolIXS,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(poolIXS)

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
  market: MarketAndconfig
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
    markets
  })
  instructions.push(marketIXS)

  const poolIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: poolIXS,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(poolIXS)

  const orderIXS: TransactionInstruction[] = []
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
  markets: MarketAndconfig[]
  selectedMarket: number
  marginAccount: MarginAccount
  provider: AnchorProvider
  pools: Record<string, Pool>
  amount: BN
}

export const settle = async ({ markets, selectedMarket, marginAccount, provider, pools, amount }: ISettle) => {
  const { market, token } = markets[selectedMarket]
  const instructions: TransactionInstruction[][] = []
  const pool = pools[token.symbol]
  const refreshIXS: TransactionInstruction[] = []

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshIXS,
    pools,
    markets: markets.map(m => m.market),
    marketAddress: market.address
  })

  instructions.push(refreshIXS)
  const settleIXS: TransactionInstruction[] = []
  const settleIX = await market.settle(marginAccount)
  marginAccount.withAccountingInvoke({
    instructions: settleIXS,
    adapterInstruction: settleIX
  })
  const change = PoolTokenChange.shiftBy(amount)
  const source = AssociatedToken.derive(market.addresses.underlyingTokenMint, marginAccount.address)
  const position = await pool.withGetOrRegisterDepositPosition({ instructions: settleIXS, marginAccount })

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
    instructions: settleIXS,
    adapterInstruction: depositIx
  })
  await marginAccount.withUpdatePositionBalance({ instructions: settleIXS, position })
  instructions.push(settleIXS)
  return sendAll(provider, [instructions])
}
