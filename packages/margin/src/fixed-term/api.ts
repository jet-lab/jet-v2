import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { FixedTermMarket, MarketAndConfig } from "./fixedTerm"
import { Address, AnchorProvider, BN } from "@project-serum/anchor"
import { FixedTermMarketConfig, MarginAccount, Pool, PoolTokenChange } from "../margin"
import { AssociatedToken } from "../token"
import { sendAll } from "../utils"

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
  await marginAccount.withCreateDepositPosition({ instructions: marketIXS, tokenMint })
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
  market: MarketAndConfig
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

  const prefreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: prefreshIXS,
    pools,
    markets: markets.filter(m => m.address != market.market.address),
  })
  instructions.push(prefreshIXS)

  await marginAccount.withRefreshDepositPosition({
    instructions: prefreshIXS,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets
  })
  instructions.push(marketIXS)

  const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: postfreshIXS,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address,  // TODO Why this in addition to `markets`?
  })
  instructions.push(postfreshIXS)

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
  market: MarketAndConfig
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

  const prefreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: prefreshIXS,
    pools,
    markets: markets.filter(m => m.address != market.market.address),
  })
  instructions.push(prefreshIXS)

  await marginAccount.withRefreshDepositPosition({
    instructions: prefreshIXS,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets
  })
  instructions.push(marketIXS)

  const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: postfreshIXS,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address,  // TODO Why this in addition to `markets`?
  })
  instructions.push(postfreshIXS)

  const orderIXS: TransactionInstruction[] = []
  // Create borrow instruction
  const borrowOffer = await market.market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
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
  market: MarketAndConfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  orderId: BN,
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
}
export const cancelOrder = async ({ market, marginAccount, provider, orderId, pools, markets }: ICancelOrder): Promise<string> => {
  let instructions: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions,
    pools,
    markets,
    marketAddress: market.market.address
  })
  const cancelLoan = await market.market.cancelOrderIx(marginAccount, orderId)
  await marginAccount.withAdapterInvoke({
    instructions,
    adapterInstruction: cancelLoan
  })
  await marginAccount.withPrioritisedPositionRefresh({
    instructions,
    pools,
    markets,
    marketAddress: market.market.address
  })

  await marginAccount.withRefreshDepositPosition({
    instructions,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  return sendAll(provider, [instructions])
}

// MARKET TAKER ORDERS

interface IBorrowNow {
  market: MarketAndConfig
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

  const prefreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: prefreshIXS,
    pools,
    markets: markets.filter(m => m.address != market.market.address),
  })
  instructions.push(prefreshIXS)

  await marginAccount.withRefreshDepositPosition({
    instructions: prefreshIXS,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  // Create relevant accounts if they do not exist
  const { marketIXS, tokenMint } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets
  })
  instructions.push(marketIXS)

  const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: postfreshIXS,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address,  // TODO Why this in addition to `markets`?
  })
  instructions.push(postfreshIXS)

  // Create borrow instruction
  const orderIXS: TransactionInstruction[] = []
  const borrowNow = await market.market.borrowNowIx(marginAccount, walletAddress, amount)

  await marginAccount.withAdapterInvoke({
    instructions: orderIXS,
    adapterInstruction: borrowNow
  })

  const change = PoolTokenChange.shiftBy(amount)
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
  market: MarketAndConfig
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

  const prefreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: prefreshIXS,
    pools,
    markets: markets.filter(m => m.address != market.market.address),
  })
  instructions.push(prefreshIXS)

  await marginAccount.withRefreshDepositPosition({
    instructions: prefreshIXS,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress,
    markets
  })
  instructions.push(marketIXS)

  const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: postfreshIXS,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address,  // TODO Why this in addition to `markets`?
  })
  instructions.push(postfreshIXS)

  const orderIXS: TransactionInstruction[] = []
  await pool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  // Create borrow instruction
  const lendNow = await market.market.lendNowIx(marginAccount, amount, walletAddress)

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
  markets: MarketAndConfig[]
  selectedMarket: MarketAndConfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  pools: Record<string, Pool>
  amount: BN
}

