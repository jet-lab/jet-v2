import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { FixedTermMarket, MarketAndConfig } from "./fixedTerm"
import { AnchorProvider, BN } from "@project-serum/anchor"
import { FixedTermMarketConfig, MarginAccount, Pool, PoolTokenChange } from "../margin"
import { AssociatedToken } from "../token"
import { sendAndConfirmV0 } from "../utils"

// CREATE MARKET ACCOUNT
interface IWithCreateFixedTermMarketAccount {
  market: FixedTermMarket
  provider: AnchorProvider
  marginAccount: MarginAccount
  walletAddress: PublicKey
}
export const withCreateFixedTermMarketAccounts = async ({
  market,
  provider,
  marginAccount,
  walletAddress
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
  autorollEnabled: boolean
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
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
  markets,
  autorollEnabled,
  airspaceLookupTables
}: ICreateLendOrder) => {
  const pool = pools[market.config.symbol]
  let instructions: TransactionInstruction[] = []

  const prefreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: prefreshIXS,
    pools,
    markets: markets.filter(m => m.address != market.market.address)
  })
  instructions = instructions.concat(prefreshIXS)

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress
  })
  instructions = instructions.concat(marketIXS)

  const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: postfreshIXS,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address // TODO Why this in addition to `markets`?
  })
  instructions = instructions.concat(postfreshIXS)

  const orderInstructions: TransactionInstruction[] = []

  // create lend instruction
  await pool.withWithdrawToMargin({
    instructions: orderInstructions,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  const loanOffer = await market.market.offerLoanIx(
    marginAccount,
    amount,
    basisPoints,
    walletAddress,
    marketConfig.borrowTenor,
    autorollEnabled
  )
  await marginAccount.withAdapterInvoke({
    instructions: orderInstructions,
    adapterInstruction: loanOffer
  })
  return sendAndConfirmV0(provider, [instructions.concat(orderInstructions)], airspaceLookupTables, [])
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
  autorollEnabled: boolean
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
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
  markets,
  autorollEnabled,
  airspaceLookupTables
}: ICreateBorrowOrder): Promise<string> => {
  let setupInstructions: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools,
    markets: markets.filter(m => m.address != market.market.address)
  })
  // instructions = instructions.concat(prefreshIXS)

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress
  })
  setupInstructions = setupInstructions.concat(marketIXS)

  // const postfreshIXS: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address // TODO Why this in addition to `markets`?
  })

  const orderInstructions: TransactionInstruction[] = []

  await marginAccount.withRefreshDepositPosition({
    instructions: orderInstructions,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: market.config.underlyingOracle
  })

  // Create borrow instruction
  const borrowOffer = await market.market.requestBorrowIx(
    marginAccount,
    walletAddress,
    amount,
    basisPoints,
    marketConfig.borrowTenor,
    autorollEnabled
  )

  await marginAccount.withAdapterInvoke({
    instructions: orderInstructions,
    adapterInstruction: borrowOffer
  })
  return sendAndConfirmV0(provider, [setupInstructions.concat(orderInstructions)], airspaceLookupTables, [])
}

interface ICancelOrder {
  market: MarketAndConfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  orderId: BN
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}
export const cancelOrder = async ({
  market,
  marginAccount,
  provider,
  orderId,
  pools,
  markets,
  airspaceLookupTables
}: ICancelOrder): Promise<string> => {
  let instructions: TransactionInstruction[] = []
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
  return sendAndConfirmV0(provider, [instructions], airspaceLookupTables, [])
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
  autorollEnabled: boolean
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}

export const borrowNow = async ({
  market,
  marginAccount,
  provider,
  walletAddress,
  pools,
  amount,
  markets,
  autorollEnabled,
  airspaceLookupTables
}: IBorrowNow): Promise<string> => {
  const pool = pools[market.config.symbol]

  let setupInstructions: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools,
    markets: markets.filter(m => m.address != market.market.address)
  })

  // Create relevant accounts if they do not exist
  const { marketIXS, tokenMint } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress
  })
  setupInstructions = setupInstructions.concat(marketIXS)

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address // TODO Why this in addition to `markets`?
  })

  await marginAccount.withRefreshDepositPosition({
    instructions: setupInstructions,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  // Create borrow instruction
  const orderInstructions: TransactionInstruction[] = []
  const borrowNow = await market.market.borrowNowIx(marginAccount, walletAddress, amount, autorollEnabled)

  await marginAccount.withAdapterInvoke({
    instructions: orderInstructions,
    adapterInstruction: borrowNow
  })

  const change = PoolTokenChange.shiftBy(amount)
  const source = AssociatedToken.derive(tokenMint, marginAccount.address)
  const position = await pool.withGetOrRegisterDepositPosition({ instructions: orderInstructions, marginAccount })

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
    instructions: orderInstructions,
    adapterInstruction: depositIx
  })
  return sendAndConfirmV0(provider, [setupInstructions.concat(orderInstructions)], airspaceLookupTables, [])
}

