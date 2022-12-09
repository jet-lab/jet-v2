import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import {
  AssociatedToken,
  FixedTermMarketConfig,
  MarginAccount,
  MarginConfig,
  Pool,
  PoolTokenChange,
  sendAll
} from "@jet-lab/margin"
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
  currentPool: Pool
}
export const withCreateFixedTermMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress,
  markets,
  refreshPools,
  pools,
  currentPool
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
    await currentPool.withPrioritisedPositionRefresh({
      instructions: marketIXS,
      pools,
      marginAccount
    })
  }
  return { tokenMint, ticketMint, marketIXS }
}

// MARKET MAKER ORDERS
interface ICreateLendOrder {
  market: FixedTermMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  walletAddress: PublicKey
  amount: BN
  basisPoints: BN
  pools: Record<string, Pool>
  currentPool: Pool
  marketAccount?: string
  marketConfig: FixedTermMarketConfig
  markets: FixedTermMarket[]
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
  // Fail if there is no active fixed term market program id in the config
  if (!marginConfig.fixedTermMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    currentPool,
    refreshPools: true
  })
  instructions.push(marketIXS)
  const orderIXS: TransactionInstruction[] = []

  // refresh pool positions
  await currentPool.withPrioritisedPositionRefresh({
    instructions: orderIXS,
    pools,
    marginAccount
  })

  // create lend instruction
  await currentPool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  const loanOffer = await market.offerLoanIx(
    marginAccount,
    amount,
    basisPoints,
    walletAddress,
    createRandomSeed(4),
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
  market: FixedTermMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  basisPoints: BN
  marketConfig: FixedTermMarketConfig
  markets: FixedTermMarket[]
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
  // Fail if there is no active fixed term market program id in the config
  if (!marginConfig.fixedTermMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    currentPool,
    refreshPools: true
  })
  instructions.push(marketIXS)

  const orderIXS: TransactionInstruction[] = []
  // refresh pools positions
  await currentPool.withPrioritisedPositionRefresh({
    instructions: orderIXS,
    pools,
    marginAccount
  })

  // Create borrow instruction
  const borrowOffer = await market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
    createRandomSeed(4),
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
  market: FixedTermMarket
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
  await currentPool.withPrioritisedPositionRefresh({
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
      market: market.addresses.market,
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
  market: FixedTermMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  markets: FixedTermMarket[]
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
  // Fail if there is no active fixed term market program id in the config
  if (!marginConfig.fixedTermMarketProgramId) {
    throw new Error("There is no fixed term market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS, tokenMint } = await withCreateFixedTermMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    currentPool,
    refreshPools: true
  })
  instructions.push(marketIXS)
  // refresh pools positions
  const orderIXS: TransactionInstruction[] = []
  await currentPool.withPrioritisedPositionRefresh({
    instructions: orderIXS,
    pools,
    marginAccount
  })

  // Create borrow instruction
  const seed = createRandomSeed(4)
  const borrowNow = await market.borrowNowIx(marginAccount, walletAddress, amount, seed)

  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: borrowNow
  })

  const change = PoolTokenChange.shiftBy(amount.sub(new BN(1)))
  const source = AssociatedToken.derive(tokenMint, marginAccount.address)
  const position = currentPool.findDepositPositionAddress(marginAccount)
  const depositIx = await currentPool.programs.marginPool.methods
    .deposit(change.changeKind.asParam(), change.value)
    .accounts({
      marginPool: currentPool.address,
      vault: currentPool.addresses.vault,
      depositNoteMint: currentPool.addresses.depositNoteMint,
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
  market: FixedTermMarket
  marginAccount: MarginAccount
  marginConfig: MarginConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  currentPool: Pool
  amount: BN
  markets: FixedTermMarket[]
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
  // Fail if there is no active fixed term market program id in the config
  if (!marginConfig.fixedTermMarketProgramId) {
    throw new Error("There is no market configured on this network")
  }

  const instructions: TransactionInstruction[][] = []
  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market,
    provider,
    marginAccount,
    walletAddress,
    markets,
    pools,
    currentPool,
    refreshPools: true
  })
  instructions.push(marketIXS)

  const orderIXS: TransactionInstruction[] = []

  // create lend instruction
  await currentPool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  // Create borrow instruction
  const lendNow = await market.lendNowIx(marginAccount, amount, walletAddress, createRandomSeed(4))

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
