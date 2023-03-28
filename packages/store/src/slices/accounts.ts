import { initWebsocket } from '../websocket';
import { StateCreator } from 'zustand';
import { JetStore } from '../store';
import { Number128 } from 'utils/Number128';
import BN from 'bn.js';
import { PriceInfo } from './prices';
import { TokenConfigInfo } from '@jet-lab/margin/dist/margin/tokenConfig';

const SETUP_LEVERAGE_FRACTION = Number128.fromDecimal(new BN(50), -2);

// interface WalletToken {
//   address: string;
//   symbol: string;
//   amount: number;
// }

interface Wallet {
  pubkey: string;
  accounts: Record<string, MarginAccountData>;
  selectedMarginAccount: string;
  // tokens: Record<string, WalletToken>;
}

export interface AccountsSlice {
  accounts: Record<string, Wallet>;
  selectedWallet: string | null;
  connectWallet: (wallet: string) => void;
  disconnectWallet: () => void;
  updateMarginAccount: (update: MarginAccountUpdate) => void;
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => void;
  selectMarginAccount: (address: string) => void;
}

export const createAccountsSlice: StateCreator<JetStore, [['zustand/devtools', never]], [], AccountsSlice> = (set, get) => ({
  accounts: {},
  selectedWallet: null, // TODO: maybe we can mak this non-nullable?
  connectWallet: async wallet => {
    set(() => {
      const cluster = get().settings.cluster
      initWebsocket(cluster, wallet)
      return ({ selectedWallet: wallet })
    }, false, 'CONNECT_WALLET');
  },
  disconnectWallet: () => set(() => ({ selectedWallet: null }), false, 'DISCONNECT_WALLET'),
  updateMarginAccount: (update: MarginAccountUpdate) => {
    return set(
      state => {
        if (!state.selectedWallet) {
          return state
        }
        const wallet = state.accounts[state.selectedWallet]
        const account = wallet.accounts[update.address];
        const valuation = getValuation(update.positions, [], {})
        const summary = getSummary(update.positions, valuation)
        // const borrowed_tokens = Number192.fromBits(update.borrowed_tokens).toNumber() / 10 ** account.decimals;
        // const deposit_tokens = update.deposit_tokens / 10 ** account.decimals;
        return {
          accounts: {
            ...state.accounts,
            [state.selectedWallet]: {
              ...wallet,
              accounts: {
                ...wallet.accounts,
                [update.address]: {
                  ...account,
                  valuation,
                  summary
                }
              }
            }
          }
        };
      },
      false,
      'UPDATE_MARGIN_ACCOUNT'
    );
  },
  initAllMarginAccounts: (update: Record<string, MarginAccountData>) => {
    // on init select first margin account if no other margin account is selected
    const keys = Object.keys(update);
    return set(
      state => ({
        ...state,
        marginAccounts: update,
        selectedAccountKey: keys.includes(String(state.selectMarginAccount)) ? state.selectMarginAccount : keys[0]
      }),
      true,
      'INIT_MARGIN_ACCOUNTS'
    );
  },
  selectMarginAccount: (address: string) => set(() => ({ selectedPoolKey: address }), false, 'SELECT_MARGIN_ACCOUNT')
});


export interface MarginAccountData {
  address: string;
  owner: string;
  airspace: string;
  positions: MarginPosition[],
  // poolPositions: Record<string, PoolPosition>;
  valuation: Valuation;
  summary: AccountSummary;
}

export interface MarginAccountUpdate {
  address: string;
  positions: MarginPosition[];
  // poolPositions: Record<string, PoolPosition>;
}

export interface MarginPosition {
  adapter: string;
  address: string;
  balance: number;
  balanceTimestamp: number;
  exponent: number;
  kind: 'Collateral' | 'AdapterCollateral' | 'Claim';
  maxStaleness: number;
  price: any; // TODO
  token: string;
  value: string; // Number192 formatted as decimal string
  valueModifier: number;
}

