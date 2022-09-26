import { BN } from "@project-serum/anchor";
import { bigIntToBn } from "./associatedToken";
export class TokenAmount {
    /**
     * Creates an instance of TokenAmount.
     * @param {BN} lamports
     * @param {number} decimals
     * @param {PublicKey} mint
     * @memberof TokenAmount
     */
    constructor(lamports, decimals) {
        if (!BN.isBN(lamports)) {
            console.warn("Amount is not a BN", lamports);
            lamports = new BN(0);
        }
        this.lamports = lamports;
        this.decimals = decimals;
        this.tokens = TokenAmount.tokenAmount(lamports, decimals);
        this.uiTokens = this.tokens.toLocaleString("fullwide", { useGrouping: true, maximumFractionDigits: decimals }); //to prevent scientific numbers. Grouping adds commas
    }
    /**
     * @static
     * @param {number} decimals
     * @param {PublicKey} mint
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static zero(decimals) {
        return new TokenAmount(new BN(0), decimals);
    }
    /**
     * Intialize a TokenAmount from the balance of a token account.
     * @static
     * @param {TokenAccount} tokenAccount
     * @param {number} decimals
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static account(tokenAccount, decimals) {
        return new TokenAmount(bigIntToBn(tokenAccount.amount), decimals);
    }
    /**
     * @static
     * @param {MintInfo} mint
     * @param {PublicKey} mintAddress
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static mint(mint) {
        return new TokenAmount(bigIntToBn(mint.supply), mint.decimals);
    }
    /**
     * @static
     * @param {string} tokenAmount
     * @param {number} decimals
     * @param {PublicKey} mint
     * @returns {TokenAmount}
     * @memberof TokenAmount
     */
    static tokens(tokenAmount, decimals) {
        return new TokenAmount(TokenAmount.tokensToLamports(tokenAmount, decimals), decimals);
    }
    /**
     * @static
     * @param {BN} lamports
     * @param {number} decimals
     * @return {TokenAmount}
     * @memberof TokenAmount
     */
    static lamports(lamports, decimals) {
        return new TokenAmount(lamports, decimals);
    }
    /**
     * @private
     * @static
     * @param {BN} lamports
     * @param {number} decimals
     * @returns {number}
     * @memberof TokenAmount
     */
    static tokenAmount(lamports, decimals) {
        const str = lamports.toString(10, decimals);
        return parseFloat(str.slice(0, -decimals) + "." + str.slice(-decimals));
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
    static tokenPrice(marketValue, price, decimals) {
        const tokens = price !== 0 ? marketValue / price : 0;
        return TokenAmount.tokens(tokens.toFixed(decimals), decimals);
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
    static tokensToLamports(tokens, decimals) {
        if (typeof tokens === "number") {
            tokens = tokens.toLocaleString("fullwide", { useGrouping: false });
        }
        // Convert from exponential notation (7.46e-7) to regular
        if (tokens.indexOf("e+") !== -1 || tokens.indexOf("e-") !== -1) {
            tokens = Number(tokens).toLocaleString("fullwide", { useGrouping: false });
        }
        let lamports = tokens;
        // Remove commas
        while (lamports.indexOf(",") !== -1) {
            lamports = lamports.replace(",", "");
        }
        // Determine if there's a decimal, take number of
        // characters after it as fractionalValue
        let fractionalValue = 0;
        const initialPlace = lamports.indexOf(".");
        if (initialPlace !== -1) {
            fractionalValue = lamports.length - (initialPlace + 1);
            // If fractional value is lesser than a lamport, round to nearest lamport
            if (fractionalValue > decimals) {
                lamports = String(parseFloat(lamports).toFixed(decimals));
            }
            // Remove decimal
            lamports = lamports.replace(".", "");
        }
        // Append zeros
        for (let i = 0; i < decimals - fractionalValue; i++) {
            lamports += "0";
        }
        // Return BN value in lamports
        return new BN(lamports);
    }
    add(b) {
        return this.do(b, BN.prototype.add);
    }
    addb(b) {
        return new TokenAmount(this.lamports.add(b), this.decimals);
    }
    addn(b) {
        return new TokenAmount(this.lamports.addn(b), this.decimals);
    }
    sub(b) {
        return this.do(b, BN.prototype.sub);
    }
    subb(b) {
        return new TokenAmount(this.lamports.sub(b), this.decimals);
    }
    subn(b) {
        return new TokenAmount(this.lamports.subn(b), this.decimals);
    }
    mul(b) {
        return this.do(b, BN.prototype.mul);
    }
    mulb(b) {
        return new TokenAmount(this.lamports.mul(b), this.decimals);
    }
    muln(b) {
        return new TokenAmount(this.lamports.muln(b), this.decimals);
    }
    div(b) {
        return this.do(b, BN.prototype.div);
    }
    divb(b) {
        return new TokenAmount(this.lamports.div(b), this.decimals);
    }
    divn(b) {
        return new TokenAmount(this.lamports.divn(b), this.decimals);
    }
    lt(b) {
        return this.lamports.lt(b.lamports);
    }
    lte(b) {
        return this.lamports.lt(b.lamports) || this.lamports.eq(b.lamports);
    }
    gt(b) {
        return this.lamports.gt(b.lamports);
    }
    gte(b) {
        return this.lamports.gt(b.lamports) || this.lamports.eq(b.lamports);
    }
    eq(b) {
        return this.lamports.eq(b.lamports);
    }
    isZero() {
        return this.lamports.isZero();
    }
    static min(a, b) {
        const callback = function (b) {
            const a = this;
            return BN.min(a, b);
        };
        return a.do(b, callback);
    }
    static max(a, b) {
        const callback = function (b) {
            const a = this;
            return BN.max(a, b);
        };
        return a.do(b, callback);
    }
    do(b, fn) {
        if (this.decimals !== b.decimals) {
            console.warn("Decimal mismatch");
            return TokenAmount.zero(this.decimals);
        }
        const amount = fn.call(this.lamports, b.lamports);
        return new TokenAmount(amount, this.decimals);
    }
}
//# sourceMappingURL=tokenAmount.js.map