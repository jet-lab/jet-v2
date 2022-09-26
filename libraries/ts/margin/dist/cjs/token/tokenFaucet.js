"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TokenFaucet = void 0;
const spl_token_1 = require("@solana/spl-token");
const web3_js_1 = require("@solana/web3.js");
const associatedToken_1 = require("./associatedToken");
const anchor_1 = require("@project-serum/anchor");
class TokenFaucet {
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
    static async withAirdrop(instructions, programs, tokenMint, tokenFaucet, tokenAccount, lamports) {
        if (!programs.config.faucetProgramId) {
            throw new Error("No spl token faucet program id");
        }
        const pubkeyNonce = await web3_js_1.PublicKey.findProgramAddress([Buffer.from("faucet", "utf8")], (0, anchor_1.translateAddress)(programs.config.faucetProgramId));
        const keys = [
            { pubkey: pubkeyNonce[0], isSigner: false, isWritable: false },
            {
                pubkey: (0, anchor_1.translateAddress)(tokenMint),
                isSigner: false,
                isWritable: true
            },
            { pubkey: (0, anchor_1.translateAddress)(tokenAccount), isSigner: false, isWritable: true },
            { pubkey: spl_token_1.TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: (0, anchor_1.translateAddress)(tokenFaucet), isSigner: false, isWritable: false }
        ];
        const faucetIx = new web3_js_1.TransactionInstruction({
            programId: (0, anchor_1.translateAddress)(programs.config.faucetProgramId),
            data: Buffer.from([1, ...lamports.toArray("le", 8)]),
            keys
        });
        instructions.push(faucetIx);
    }
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
    static async airdropToken(programs, provider, faucet, user, mint, lamports) {
        const instructions = [];
        // Check for user token account
        // If it doesn't exist add instructions to create it
        const address = await associatedToken_1.AssociatedToken.withCreate(instructions, provider, user, mint);
        // Create airdrop instructions
        await this.withAirdrop(instructions, programs, mint, faucet, address, lamports);
        // Execute airdrop
        return provider.sendAndConfirm(new web3_js_1.Transaction().add(...instructions));
    }
    /** Airdrops native SOL if the mint is the native mint. */
    static async airdrop(programs, provider, lamports, token, owner = provider.wallet.publicKey) {
        const mintAddress = (0, anchor_1.translateAddress)(token.mint);
        const ownerAddress = (0, anchor_1.translateAddress)(owner);
        const faucet = token.faucet;
        const ix = [];
        const destination = associatedToken_1.AssociatedToken.derive(token.mint, owner);
        // Optionally create a token account for wallet
        if (!mintAddress.equals(spl_token_1.NATIVE_MINT) && !(await associatedToken_1.AssociatedToken.exists(provider.connection, token.mint, owner))) {
            const createTokenAccountIx = (0, spl_token_1.createAssociatedTokenAccountInstruction)(ownerAddress, destination, ownerAddress, mintAddress);
            ix.push(createTokenAccountIx);
        }
        if (mintAddress.equals(spl_token_1.NATIVE_MINT)) {
            // Sol airdrop
            // Use a specific endpoint. A hack because some devnet endpoints are unable to airdrop
            const endpoint = new web3_js_1.Connection("https://api.devnet.solana.com", anchor_1.AnchorProvider.defaultOptions().commitment);
            const airdropTxnId = await endpoint.requestAirdrop(ownerAddress, parseInt(lamports.toString()));
            await endpoint.confirmTransaction(airdropTxnId);
            return airdropTxnId;
        }
        else if (faucet) {
            // Faucet airdrop
            await this.withAirdrop(ix, programs, mintAddress, (0, anchor_1.translateAddress)(faucet), destination, lamports);
            return await provider.sendAndConfirm(new web3_js_1.Transaction().add(...ix));
        }
        else {
            // Mint to the destination token account
            const mintToIx = (0, spl_token_1.createMintToInstruction)(mintAddress, destination, ownerAddress, BigInt(lamports.toString()));
            ix.push(mintToIx);
            return await provider.sendAndConfirm(new web3_js_1.Transaction().add(...ix));
        }
    }
}
exports.TokenFaucet = TokenFaucet;
//# sourceMappingURL=tokenFaucet.js.map