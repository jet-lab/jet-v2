import { BN } from "@project-serum/anchor";
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, } from "@solana/web3.js";
import { fetchData, findDerivedAccount } from "./utils";
/**  The underlying token */
export const AssetKindToken = { underlyingToken: {} };
/** The bond tickets */
export const AssetKindTicket = { bondTicket: {} };
/**
 * A class for user level interaction with the bonds orderbook.
 *
 * Allows placing orders
 */
export class BondsUser {
    constructor(market, user, ticketAccount, marginAccount, borrowerAccount, borrowerAccountAddress) {
        this.bondMarket = market;
        this.user = user;
        this.marginAccount = marginAccount;
        this.borrowerAccount = borrowerAccount;
        this.addresses = {
            ticketAccount,
            marginAccount: marginAccount ? marginAccount.address : undefined,
            borrowerAccount: borrowerAccountAddress,
            claims: borrowerAccount ? borrowerAccount.claims : undefined,
        };
    }
    get provider() {
        return this.bondMarket.provider;
    }
    /**
     *
     * @param market The `BondMarket` this user account belongs to
     * @param user the pubkey of the signer that interacts with the market
     * @returns BondsUser
     */
    static async load(market, user) {
        const ticketAccount = await market.deriveTicketAddress(user);
        return new BondsUser(market, user, ticketAccount);
    }
    /**
     *
     * Loads a `BondsUser` given a margin account and `BondMarket`
     *
     * @param market the bond market this BondsUser is derived from
     * @param marginAccount the marginAccount
     * @returns `BondsUser`
     */
    static async loadWithMarginAccount(market, marginAccount) {
        // TODO: use margin spl accounts when change is in
        const ticketAccount = await market.deriveTicketAddress(marginAccount.owner);
        const borrowerAccountAddress = await BondsUser.deriveMarginUser(market, marginAccount.address);
        let borrowerAccount;
        try {
            const data = await fetchData(market.program.provider.connection, borrowerAccountAddress);
            borrowerAccount = market.program.coder.accounts.decode("MarginUser", data);
        }
        catch {
            borrowerAccount = undefined;
        }
        return new BondsUser(market, marginAccount.owner, ticketAccount, marginAccount, borrowerAccount, borrowerAccountAddress);
    }
    async borrowOrderIx(args) {
        if (this.marginAccount) {
            throw "Margin Account needed to place a borrow order";
        }
        const params = {
            maxBondTicketQty: args.maxBondTicketQty,
            maxUnderlyingTokenQty: args.maxUnderlyingTokenQty,
            limitPrice: args.limitPrice,
            matchLimit: args.matchLimit ?? new BN(1000),
            postOnly: args.postOnly ?? false,
            postAllowed: args.postAllowed ?? true,
            autoStake: args.autoStake ?? true,
        };
        const obligation = await BondsUser.deriveObligation(this.marginAccount.address, args.seed, this.bondMarket.program.programId);
        return await this.bondMarket.program.methods
            .marginBorrowOrder(params, new BN(args.seed))
            .accounts({
            ...this.bondMarket.addresses,
            ...this.addresses,
            obligation,
            payer: args.payer,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    async exchangeTokensForTicketsIx(amount) {
        return await this.bondMarket.exchangeTokensForTicketsIx({
            amount,
            user: this.user,
            userBondTicketVault: this.addresses.ticketAccount,
        });
    }
    async loadClaimTicket(seed) {
        const key = await this.bondMarket.deriveClaimTicketKey(this.user, seed);
        const data = (await this.bondMarket.provider.connection.getAccountInfo(key)).data;
        return await this.bondMarket.program.coder.accounts.decode("ClaimTicket", data);
    }
    /**
     *
     * @param payer Payer for pda initialization
     * @param tokenAddress Address to recieve settled tokens
     * @param ticketAddress Address to recieve settled tickets
     * @returns
     */
    async initializeMarginUser(payer, tokenAddress, ticketAddress) {
        const underlyingSettlement = tokenAddress ??
            (await getAssociatedTokenAddress(this.bondMarket.addresses.underlyingTokenMint, new PublicKey(this.user)));
        const ticketSettlement = tokenAddress ??
            (await getAssociatedTokenAddress(this.bondMarket.addresses.bondTicketMint, new PublicKey(this.user)));
        return await this.bondMarket.program.methods
            .initializeMarginUser()
            .accounts({
            ...this.bondMarket.addresses,
            ...this.addresses,
            underlyingSettlement,
            ticketSettlement,
            payer,
            rent: SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        })
            .instruction();
    }
    static async deriveMarginUser(bondMarket, marginAccountAddress) {
        return await findDerivedAccount(["margin_borrower", bondMarket.address, marginAccountAddress], bondMarket.program.programId);
    }
    static async deriveObligation(borrowerAccount, seed, programId) {
        return await findDerivedAccount(["obligation", borrowerAccount, seed], new PublicKey(programId));
    }
}
//# sourceMappingURL=bondsUser.js.map