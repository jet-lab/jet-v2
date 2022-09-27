import { BN } from "@project-serum/anchor"
import { TokenAmount } from "../../"

export class PoolTokenChangeKind {
  constructor(private kind: PoolTokenChangeKindType) {}

  public static setTo() {
    return new PoolTokenChangeKind({ setTo: {} })
  }

  public static shiftBy() {
    return new PoolTokenChangeKind({ shiftBy: {} })
  }

  asParam() {
    return this.kind
  }

  isShiftBy(): boolean {
    return "shiftBy" in this.kind
  }
}

export type PoolTokenChangeKindType = { setTo: {} } | { shiftBy: {} }

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
  constructor(public changeKind: PoolTokenChangeKind, public value: BN) {}

  /**
   * A `TokenChange` to be used to set the given token value
   * @static
   * @param {TokenAmount | BN | number} value
   * @returns {PoolToken}
   * @memberof TokenChange
   */
  static setTo(value: TokenAmount | BN | number): PoolTokenChange {
    return new PoolTokenChange(
      PoolTokenChangeKind.setTo(),
      typeof value === "object" && "lamports" in value ? value.lamports : new BN(value)
    )
  }

  /**
   * A `TokenChange` to be used to shift the given token value
   * @static
   * @param {TokenAmount | BN| number} value
   * @returns {PoolTokenChange}
   * @memberof TokenChange
   */
  static shiftBy(value: TokenAmount | BN | number): PoolTokenChange {
    return new PoolTokenChange(
      PoolTokenChangeKind.shiftBy(),
      typeof value === "object" && "lamports" in value ? value.lamports : new BN(value)
    )
  }
}