interface ILendNow {
  market: MarketAndConfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  walletAddress: PublicKey
  pools: Record<string, Pool>
  amount: BN
  markets: FixedTermMarket[]
  autorollEnabled: boolean
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}

export const lendNow = async ({
  market,
  marginAccount,
  provider,
  walletAddress,
  pools,
  amount,
  markets,
  autorollEnabled,
  airspaceLookupTables
}: ILendNow): Promise<string> => {
  const pool = pools[market.config.symbol]

  let setupInstructions: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools,
    markets: markets.filter(m => m.address != market.market.address)
  })

  // Create relevant accounts if they do not exist
  const { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: market.market,
    provider,
    marginAccount,
    walletAddress
  })
  setupInstructions = setupInstructions.concat(marketIXS)

  // TODO: why do we refresh twice?
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: setupInstructions,
    pools: [],
    markets: [market.market],
    marketAddress: market.market.address // TODO Why this in addition to `markets`?
  })

  const orderInstructions: TransactionInstruction[] = []
  await pool.withWithdrawToMargin({
    instructions: orderInstructions,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })

  // Create borrow instruction
  const lendNow = await market.market.lendNowIx(marginAccount, amount, walletAddress, autorollEnabled)

  await marginAccount.withAdapterInvoke({
    instructions: orderInstructions,
    adapterInstruction: lendNow
  })

  await marginAccount.withUpdateAllPositionBalances({ instructions: orderInstructions })

  return sendAndConfirmV0(provider, [
    setupInstructions.concat(
      orderInstructions
    )
  ], airspaceLookupTables, [])
}

interface ISettle {
  markets: MarketAndConfig[]
  selectedMarket: MarketAndConfig
  marginAccount: MarginAccount
  provider: AnchorProvider
  pools: Record<string, Pool>
  amount: BN
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}

export const settle = async ({
  markets,
  selectedMarket,
  marginAccount,
  provider,
  pools,
  amount,
  airspaceLookupTables
}: ISettle) => {
  const { market, token } = selectedMarket
  const pool = pools[token.symbol]
  const refreshInstructions: TransactionInstruction[] = []

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshInstructions,
    pools,
    markets: markets.map(m => m.market)
  })

  const settleInstructions: TransactionInstruction[] = []
  const change = PoolTokenChange.shiftBy(amount)
  const source = AssociatedToken.derive(market.addresses.underlyingTokenMint, marginAccount.address)
  const position = await pool.withGetOrRegisterDepositPosition({ instructions: settleInstructions, marginAccount })

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
    instructions: settleInstructions,
    adapterInstruction: depositIx
  })
  await marginAccount.withUpdatePositionBalance({ instructions: settleInstructions, position })
  return sendAndConfirmV0(provider, [refreshInstructions.concat(settleInstructions)], airspaceLookupTables, [])
}

interface IRepay {
  amount: BN
  marginAccount: MarginAccount
  market: MarketAndConfig
  provider: AnchorProvider
  termLoans: Array<Loan>
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}

export const repay = async ({
  marginAccount,
  market,
  amount,
  provider,
  termLoans,
  pools,
  markets,
  airspaceLookupTables
}: IRepay) => {
  let instructions: TransactionInstruction[] = []
  let refreshInstructions: TransactionInstruction[] = []

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshInstructions,
    pools,
    markets,
    marketAddress: market.market.address
  })

  await marginAccount.withRefreshDepositPosition({
    instructions: refreshInstructions,
    config: marginAccount.findTokenConfigAddress(market.token.mint),
    priceOracle: new PublicKey(market.config.underlyingOracle.valueOf())
  })

  const pool = pools[market.token.symbol]
  await pool.withWithdrawToMargin({
    instructions: instructions,
    marginAccount,
    change: PoolTokenChange.shiftBy(amount)
  })
  const source = AssociatedToken.derive(market.market.addresses.underlyingTokenMint, marginAccount.address)

  let amountLeft = new BN(amount)

  let sortedTermLoans = termLoans.sort(
    (a, b) => a.maturation_timestamp - b.maturation_timestamp || a.sequence_number - b.sequence_number
  )
  while (amountLeft.gt(new BN(0))) {
    const currentLoan = sortedTermLoans[0]
    const nextLoan = sortedTermLoans[1]
    const balance = new BN(currentLoan.remaining_balance)
    if (balance.gte(amountLeft)) {
      const ix = await market.market.repay({
        user: marginAccount,
        termLoan: currentLoan.address,
        nextTermLoan: nextLoan ? nextLoan.address : new PublicKey("11111111111111111111111111111111").toBase58(),
        payer: currentLoan.payer,
        amount: amountLeft,
        source
      })
      await marginAccount.withAdapterInvoke({
        instructions: instructions,
        adapterInstruction: ix
      })
      amountLeft = amountLeft.sub(amountLeft)
    } else {
      const ix = await market.market.repay({
        user: marginAccount,
        termLoan: currentLoan.address,
        nextTermLoan: nextLoan ? nextLoan.address : new PublicKey("11111111111111111111111111111111").toBase58(),
        payer: currentLoan.payer,
        amount: balance,
        source
      })
      await marginAccount.withAdapterInvoke({
        instructions: instructions,
        adapterInstruction: ix
      })
      amountLeft = amountLeft.sub(balance)
      sortedTermLoans.shift()
    }
  }
  const refreshIxs: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: instructions,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions = instructions.concat(refreshIxs)
  return sendAndConfirmV0(provider, [refreshInstructions.concat(instructions)], airspaceLookupTables, [])
}

