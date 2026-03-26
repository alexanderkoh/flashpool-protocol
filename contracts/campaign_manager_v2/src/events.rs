use soroban_sdk::{contracttype, Address, Env, symbol_short};

#[contracttype]
#[derive(Clone, Debug)]
pub struct CampaignCreatedEvent {
    pub id: u32,
    pub creator: Address,
    pub adapter_id: i128,
    pub pool_address: Address,
    pub reward_flash: i128,
    pub bonus_flash: i128,
    pub duration: u32,
    pub target_lp: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UserJoinedEvent {
    pub campaign_id: u32,
    pub user: Address,
    pub lp: i128,
    pub rank: u32,
    pub weight: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub campaign_id: u32,
    pub user: Address,
    pub flash_amount: i128,
    pub fully_vested: bool,
}

pub fn emit_campaign_created(e: &Env, evt: CampaignCreatedEvent) {
    e.events().publish((symbol_short!("created"),), evt);
}

pub fn emit_user_joined(e: &Env, evt: UserJoinedEvent) {
    e.events().publish((symbol_short!("joined"),), evt);
}

pub fn emit_reward_claimed(e: &Env, evt: RewardClaimedEvent) {
    e.events().publish((symbol_short!("claimed"),), evt);
}
