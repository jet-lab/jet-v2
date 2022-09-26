import { Address, AnchorProvider } from "@project-serum/anchor";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { MarginTokenConfig } from "../config";
import { MarginPrograms } from "../marginClient";
import { PoolAddresses, Pool } from "./pool";
import { MarginPoolConfigData } from "./state";
interface IPoolCreationParams {
    tokenMint: Address;
    collateralWeight: number;
    maxLeverage: number;
    pythProduct: Address;
    pythPrice: Address;
    marginPoolConfig: MarginPoolConfigData;
    provider?: AnchorProvider;
    programs?: MarginPrograms;
}
/**
 * Class that allows the creation and management of margin pools.
 */
export declare class PoolManager {
    programs: MarginPrograms;
    provider: AnchorProvider;
    owner: PublicKey;
    constructor(programs: MarginPrograms, provider: AnchorProvider);
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
    load({ tokenMint, tokenConfig, programs }: {
        tokenMint: Address;
        tokenConfig: MarginTokenConfig;
        programs?: MarginPrograms;
    }): Promise<Pool>;
    /**
     * Loads all margin pools bases on the config provided to the manager
     *
     * @param {MarginPrograms} [programs=this.programs]
     * @return {Promise<Record<string, Pool>>}
     * @memberof PoolManager
     */
    loadAll(programs?: MarginPrograms): Promise<Record<string, Pool>>;
    setProvider(provider: AnchorProvider): void;
    setPrograms(programs: MarginPrograms): void;
    /**
     * Creates a margin pool
     * @param args  // TODO document interface
     * @returns
     */
    create({ tokenMint, collateralWeight, maxLeverage, pythProduct, pythPrice, marginPoolConfig, provider, programs }: IPoolCreationParams): Promise<string>;
    /**
     * // TODO add description
     * @param instructions
     * @param requester
     * @param addresses
     * @param address
     */
    withCreateMarginPool({ instructions, requester, addresses, address, programs }: {
        instructions: TransactionInstruction[];
        requester: Address;
        addresses: PoolAddresses;
        address: PublicKey;
        programs?: MarginPrograms;
    }): Promise<void>;
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
    withConfigureMarginPool({ instructions, requester, collateralWeight, maxLeverage, pythProduct, pythPrice, marginPoolConfig, addresses, address, programs }: {
        instructions: TransactionInstruction[];
        requester: Address;
        collateralWeight: number;
        maxLeverage: number;
        pythProduct: Address;
        pythPrice: Address;
        marginPoolConfig: MarginPoolConfigData;
        addresses: PoolAddresses;
        address: PublicKey;
        programs?: MarginPrograms;
    }): Promise<void>;
    /**
     * Derive accounts from tokenMint
     * @param {MarginPrograms} programs
     * @param {Address} tokenMint
     * @returns {PublicKey} Margin Pool Address
     */
    private _derive;
}
export {};
//# sourceMappingURL=poolManager.d.ts.map