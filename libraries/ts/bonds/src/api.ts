import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { AssociatedToken, MarginAccount, MarginConfig, Pool, sendAll } from '@jet-lab/margin';
import { BondMarket } from './bondMarket';
import { AnchorProvider, BN } from '@project-serum/anchor';

const createRandomSeed = (byteLength: number) => {
    const max = 127;
    const min = 0;
    return Uint8Array.from(new Array(byteLength).fill(0).map(() => Math.ceil(Math.random() * (max - min) + min)))
}

interface IWithCreateFixedMarketAccount {
    market: BondMarket
    provider: AnchorProvider
    marginAccount: MarginAccount,
    walletAddress: PublicKey
    instructions: TransactionInstruction[],
    borrowerAccount: PublicKey
}
export const withCreateFixedMarketAccounts = async ({
    market,
    provider,
    marginAccount,
    walletAddress,
    instructions,
    borrowerAccount
}: IWithCreateFixedMarketAccount) => {
    const tokenMint = market.addresses.underlyingTokenMint;
    const ticketMint = market.addresses.bondTicketMint;
    await AssociatedToken.withCreate(instructions, provider, marginAccount.address, tokenMint);
    await AssociatedToken.withCreate(instructions, provider, marginAccount.address, ticketMint);    
    const info = await provider.connection.getAccountInfo(borrowerAccount)
    if (!info) {
        const createAccountIx = await market.registerAccountWithMarket(marginAccount, walletAddress);
        await marginAccount.withAdapterInvoke({
            instructions,
            adapterInstruction: createAccountIx
        });
    }
}

export const createFixedLendOrder = () => {

}

interface IBorrowOrder {
    market: BondMarket
    marginAccount: MarginAccount
    marginConfig: MarginConfig
    provider: AnchorProvider
    walletAddress: PublicKey
    pools: Record<string, Pool>
    currentPool: Pool
    amount: BN,
    basisPoints: BN,
    marketAccount?: string
}


export const createFixedBorrowOrder = async ({
    market,
    marginAccount,
    marginConfig,
    provider,
    walletAddress,
    pools,
    currentPool,
    amount,
    basisPoints,
    marketAccount,
}: IBorrowOrder): Promise<string> => {
    // Fail if there is no active bonds program id in the config
    if (!marginConfig.bondsProgramId) {
        throw new Error('There is no market configured on this network')
    }

    const borrowerAccount = await market.deriveMarginUserAddress(marginAccount)

    const instructions: TransactionInstruction[][]= []
    // Create relevant accounts if they do not exist
    if (!marketAccount) {
        const accountInstructions: TransactionInstruction[] = [];
        await withCreateFixedMarketAccounts({
            market,
            provider,
            marginAccount,
            walletAddress,
            instructions: accountInstructions,
            borrowerAccount
        })
        if (accountInstructions.length > 0) {
            instructions.push(accountInstructions)
        }
    }

    const borrowInstructions: TransactionInstruction[] = []

    await currentPool.withMarginRefreshAllPositionPrices({
        instructions: borrowInstructions,
        pools,
        marginAccount
    });

    const borrowOffer = await market.requestBorrowIx(
        marginAccount,
        walletAddress,
        amount,
        basisPoints,
        createRandomSeed(4)
    );
    const refreshIx = await market.program.methods
      .refreshPosition(true)
      .accounts({
        borrowerAccount,
        marginAccount: marginAccount.address,
        claimsMint: market.addresses.claimsMint,
        bondManager: market.addresses.bondManager,
        underlyingOracle: market.addresses.underlyingOracle,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .instruction();

    await marginAccount.withAdapterInvoke({
      instructions: borrowInstructions,
      adapterInstruction: refreshIx
    });

    await marginAccount.withAdapterInvoke({
        instructions: borrowInstructions,
        adapterInstruction: borrowOffer
    });

    instructions.push(borrowInstructions)
    return sendAll(provider, [instructions])
};