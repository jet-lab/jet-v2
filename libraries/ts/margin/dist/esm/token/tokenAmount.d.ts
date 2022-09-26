import { BN } from "@project-serum/anchor";
import { Account, Mint } from "@solana/spl-token";
export declare class TokenAmount {
    /** Raw amount of token lamports */
    lamports: BN;
    /** Number of decimals configured for token's mint. */
    decimals: number;
    /** Token amount as string for UI, accounts for decimals. Imprecise at large numbers */
    uiTokens: string;
    /** Token amount as a float, accounts for decimals. Imprecise at large numbers */
    tokens: number;
    /**
     * Creates an instance of TokenAmount.
     * @param {BN} lamports
     * @param {number} decimals
     * @param {PublicKey} mint
     * @memberof TokenAmount
     */
    constructor(lamports: BN, decimals: number);
    /**
     * @static
     * @param {number} decimals
     * @param {PublicKey} mint
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static zero(decimals: number): TokenAmount;
    /**
     * Intialize a TokenAmount from the balance of a token account.
     * @static
     * @param {TokenAccount} tokenAccount
     * @param {number} decimals
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static account(tokenAccount: Account, decimals: number): TokenAmount;
    /**
     * @static
     * @param {MintInfo} mint
     * @param {PublicKey} mintAddress
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static mint(mint: Mint): TokenAmount;
    /**
     * @static
     * @param {string} tokenAmount
     * @param {number} decimals
     * @param {PublicKey} mint
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static tokens(tokenAmount: string | number, decimals: number): TokenAmount;
    /**
     * @static
     * @param {BN} lamports
     * @param {number} decimals
     * @return {TokenAmount}
     * @memberof TokenAmount
     */
    static lamports(lamports: BN, decimals: number): TokenAmount;
    /**
     * @private
     * @static
     * @param {BN} lamports
     * @param {number} decimals
     * @returns {number}
     * @memberof TokenAmount
     */
    private static tokenAmount;
    /**
     * @static
     * @param {number} marketValue
     * @param {number} price
     * @param {number} decimals
     * @param {PublicKey} mint
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static tokenPrice(marketValue: number, price: number, decimals: number): TokenAmount;
    /**
     * Convert a tokens string into lamports BN
     * @private
     * @static
     * @param {string} tokens
     * @param {number} decimals
     * @returns {BN}
     * @memberof TokenAmount
     */
    private static tokensToLamports;
    add(b: TokenAmount): TokenAmount;
    addb(b: BN): TokenAmount;
    addn(b: number): TokenAmount;
    sub(b: TokenAmount): TokenAmount;
    subb(b: BN): TokenAmount;
    subn(b: number): TokenAmount;
    mul(b: TokenAmount): TokenAmount;
    mulb(b: BN): TokenAmount;
    muln(b: number): TokenAmount;
    div(b: TokenAmount): TokenAmount;
    divb(b: BN): TokenAmount;
    divn(b: number): TokenAmount;
    lt(b: TokenAmount): boolean;
    lte(b: TokenAmount): boolean;
    gt(b: TokenAmount): boolean;
    gte(b: TokenAmount): boolean;
    eq(b: TokenAmount): boolean;
    isZero(): boolean;
    static min(a: TokenAmount, b: TokenAmount): TokenAmount;
    static max(a: TokenAmount, b: TokenAmount): TokenAmount;
    private do;
}
//# sourceMappingURL=tokenAmount.d.ts.map