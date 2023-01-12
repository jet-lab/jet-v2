use anchor_lang::prelude::*;

#[error_code]
pub enum FixedTermErrorCode {
    #[msg("overflow occured on checked_add")]
    ArithmeticOverflow,
    #[msg("underflow occured on checked_sub")]
    ArithmeticUnderflow,
    #[msg("bad fixed-point division")]
    FixedPointDivision,
    #[msg("owner does not own the ticket")]
    DoesNotOwnTicket,
    #[msg("signer does not own the event adapter")]
    DoesNotOwnEventAdapter,
    #[msg("this market owner does not own this market")]
    DoesNotOwnMarket,
    #[msg("queue does not have room for another event")]
    EventQueueFull,
    #[msg("failed to deserialize the SplitTicket or ClaimTicket")]
    FailedToDeserializeTicket,
    #[msg("failed to add event to the queue")]
    FailedToPushEvent,
    #[msg("ticket is not mature and cannot be claimed")]
    ImmatureTicket,
    #[msg("not enough seeds were provided for the accounts that need to be initialized")]
    InsufficientSeeds,
    #[msg("invalid auto roll configuration")]
    InvalidAutoRollConfig,
    #[msg("order price is prohibited")]
    InvalidOrderPrice,
    #[msg("this token account is not a valid position for this margin user")]
    InvalidPosition,
    #[msg("failed to invoke account creation")]
    InvokeCreateAccount,
    #[msg("failed to properly serialize or deserialize a data structure")]
    IoError,
    #[msg("this market state account is not owned by the current program")]
    MarketStateNotProgramOwned,
    #[msg("tried to access a missing adapter account")]
    MissingEventAdapter,
    #[msg("tried to access a missing split ticket account")]
    MissingSplitTicket,
    #[msg("consume_events instruction failed to consume a single event")]
    NoEvents,
    #[msg("expected additional remaining accounts, but there were none")]
    NoMoreAccounts,
    #[msg("the debt has a non-zero balance")]
    NonZeroDebt,
    #[msg("there was a problem loading the price oracle")]
    OracleError,
    #[msg("id was not found in the user's open orders")]
    OrderNotFound,
    #[msg("Orderbook is not taking orders")]
    OrderbookPaused,
    #[msg("aaob did not match or post the order. either posting is disabled or the order was too small")]
    OrderRejected,
    #[msg("price could not be accessed from oracle")]
    PriceMissing,
    #[msg("expected a term deposit with a different sequence number")]
    TermDepositHasWrongSequenceNumber,
    #[msg("expected a term loan with a different sequence number")]
    TermLoanHasWrongSequenceNumber,
    #[msg("claim ticket is not from this manager")]
    TicketNotFromManager,
    #[msg("ticket settlement account is not registered as a position in the margin account")]
    TicketSettlementAccountNotRegistered,
    #[msg("tickets are paused")]
    TicketsPaused,
    #[msg("this signer is not authorized to place a permissioned order")]
    UnauthorizedCaller,
    #[msg("underlying settlement account is not registered as a position in the margin account")]
    UnderlyingSettlementAccountNotRegistered,
    #[msg("this user does not own the user account")]
    UserDoesNotOwnAccount,
    #[msg("this adapter does not belong to the user")]
    UserDoesNotOwnAdapter,
    #[msg("this user account is not associated with this fixed term market")]
    UserNotInMarket,
    #[msg("the wrong adapter account was passed to this instruction")]
    WrongAdapter,
    #[msg("the market is configured for a different airspace")]
    WrongAirspace,
    #[msg("the signer is not authorized to perform this action in the current airspace")]
    WrongAirspaceAuthorization,
    #[msg("asks account does not belong to this market")]
    WrongAsks,
    #[msg("bids account does not belong to this market")]
    WrongBids,
    #[msg("wrong authority for this crank instruction")]
    WrongCrankAuthority,
    #[msg("event queue account does not belong to this market")]
    WrongEventQueue,
    #[msg("adapter does not belong to given market")]
    WrongMarket,
    #[msg("this market state is not associated with this market")]
    WrongMarketState,
    #[msg("wrong TicketManager account provided")]
    WrongTicketManager,
    #[msg("the wrong account was provided for the token account that represents a user's claims")]
    WrongClaimAccount,
    #[msg(
        "the wrong account was provided for the token account that represents a user's collateral"
    )]
    WrongTicketCollateralAccount,
    #[msg("the wrong account was provided for the claims token mint")]
    WrongClaimMint,
    #[msg("the wrong account was provided for the collateral token mint")]
    WrongCollateralMint,
    #[msg("wrong fee destination")]
    WrongFeeDestination,
    #[msg("wrong oracle address was sent to instruction")]
    WrongOracle,
    #[msg("wrong margin user account address was sent to instruction")]
    WrongMarginUser,
    #[msg("wrong authority for the margin user account address was sent to instruction")]
    WrongMarginUserAuthority,
    #[msg("incorrect authority account")]
    WrongProgramAuthority,
    #[msg("not the ticket mint for this fixed term market")]
    WrongTicketMint,
    #[msg("wrong underlying token mint for this fixed term market")]
    WrongUnderlyingTokenMint,
    #[msg("wrong user account address was sent to instruction")]
    WrongUserAccount,
    #[msg("wrong vault address was sent to instruction")]
    WrongVault,
    #[msg("attempted to divide with zero")]
    ZeroDivision,
    #[msg("missing authority signature")]
    MissingAuthoritySignature,
    #[msg("this deposit is not configured for an auto roll")]
    TermDepositIsNotAutoRoll,
}
