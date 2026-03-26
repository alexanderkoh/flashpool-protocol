use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FlashError {
    // Initialization
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization
    NotAdmin = 10,
    NotAuthorized = 11,

    // Campaign lifecycle
    CampaignNotFound = 20,
    CampaignNotEnded = 21,
    CampaignAlreadyEnded = 22,
    CampaignActiveForPool = 23,

    // User operations
    NothingToClaim = 30,
    InsufficientAmount = 31,
    PositionNotFound = 32,

    // External calls
    SwapFailed = 40,
    SlippageExceeded = 41,
    AdapterCallFailed = 42,
    InsufficientRewardPool = 43,

    // Config
    InvalidBps = 50,
    InvalidDuration = 51,
    InvalidTarget = 52,
}
