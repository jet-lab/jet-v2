import { BN } from "@project-serum/anchor"
import { Account, Mint } from "@solana/spl-token"
import { Number192 } from "../utils"
import { bigIntToBn } from "./associatedToken"

export class TokenAmount {
  /** Token lamports */
  lamports: BN
  /** Token amount as string for UI, accounts for decimals. Imprecise at large numbers */
  uiTokens: string
  /** Token amount as a float, accounts for decimals. Imprecise at large numbers */
  tokens: number

  /**
   * Creates an instance of TokenAmount.
   * @param {BN} lamports
   * @param {number} decimals
   * @param {PublicKey} mint
   * @memberof TokenAmount
   */
  constructor(
    /** Raw amount */
    public units: BN,
    /** Number of decimals configured for token's mint. */
    public decimals: number,
    /** Number of decimals configured for token's mint. */
    public precision: number
  ) {
    this.lamports = TokenAmount.lamportAmount(units, precision)
    this.tokens = TokenAmount.tokenAmount(units, decimals, precision)
    this.uiTokens = this.tokens.toLocaleString("fullwide", { useGrouping: true }) //to prevent scientific numbers. Grouping adds commas
  }

  /**
   * @static
   * @param {number} decimals
   * @param {PublicKey} mint
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  static zero(decimals: number): TokenAmount {
    return new TokenAmount(Number192.ZERO, decimals, 0)
  }

  /**
   * @static
   * @param {TokenAccount} account
   * @param {number} decimals
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  static account(account: Account, decimals: number): TokenAmount {
    return new TokenAmount(bigIntToBn(account.amount), decimals, 0)
  }

  /**
   * @static
   * @param {MintInfo} mint
   * @param {PublicKey} mintAddress
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  static mint(mint: Mint): TokenAmount {
    return new TokenAmount(bigIntToBn(mint.supply), mint.decimals, 0)
  }

  /**
   * @static
   * @param {string} tokenAmount
   * @param {number} decimals
   * @param {PublicKey} mint
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  static tokens(tokenAmount: string, decimals: number): TokenAmount {
    return new TokenAmount(TokenAmount.tokensToLamports(tokenAmount, decimals), decimals, 0)
  }

  /**
   * @static
   * @param {BN} lamports
   * @param {number} decimals
   * @return {TokenAmount}
   * @memberof TokenAmount
   */
  static lamports(lamports: BN, decimals: number): TokenAmount {
    return new TokenAmount(lamports, decimals, 0)
  }

  static units(units: BN, decimals: number, precision: number): TokenAmount {
    return new TokenAmount(units, decimals, precision)
  }

  static decimal(units: BN, decimals: number, precision: number) {}

  /**
   * @private
   * @static
   * @param {BN} units
   * @param {number} decimals
   * @returns {number}
   * @memberof TokenAmount
   */
  private static tokenAmount(units: BN, decimals: number, precision: number): number {
    const extraPrecision = decimals + precision
    const str = units.toString(10, extraPrecision)
    return parseFloat(str.slice(0, -extraPrecision) + "." + str.slice(-extraPrecision))
  }

  private static lamportAmount(units: BN, precision: number) {
    if (precision === 0) {
      return units
    }
    return units.div(new BN(10).pow(new BN(precision)))
  }

  /**
   * @static
   * @param {number} marketValue
   * @param {number} price
   * @param {number} decimals
   * @param {PublicKey} mint
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  static tokenPrice(marketValue: number, price: number, decimals: number): TokenAmount {
    const tokens = price !== 0 ? marketValue / price : 0
    return TokenAmount.tokens(tokens.toFixed(decimals), decimals)
  }

  /**
   * Convert a tokens string into lamports BN
   * @private
   * @static
   * @param {string} tokensStr
   * @param {number} decimals
   * @returns {BN}
   * @memberof TokenAmount
   */
  private static tokensToLamports(tokensStr: string, decimals: number): BN {
    // Convert from exponential notation (7.46e-7) to regular
    if (tokensStr.indexOf("e+") !== -1 || tokensStr.indexOf("e-") !== -1) {
      tokensStr = Number(tokensStr).toLocaleString("fullwide", { useGrouping: false })
    }

    let lamports: string = tokensStr

    // Remove commas
    while (lamports.indexOf(",") !== -1) {
      lamports = lamports.replace(",", "")
    }

    // Determine if there's a decimal, take number of
    // characters after it as fractionalValue
    let fractionalValue = 0
    const initialPlace = lamports.indexOf(".")
    if (initialPlace !== -1) {
      fractionalValue = lamports.length - (initialPlace + 1)

      // If fractional value is lesser than a lamport, round to nearest lamport
      if (fractionalValue > decimals) {
        lamports = String(parseFloat(lamports).toFixed(decimals))
      }

      // Remove decimal
      lamports = lamports.replace(".", "")
    }

    // Append zeros
    for (let i = 0; i < decimals - fractionalValue; i++) {
      lamports += "0"
    }

    // Return BN value in lamports
    return new BN(lamports)
  }

