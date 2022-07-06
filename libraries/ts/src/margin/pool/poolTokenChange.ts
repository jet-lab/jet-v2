import { BN } from "@project-serum/anchor"

type PoolTokenChangeSetValue = { setValue: Record<string, never> }
type PoolTokenChangeShiftValue = { shiftValue: Record<string, never> }

export type PoolTokenChangeKind = PoolTokenChangeSetValue | PoolTokenChangeShiftValue

/**
 * TODO:
 * @export
 * @class TokenChange
 */
export class PoolTokenChange {
  /**
   * Creates an instance of Amount.
   * @param {PoolTokenChangeKind} changeKind
   * @param {BN} value
   * @memberof TokenChange
   */
  constructor(public changeKind: PoolTokenChangeKind, public value: BN) { }

  /** 
   * A `TokenChange` to be used to set the given token value
   * @static
   * @param {number | BN} value
   * @returns {PoolToken}
   * @memberof TokenChange
  */
  static setTo(value: number | BN): PoolTokenChange {
    return new PoolTokenChange({ setValue: {} }, new BN(value))
  }

  /** 
   * A `TokenChange` to be used to shift the given token value
   * @static
   * @param {number | BN} value
   * @returns {PoolTokenChange}
   * @memberof TokenChange
  */
  static shiftBy(value: number | BN): PoolTokenChange {
    return new PoolTokenChange({ shiftValue: {} }, new BN(value))
  }

  /**
   * Converts the class instance into an object that can
   * be used as an argument for Solana instruction calls.
   * @returns {{ kind: never; value: BN }}
   * @memberof TokenChange
   */
  toRpcArg(): { kind: never; tokens: BN } {
    return {
      kind: this.changeKind as never,
      tokens: this.value
    }
  }
}
