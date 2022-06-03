import { BN } from "@project-serum/anchor"

type PoolAmountKindTokens = { tokens: Record<string, never> }
type PoolAmountKindNotes = { notes: Record<string, never> }

export type PoolAmountKind = PoolAmountKindTokens | PoolAmountKindNotes

/**
 * TODO:
 * @export
 * @class Amount
 */
export class PoolAmount {
  /**
   * Creates an instance of Amount.
   * @param {PoolAmountKind} kind
   * @param {BN} value
   * @memberof Amount
   */
  constructor(public kind: PoolAmountKind, public value: BN) {}

  /**
   * TODO:
   * @static
   * @param {(number | BN)} amount
   * @returns {PoolAmount}
   * @memberof Amount
   */
  static tokens(amount: number | BN): PoolAmount {
    return new PoolAmount({ tokens: {} }, new BN(amount))
  }

  /**
   * TODO:
   * @static
   * @param {(number | BN)} amount
   * @returns {PoolAmount}
   * @memberof Amount
   */
  static notes(amount: number | BN): PoolAmount {
    return new PoolAmount({ notes: {} }, new BN(amount))
  }

  /**
   * Converts the class instance into an object that can
   * be used as an argument for Solana instruction calls.
   * @returns {{ units: never; value: BN }}
   * @memberof Amount
   */
  toRpcArg(): { kind: never; value: BN } {
    return {
      kind: this.kind as never,
      value: this.value
    }
  }
}