export const settle = async ({ markets, selectedMarket, marginAccount, provider, pools, amount }: ISettle) => {
  const { market, token } = selectedMarket
  const instructions: TransactionInstruction[][] = []
  const pool = pools[token.symbol]
  const refreshIXS: TransactionInstruction[] = []

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshIXS,
    pools,
    markets: markets.map(m => m.market),
  })

  instructions.push(refreshIXS)
  const settleIXS: TransactionInstruction[] = []
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

interface IRepay {
  amount: BN,
  marginAccount: MarginAccount,
  market: MarketAndConfig,
  provider: AnchorProvider,
  termLoans: Array<{
    address: Address,
    balance: number,
    maturation_timestamp: number
    sequence_number: number,
    payer: string
  }>
  pools: Record<string, Pool>,
  markets: FixedTermMarket[],
}

export const repay = async ({
  marginAccount,
  market,
  amount,
  provider,
  termLoans,
  pools,
  markets,
}: IRepay) => {
  const instructions: TransactionInstruction[][] = []

  const poolIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: poolIXS,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(poolIXS)

  await marginAccount.withRefreshDepositPosition({
    instructions: poolIXS,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  const orderIXS: TransactionInstruction[] = []
  const pool = pools[market.token.symbol]
  await pool.withWithdrawToMargin({
    instructions: orderIXS,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })
  const source = AssociatedToken.derive(market.market.addresses.underlyingTokenMint, marginAccount.address)

  let amountLeft = new BN(amount)

  let sortedTermLoans = termLoans.sort((a, b) => a.maturation_timestamp - b.maturation_timestamp || a.sequence_number - b.sequence_number)
  while (amountLeft.gt(new BN(0))) {
    const currentLoan = sortedTermLoans[0]
    const nextLoan = sortedTermLoans[1]
    const balance = new BN(currentLoan.balance)
    if (balance.gte(amountLeft)) {
      const ix = await market.market.repay({
        user: marginAccount,
        termLoan: currentLoan.address,
        nextTermLoan: nextLoan ? nextLoan.address : new PublicKey('11111111111111111111111111111111').toBase58(),
        payer: currentLoan.payer,
        amount: amountLeft,
        source,
      })
      await marginAccount.withAdapterInvoke({
        instructions: orderIXS,
        adapterInstruction: ix
      })
      amountLeft = amountLeft.sub(amountLeft)
    } else {
      const ix = await market.market.repay({
        user: marginAccount,
        termLoan: currentLoan.address,
        nextTermLoan: nextLoan ? nextLoan.address : new PublicKey('11111111111111111111111111111111').toBase58(),
        payer: currentLoan.payer,
        amount: balance,
        source,
      })
      await marginAccount.withAdapterInvoke({
        instructions: orderIXS,
        adapterInstruction: ix
      })
      amountLeft = amountLeft.sub(balance)
      sortedTermLoans.shift()
    }
  }
  instructions.push(orderIXS)
  const refreshIxs: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshIxs,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(refreshIxs)
  return sendAll(provider, [instructions])
}


interface IRedeem {
  marginAccount: MarginAccount
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
  market: MarketAndConfig
  provider: AnchorProvider
  deposits: Array<{
    id: number
    address: string,
    sequence_number: number,
    maturation_timestamp: number,
    balance: number,
    rate: number,
    payer: string,
    created_timestamp: number
  }>
}
export const redeem = async ({
  marginAccount,
  pools,
  markets,
  market,
  provider,
  deposits
}: IRedeem) => {
  const instructions: TransactionInstruction[][] = []
  const refreshIxs: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshIxs,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions.push(refreshIxs)

  await marginAccount.withRefreshDepositPosition({
    instructions: refreshIxs,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  const redeemIxs: TransactionInstruction[] = []
  const sortedDeposits = deposits.sort((a, b) => a.sequence_number - b.sequence_number)

  for (let i = 0; i < sortedDeposits.length; i++) {
    const deposit = sortedDeposits[i]
    const redeem = await market.market.redeemDeposit(
      marginAccount,
      deposit,
      market.market
    )
    await marginAccount.withAdapterInvoke({
      instructions: redeemIxs,
      adapterInstruction: redeem
    })
  }

  instructions.push(redeemIxs)
  return sendAll(provider, [instructions])
}