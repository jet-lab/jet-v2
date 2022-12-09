import { BN } from "@project-serum/anchor"
import { Account, Mint } from "@solana/spl-token"
import { bigIntToBn } from "./associatedToken"

export class TokenAmount {
  /** Raw amount of token lamports */
  public lamports: BN
  /** Number of decimals configured for token's mint. */
  public decimals: number
  /** Token amount as string for UI, accounts for decimals. Imprecise at large numbers */
  public uiTokens: string
  /** Token amount as a float, accounts for decimals. Imprecise at large numbers */
  public tokens: number

  /**
   * Creates an instance of TokenAmount.
   * @param {BN} lamports
   * @param {number} decimals
   * @param {PublicKey} mint
   * @memberof TokenAmount
   */
  constructor(lamports: BN, decimals: number) {
    if (!BN.isBN(lamports)) {
      console.warn("Amount is not a BN", lamports)
      lamports = new BN(0)
    }
    this.lamports = lamports
    this.decimals = decimals
    this.tokens = TokenAmount.tokenAmount(lamports, decimals)
    this.uiTokens = this.tokens.toLocaleString("fullwide", { useGrouping: true, maximumFractionDigits: decimals }) //to prevent scientific numbers. Grouping adds commas
  }

  /**
   * @static
   * @param {number} decimals
   * @param {PublicKey} mint
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  public static zero(decimals: number): TokenAmount {
    return new TokenAmount(new BN(0), decimals)
  }

  /**
   * Intialize a TokenAmount from the balance of a token account.
   * @static
   * @param {TokenAccount} tokenAccount
   * @param {number} decimals
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  public static account(tokenAccount: Account, decimals: number): TokenAmount {
    return new TokenAmount(bigIntToBn(tokenAccount.amount), decimals)
  }

  /**
   * @static
   * @param {MintInfo} mint
   * @param {PublicKey} mintAddress
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  public static mint(mint: Mint): TokenAmount {
    return new TokenAmount(bigIntToBn(mint.supply), mint.decimals)
  }

  /**
   * @static
   * @param {string} tokenAmount
   * @param {number} decimals
   * @param {PublicKey} mint
   * @returns {TokenAmount}
   * @memberof TokenAmount
   */
  public static tokens(tokenAmount: string | number, decimals: number): TokenAmount {
    return new TokenAmount(TokenAmount.tokensToLamports(tokenAmount, decimals), decimals)
  }

  /**
   * @static
   * @param {BN} lamports
   * @param {number} decimals
   * @return {TokenAmount}
   * @memberof TokenAmount
   */
  public static lamports(lamports: BN, decimals: number): TokenAmount {
    return new TokenAmount(lamports, decimals)
  }

  /**
   * @private
   * @static
   * @param {BN} lamports
   * @param {number} decimals
   * @returns {number}
   * @memberof TokenAmount
   */
  private static tokenAmount(lamports: BN, decimals: number): number {
    const str = lamports.toString(10, decimals)
    return parseFloat(str.slice(0, -decimals) + "." + str.slice(-decimals))
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
  public static tokenPrice(marketValue: number, price: number, decimals: number): TokenAmount {
    const tokens = price !== 0 ? marketValue / price : 0
    return TokenAmount.tokens(tokens.toFixed(decimals), decimals)
  }

  /**
   * Convert a tokens string into lamports BN
   * @private
   * @static
   * @param {string} tokens
   * @param {number} decimals
   * @returns {BN}
   * @memberof TokenAmount
   */
  private static tokensToLamports(tokens: string | number, decimals: number): BN {
    if (typeof tokens === "number") {
      tokens = tokens.toLocaleString("fullwide", { useGrouping: false })
    }
    // Convert from exponential notation (7.46e-7) to regular
    if (tokens.indexOf("e+") !== -1 || tokens.indexOf("e-") !== -1) {
      tokens = Number(tokens).toLocaleString("fullwide", { useGrouping: false })
    }

    let lamports: string = tokens

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

  public add(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.add)
  }

  public addb(b: BN): TokenAmount {
    return new TokenAmount(this.lamports.add(b), this.decimals)
  }

  public addn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.addn(b), this.decimals)
  }

  public sub(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.sub)
  }

  public subb(b: BN): TokenAmount {
    return new TokenAmount(this.lamports.sub(b), this.decimals)
  }

  public subn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.subn(b), this.decimals)
  }

  public mul(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.mul)
  }

  public mulb(b: BN): TokenAmount {
    return new TokenAmount(this.lamports.mul(b), this.decimals)
  }

  public muln(b: number): TokenAmount {
    return new TokenAmount(this.lamports.muln(b), this.decimals)
  }

  public div(b: TokenAmount): TokenAmount {
    return this.do(b, BN.prototype.div)
  }

  public divb(b: BN): TokenAmount {
    return new TokenAmount(this.lamports.div(b), this.decimals)
  }

  public divn(b: number): TokenAmount {
    return new TokenAmount(this.lamports.divn(b), this.decimals)
  }

  public lt(b: TokenAmount): boolean {
    return this.lamports.lt(b.lamports)
  }

  public lte(b: TokenAmount): boolean {
    return this.lamports.lt(b.lamports) || this.lamports.eq(b.lamports)
  }

  public gt(b: TokenAmount): boolean {
    return this.lamports.gt(b.lamports)
  }

  public gte(b: TokenAmount): boolean {
    return this.lamports.gt(b.lamports) || this.lamports.eq(b.lamports)
  }

  public eq(b: TokenAmount): boolean {
    return this.lamports.eq(b.lamports)
  }

  public isZero(): boolean {
    return this.lamports.isZero()
  }

  static min(a: TokenAmount, b: TokenAmount) {
    const callback = function (this: BN, b: BN) {
      const a: BN = this
      return BN.min(a, b)
    }

    return a.do(b, callback)
  }

  static max(a: TokenAmount, b: TokenAmount) {
    const callback = function (this: BN, b: BN) {
      const a: BN = this
      return BN.max(a, b)
    }

    return a.do(b, callback)
  }

  private do(b: TokenAmount, fn: (b: BN) => BN): TokenAmount {
    if (this.decimals !== b.decimals) {
      console.warn("Decimal mismatch")
      return TokenAmount.zero(this.decimals)
    }
    const amount = fn.call(this.lamports, b.lamports)
    return new TokenAmount(amount, this.decimals)
  }
}