interface IRedeem {
  marginAccount: MarginAccount
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
  market: MarketAndConfig
  provider: AnchorProvider
  deposits: Array<Deposit>
  airspaceLookupTables: {
    address: string
    data: Uint8Array
  }[]
}
export const redeem = async ({
  marginAccount,
  pools,
  markets,
  market,
  provider,
  deposits,
  airspaceLookupTables
}: IRedeem) => {
  let instructions: TransactionInstruction[] = []
  const refreshIxs: TransactionInstruction[] = []
  await marginAccount.withPrioritisedPositionRefresh({
    instructions: refreshIxs,
    pools,
    markets,
    marketAddress: market.market.address
  })
  instructions = instructions.concat(refreshIxs)

  const redeemIxs: TransactionInstruction[] = []
  const sortedDeposits = deposits.sort((a, b) => a.sequence_number - b.sequence_number)

  for (let i = 0; i < sortedDeposits.length; i++) {
    const deposit = sortedDeposits[i]
    const redeem = await market.market.redeemDeposit(marginAccount, deposit, market.market)
    await marginAccount.withAdapterInvoke({
      instructions: redeemIxs,
      adapterInstruction: redeem
    })
  }

  instructions = instructions.concat(redeemIxs)
  return sendAndConfirmV0(provider, [instructions], airspaceLookupTables, [])
}

interface IConfigureAutoRoll {
  account: MarginAccount
  marketAndConfig: MarketAndConfig
  provider: AnchorProvider
  walletAddress: PublicKey
  payload: {
    lendPrice: bigint
    borrowPrice: bigint
  }
}
export const configAutoroll = async ({
  account,
  marketAndConfig,
  provider,
  payload,
  walletAddress
}: IConfigureAutoRoll) => {
  let { marketIXS } = await withCreateFixedTermMarketAccounts({
    market: marketAndConfig.market,
    provider,
    marginAccount: account,
    walletAddress
  })

  const lendSetupIX = await marketAndConfig.market.configAutorollLend(account, payload.lendPrice)
  marketIXS.push(lendSetupIX)

  const borrowSetupIX = await marketAndConfig.market.configAutorollBorrow(
    account,
    payload.borrowPrice,
    new BN(marketAndConfig.config.borrowTenor > 120 ? marketAndConfig.config.borrowTenor - 30 * 60 : 100)
  ) // market tenor - 30 minutes of lead time
  marketIXS.push(borrowSetupIX)

  return sendAndConfirmV0(provider, [marketIXS], [], [])
}

interface IToggleAutorollPosition {
  position: Loan | Deposit // deposit
  provider: AnchorProvider
  marginAccount: MarginAccount
  market: FixedTermMarket
  pools: Record<string, Pool>
  markets: FixedTermMarket[]
}

export const toggleAutorollPosition = async ({
  position,
  marginAccount,
  market,
  provider,
  pools,
  markets
}: IToggleAutorollPosition) => {
  let ix: TransactionInstruction
  let tx: TransactionInstruction[] = []

  await marginAccount.withPrioritisedPositionRefresh({
    instructions: tx,
    pools,
    markets,
    marketAddress: market.address
  })

  if ("remaining_balance" in position) {
    ix = await market.toggleAutorollLoan(marginAccount, position.address)
  } else {
    ix = await market.toggleAutorollDeposit(marginAccount, position.address)
  }

  marginAccount.withAdapterInvoke({
    instructions: tx,
    adapterInstruction: ix
  })
  return sendAndConfirmV0(provider, [tx], [], [])
}
