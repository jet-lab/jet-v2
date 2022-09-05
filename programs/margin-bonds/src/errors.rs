use anchor_lang::prelude::*;

#[error_code]
pub enum BondsError {
    #[msg("overflow occured on checked_add")]
    ArithmeticOverflow,
    #[msg("underflow occured on checked_sub")]
    ArithmeticUnderflow,
    #[msg("owner does not own the ticket")]
    DoesNotOwnTicket,
    #[msg("signer does not own the event adapter")]
    DoesNotOwnEventAdapter,
    #[msg("queue does not have room for another event")]
    EventQueueFull,
    #[msg("failed to deserialize the SplitTicket or ClaimTicket")]
    FailedToDeserializeTicket,
    #[msg("bond is not mature and cannot be claimed")]
    ImmatureBond,
    #[msg("not enough seeds were provided for the accounts that need to be initialized")]
    InsufficientSeeds,
    #[msg("the wrong event type was unwrapped\nthis condition should be impossible, and does not result from invalid input")]
    InvalidEvent,
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
    #[msg("there was a problem loading the price oracle")]
    OracleError,
    #[msg("id was not found in the user's open orders")]
    OrderNotFound,
    #[msg("Orderbook is not taking orders")]
    OrderbookPaused,
    #[msg("price could not be accessed from oracle")]
    PriceMissing,
    #[msg("claim ticket is not from this manager")]
    TicketNotFromManager,
    #[msg("this signer is not authorized to place a permissioned order")]
    UnauthorizedCaller,
    #[msg("this user does not own the user account")]
    UserDoesNotOwnAccount,
    #[msg("this adapter does not belong to the user")]
    UserDoesNotOwnAdapter,
    #[msg("this user account is not associated with this bond market")]
    UserNotInMarket,
    #[msg("asks account does not belong to this market")]
    WrongAsks,
    #[msg("bids account does not belong to this market")]
    WrongBids,
    #[msg("adapter does not belong to given bond manager")]
    WrongBondManager,
    #[msg("wrong authority for this crank instruction")]
    WrongCrankAuthority,
    #[msg("event queue account does not belong to this market")]
    WrongEventQueue,
    #[msg("this market state is not associated with this market")]
    WrongMarketState,
    #[msg("wrong TicketManager account provided")]
    WrongTicketManager,
    #[msg("this market owner does not own this market")]
    DoesNotOwnMarket,
    #[msg("the wrong account was provided for the token account that represents a user's claims")]
    WrongClaimAccount,
    #[msg("the wrong account was provided for the claims token mint")]
    WrongClaimMint,
    #[msg("the wrong account was provided for the claims token mint")]
    WrongDepositsMint,
    #[msg("wrong oracle address was sent to instruction")]
    WrongOracle,
    #[msg("wrong margin borrower account address was sent to instruction")]
    WrongMarginUser,
    #[msg("incorrect authority account")]
    WrongProgramAuthority,
    #[msg("not the ticket mint for this bond market")]
    WrongTicketMint,
    #[msg("wrong vault address was sent to instruction")]
    WrongVault,
    #[msg("attempted to divide with zero")]
    ZeroDivision,
}
