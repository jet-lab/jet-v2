import { translateAddress } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import { findDerivedAccount } from "../../utils/pda";
import { Pool } from "./pool";
/**
 * Class that allows the creation and management of margin pools.
 */
export class PoolManager {
    constructor(programs, provider) {
        this.programs = programs;
        this.provider = provider;
        this.owner = provider.wallet.publicKey;
    }
    /**
     * Load a margin pool
     *
     * @param {{
     *     tokenMint: Address
     *     poolConfig?: MarginPoolConfig
     *     tokenConfig?: MarginTokenConfig
     *     programs?: MarginPrograms
     *   }}
     * @return {Promise<Pool>}
     * @memberof PoolManager
     */
    async load({ tokenMint, tokenConfig, programs = this.programs }) {
        const addresses = this._derive({ programs, tokenMint });
        const marginPool = new Pool(programs, addresses, tokenConfig);
        await marginPool.refresh();
        return marginPool;
    }
    /**
     * Loads all margin pools bases on the config provided to the manager
     *
     * @param {MarginPrograms} [programs=this.programs]
     * @return {Promise<Record<string, Pool>>}
     * @memberof PoolManager
     */
    async loadAll(programs = this.programs) {
        // FIXME: This could be faster with fewer round trips to rpc
        const pools = {};
        for (const poolConfig of Object.values(programs.config.tokens)) {
            const tokenConfig = programs.config.tokens[poolConfig.symbol];
            if (tokenConfig) {
                const pool = await this.load({
                    tokenMint: poolConfig.mint,
                    tokenConfig
                });
                pools[poolConfig.symbol] = pool;
            }
        }
        return pools;
    }
    setProvider(provider) {
        this.provider = provider;
    }
    setPrograms(programs) {
        this.programs = programs;
    }
    /**
     * Creates a margin pool
     * @param args  // TODO document interface
     * @returns
     */
    async create({ tokenMint, collateralWeight, maxLeverage, pythProduct, pythPrice, marginPoolConfig, provider = this.provider, programs = this.programs }) {
        const addresses = this._derive({ programs: programs, tokenMint });
        const address = addresses.marginPool;
        const ix1 = [];
        if (this.owner) {
            try {
                await this.withCreateMarginPool({
                    instructions: ix1,
                    requester: this.owner,
                    addresses,
                    address
                });
                await provider.sendAndConfirm(new Transaction().add(...ix1));
                const ix2 = [];
                await this.withConfigureMarginPool({
                    instructions: ix2,
                    requester: this.owner,
                    collateralWeight,
                    maxLeverage,
                    pythProduct,
                    pythPrice,
                    marginPoolConfig,
                    addresses,
                    address: addresses.marginPool
                });
                return await provider.sendAndConfirm(new Transaction().add(...ix2));
            }
            catch (err) {
                console.log(err);
                throw err;
            }
        }
        else {
            throw new Error("No owner keypair provided");
        }
    }
    /**
     * // TODO add description
     * @param instructions
     * @param requester
     * @param addresses
     * @param address
     */
    async withCreateMarginPool({ instructions, requester, addresses, address, programs = this.programs }) {
        const authority = findDerivedAccount(programs.config.controlProgramId);
        const feeDestination = findDerivedAccount(programs.config.controlProgramId, "margin-pool-fee-destination", address);
        const ix = await programs.control.methods
            .createMarginPool()
            .accounts({
            requester,
            authority,
            feeDestination,
            marginPool: address,
            vault: addresses.vault,
            depositNoteMint: addresses.depositNoteMint,
            loanNoteMint: addresses.loanNoteMint,
            tokenMint: addresses.tokenMint,
            tokenMetadata: addresses.tokenMetadata,
            depositNoteMetadata: addresses.depositNoteMetadata,
            loanNoteMetadata: addresses.loanNoteMetadata,
            marginPoolProgram: programs.config.marginPoolProgramId,
            metadataProgram: programs.config.metadataProgramId,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY
        })
            .instruction();
        instructions.push(ix);
    }
    /**
     * Create a margin pool by configuring the token with the control program.
     *
     * # Instructions
     *
     * - jet_control::configure_token - configures an SPL token and creates its pool
     * @param instructions
     * @param requester
     * @param collateralWeight
     * @param maxLeverage
     * @param feeDestination
     * @param pythProduct
     * @param pythPrice
     * @param marginPoolConfig
     * @param addresses
     * @param address
     */
    async withConfigureMarginPool({ instructions, requester, collateralWeight, maxLeverage, pythProduct, pythPrice, marginPoolConfig, addresses, address, programs = this.programs }) {
        // Set the token configuration, e.g. collateral weight
        const metadata = {
            tokenKind: { collateral: {} },
            collateralWeight: collateralWeight,
            maxLeverage: maxLeverage
        };
        const ix = await programs.control.methods
            .configureMarginPool({
            tokenKind: metadata.tokenKind,
            collateralWeight: metadata.collateralWeight,
            maxLeverage
        }, marginPoolConfig)
            .accounts({
            requester,
            authority: addresses.controlAuthority,
            tokenMint: addresses.tokenMint,
            marginPool: address,
            tokenMetadata: addresses.tokenMetadata,
            depositMetadata: addresses.depositNoteMetadata,
            loanMetadata: addresses.loanNoteMetadata,
            pythProduct: pythProduct,
            pythPrice: pythPrice,
            marginPoolProgram: programs.config.marginPoolProgramId,
            metadataProgram: programs.config.metadataProgramId
        })
            .instruction();
        instructions.push(ix);
    }
    /**
     * Derive accounts from tokenMint
     * @param {MarginPrograms} programs
     * @param {Address} tokenMint
     * @returns {PublicKey} Margin Pool Address
     */
    _derive({ programs, tokenMint }) {
        const tokenMintAddress = translateAddress(tokenMint);
        const programId = translateAddress(programs.config.marginPoolProgramId);
        const marginPool = findDerivedAccount(programId, tokenMintAddress);
        const vault = findDerivedAccount(programId, marginPool, "vault");
        const depositNoteMint = findDerivedAccount(programId, marginPool, "deposit-notes");
        const loanNoteMint = findDerivedAccount(programId, marginPool, "loan-notes");
        const marginPoolAdapterMetadata = findDerivedAccount(programs.config.metadataProgramId, programId);
        const tokenMetadata = findDerivedAccount(programs.config.metadataProgramId, tokenMintAddress);
        const depositNoteMetadata = findDerivedAccount(programs.config.metadataProgramId, depositNoteMint);
        const loanNoteMetadata = findDerivedAccount(programs.config.metadataProgramId, loanNoteMint);
        const controlAuthority = findDerivedAccount(programs.config.controlProgramId);
        return {
            tokenMint: tokenMintAddress,
            marginPool,
            vault,
            depositNoteMint,
            loanNoteMint,
            marginPoolAdapterMetadata,
            tokenMetadata,
            depositNoteMetadata,
            loanNoteMetadata,
            controlAuthority
        };
    }
}
//# sourceMappingURL=poolManager.js.map