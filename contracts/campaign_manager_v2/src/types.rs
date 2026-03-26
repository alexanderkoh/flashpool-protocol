use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolDefaults {
    pub gamma: u32,
    pub vesting_period: u32,
    pub surplus_bps: u32,
    pub ttl_thresh: u32,
    pub ttl_bump: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConfig {
    pub admin: Address,
    pub flash: Address,
    pub usdc: Address,
    pub treasury: Address,
    pub tinyamm: Address,
    pub hoops_router: Address,
    pub next_campaign_id: u32,
    pub defaults: ProtocolDefaults,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Campaign {
    pub adapter_id: i128,
    pub pool_address: Address,
    pub lp_token: Address,
    pub creator: Address,
    pub end_ledger: u32,
    pub vesting_end: u32,
    pub target_lp: i128,
    pub total_lp: i128,
    pub total_weight: i128,
    pub reward_flash: i128,
    pub bonus_flash: i128,
    pub user_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UserPos {
    pub lp: i128,
    pub weight: i128,
    pub rank: u32,
    pub joined_ledger: u32,
    pub total_reward: i128,
    pub claimed: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ActiveCampaignInfo {
    pub campaign_id: u32,
    pub end_ledger: u32,
}