  private setPrecision(precision: number) {
    const extraPrecision = precision - this.precision
    let units: BN
    if (extraPrecision < 0) {
      units = this.units.div(new BN(10).pow(new BN(extraPrecision)))
    } else {
      units = this.units.mul(new BN(10).pow(new BN(extraPrecision)))
    }
    return TokenAmount.units(units, this.decimals, precision)
  }

  add(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.add)
  }

  public addb(b: BN): TokenAmount {
    return new TokenAmount(
      this.lamports.add(b.mul(new BN(10).pow(new BN(this.precision)))),
      this.decimals,
      this.precision
    )
  }

  public addn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.addn(b * 10 ** this.precision), this.decimals, this.precision)
  }

  sub(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.sub)
  }

  public subb(b: BN): TokenAmount {
    return new TokenAmount(
      this.lamports.sub(b.mul(new BN(10).pow(new BN(this.precision)))),
      this.decimals,
      this.precision
    )
  }

  public subn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.subn(b * 10 ** this.precision), this.decimals, this.precision)
  }

  mul(b: TokenAmount): TokenAmount {
    const lamport = TokenAmount.lamports(new BN(1), this.decimals)
    return this.do(b, BN.prototype.mul).do(lamport, BN.prototype.div)
  }

  public mulb(b: BN): TokenAmount {
    return new TokenAmount(
      this.lamports.mul(b.mul(new BN(10).pow(new BN(this.precision)))),
      this.decimals,
      this.precision
    )
  }

  public muln(b: number): TokenAmount {
    return new TokenAmount(this.lamports.muln(b * 10 ** this.precision), this.decimals, this.precision)
  }

  div(b: TokenAmount): TokenAmount {
    const lamport = TokenAmount.lamports(new BN(1), this.decimals)
    return this.do(lamport, BN.prototype.mul).do(b, BN.prototype.div)
  }

  public divb(b: BN): TokenAmount {
    return new TokenAmount(
      this.lamports.div(b.mul(new BN(10).pow(new BN(this.precision)))),
      this.decimals,
      this.precision
    )
  }

  public divn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.divn(b * 10 ** this.precision), this.decimals, this.precision)
  }

  lt(b: TokenAmount): boolean {
    return this.lamports.lt(b.lamports)
  }

  gt(b: TokenAmount): boolean {
    return this.lamports.gt(b.lamports)
  }

  eq(b: TokenAmount): boolean {
    return this.lamports.eq(b.lamports)
  }

  isZero(): boolean {
    return this.lamports.isZero()
  }

  static max(a: TokenAmount, b: TokenAmount): TokenAmount {
    function max(this: BN, b: BN) {
      return BN.max(this, b)
    }
    return a.do(b, max)
  }

  private do(b: TokenAmount, fn: (b: BN) => BN): TokenAmount {
    let a: TokenAmount = this
    if (a.decimals !== b.decimals) {
      console.warn("Decimal mismatch")
      return TokenAmount.zero(a.decimals)
    }
    if (a.precision < b.precision) {
      a = a.setPrecision(b.precision)
    } else if (b.precision < a.precision) {
      b = b.setPrecision(a.precision)
    }
    const amount = fn.call(a.units, b.units)
    return new TokenAmount(amount, a.decimals, a.precision)
  }
}
