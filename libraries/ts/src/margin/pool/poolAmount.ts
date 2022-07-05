import { BN } from "@project-serum/anchor"

type PoolAmountChangeSetValue = { setValue: Record<string, never> }
type PoolAmountChangeShiftValue = { shiftValue: Record<string, never> }

export type PoolAmountChangeKind = PoolAmountChangeSetValue | PoolAmountChangeShiftValue

/**
 * TODO:
 * @export
 * @class Amount
 */
export class PoolAmount {
  /**
   * Creates an instance of Amount.
   * @param {PoolAmountChangeKind} changeKind
   * @param {BN} value
   * @memberof Amount
   */
  constructor(public changeKind: PoolAmountChangeKind, public value: BN) { }

  /** 
   * An `Amount` to be used to set the given token value
   * @static
   * @param {number | BN} value
   * @returns {PoolAmount}
   * @memberof Amount
  */
  static setTo(value: number | BN): PoolAmount {
    return new PoolAmount({ setValue: {} }, new BN(value))
  }

  /** 
   * An `Amount` to be used to set the given token value
   * @static
   * @param {number | BN} value
   * @returns {PoolAmount}
   * @memberof Amount
  */
  static shiftBy(value: number | BN): PoolAmount {
    return new PoolAmount({ shiftValue: {} }, new BN(value))
  }

  /**
   * Converts the class instance into an object that can
   * be used as an argument for Solana instruction calls.
   * @returns {{ units: never; changeKind: never; value: BN }}
   * @memberof Amount
   */
  toRpcArg(): { kind: never; changeKind: never; value: BN } {
    return {
      kind: { tokens: {} } as never,
      changeKind: this.changeKind as never,
      value: this.value
    }
  }
}
