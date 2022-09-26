import { Address, BN, AnchorProvider } from "@project-serum/anchor";
import { MarginPrograms, MarginTokenConfig } from "../margin";
export declare class TokenFaucet {
    /**
     * TODO:
     * @private
     * @static
     * @param {TransactionInstruction[]} instructions
     * @param {PublicKey} tokenMint
     * @param {PublicKey} tokenFaucet
     * @param {PublicKey} tokenAccount
     * @memberof TokenFaucet
     */
    private static withAirdrop;
    /**
     * TODO:
     * @static
     * @param {AnchorProvider} provider
     * @param {Address} faucet
     * @param {Address} user
     * @param {Address} mint
     * @returns {Promise<string>}
     * @memberof TokenFaucet
     */
    static airdropToken(programs: MarginPrograms, provider: AnchorProvider, faucet: Address, user: Address, mint: Address, lamports: BN): Promise<string>;
    /** Airdrops native SOL if the mint is the native mint. */
    static airdrop(programs: MarginPrograms, provider: AnchorProvider, lamports: BN, token: MarginTokenConfig, owner?: Address): Promise<string>;
}
//# sourceMappingURL=tokenFaucet.d.ts.map