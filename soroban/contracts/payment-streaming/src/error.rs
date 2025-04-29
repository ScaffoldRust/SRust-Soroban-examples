use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PaymentStreamingError {
    ChannelAlreadyClosed = 1,
    InvalidFinalState = 2,
    ChannelNotFound = 3,
    ChannelIsClosed = 4,
    InvalidAmount = 5,
    InvalidParameters = 6,
    StreamNotFound = 7,
    StreamAlreadyInactive = 8,
    StreamNotActive = 9,
    StreamAllreadyPaused = 10,
    Channelisclosed = 11,
    Invalidamount = 12,
    InsufficientFunds = 13,
    InvalidDeposit = 14
}