import { BN } from "@project-serum/anchor"

export class PoolTokenChangeKind {
  constructor(private byte: number) { }

  public static setTo() {
    return new PoolTokenChangeKind(0);
  }

  public static shiftBy() {
    return new PoolTokenChangeKind(1);
  }

  asByte() {
    return this.byte;
  }
}

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
    return new PoolTokenChange(PoolTokenChangeKind.setTo(), new BN(value))
  }

  /** 
   * A `TokenChange` to be used to shift the given token value
   * @static
   * @param {number | BN} value
   * @returns {PoolTokenChange}
   * @memberof TokenChange
  */
  static shiftBy(value: number | BN): PoolTokenChange {
    return new PoolTokenChange(PoolTokenChangeKind.shiftBy(), new BN(value))
  }
}