// TODO: pool positions have to factor in accrued interest
const getValuation = (
  positions: MarginPosition[],
  // TODO: don't import this, we should figure out where to get this data
  tokenConfigInfo: TokenConfigInfo[],
  prices: Record<string, PriceInfo>
): Valuation => {
  let pastDue = false
  let liabilities = Number128.ZERO
  let requiredCollateral = Number128.ZERO
  let requiredSetupCollateral = Number128.ZERO
  let weightedCollateral = Number128.ZERO
  // TODO: should we be showing users stale collateral if we know all prices in the UI?
  // const staleCollateralList: [string, ErrorCode][] = []
  // const claimErrorList: [PublicKey, ErrorCode][] = []

  for (const position of positions) {
    if (position.balance === 0) { // TODO: or kind === 'NoValue'
      // No need to update
      continue;
    }
    const kind = position.kind
    // const value = Number128.from(new BN(position.value))
    // Get the price
    let price = Number128.ZERO
    let value = Number128.ZERO
    if (kind === 'AdapterCollateral') {
      let tokenConfig = tokenConfigInfo.find(i => i.underlyingMint.toBase58() === position.token);
      // TODO handle nullability
      let p = prices[tokenConfig!.underlyingMint.toBase58()]
      price = Number128.fromDecimal(new BN(p.price), -8) // TODO: don't hardcode
    } else if (kind === 'Collateral') {
      // TODO
    } else if (kind === 'Claim') {
      // TODO
    }

    if (kind === 'AdapterCollateral' || kind === 'Collateral') {
      // TODO
      // weightedCollateral = weightedCollateral.add(value.mul(position.valueModifier))
    } else if (kind === 'Claim') {
      // TODO: pastDue
      // if (
      //     position.balance.gt(new BN(0)) &&
      //     (position.flags & AdapterPositionFlags.PastDue) === AdapterPositionFlags.PastDue
      // ) {
      //     pastDue = true
      // }
      liabilities = liabilities.add(value)
      // TODO
      // requiredCollateral = requiredCollateral.add(position.requiredCollateralValue())
      // requiredSetupCollateral = requiredSetupCollateral.add(
      //     position.requiredCollateralValue(SETUP_LEVERAGE_FRACTION)
      // )
    }
  }

  const effectiveCollateral = weightedCollateral.sub(liabilities)

  return {
    liabilities,
    pastDue,
    requiredCollateral,
    requiredSetupCollateral,
    weightedCollateral,
    effectiveCollateral,
    get availableCollateral(): Number128 {
      return effectiveCollateral.sub(requiredCollateral)
    },
    get availableSetupCollateral(): Number128 {
      return effectiveCollateral.sub(requiredSetupCollateral)
    }
  }
}


const getSummary = (positions: MarginPosition[], valuation: Valuation): AccountSummary => {
  let collateralValue = Number128.ZERO

  for (const position of positions) {
    const kind = position.kind
    if (kind === 'Collateral' || kind === 'AdapterCollateral') {
      const value = Number128.from(new BN(position.value))
      collateralValue = collateralValue.add(value)
    }
  }

  const equity = collateralValue.sub(valuation.liabilities)

  const exposureNumber = valuation.liabilities.toNumber()
  const cRatio = exposureNumber === 0 ? Infinity : collateralValue.toNumber() / exposureNumber
  const minCRatio = exposureNumber === 0 ? 1 : 1 + valuation.effectiveCollateral.toNumber() / exposureNumber
  const depositedValue = collateralValue.toNumber()
  const borrowedValue = valuation.liabilities.toNumber()
  const accountBalance = equity.toNumber()

  let leverage = 1.0
  if (valuation.liabilities.gt(Number128.ZERO)) {
    if (equity.lt(Number128.ZERO) || equity.eq(Number128.ZERO)) {
      leverage = Infinity
    } else {
      leverage = collateralValue.div(equity).toNumber()
    }
  }

  const availableCollateral = valuation.effectiveCollateral.sub(valuation.requiredCollateral).toNumber()
  return {
    depositedValue,
    borrowedValue,
    accountBalance,
    availableCollateral,
    leverage,
    riskIndicator: 0 // TODO
  }
}


/** A summation of the USD values of various positions used in margin accounting. */
export interface Valuation {
  liabilities: Number128
  requiredCollateral: Number128
  requiredSetupCollateral: Number128
  weightedCollateral: Number128
  effectiveCollateral: Number128
  availableCollateral: Number128
  availableSetupCollateral: Number128
  pastDue: boolean
}

export interface AccountSummary {
  depositedValue: number
  borrowedValue: number
  accountBalance: number
  availableCollateral: number
  leverage: number
  riskIndicator: number
}