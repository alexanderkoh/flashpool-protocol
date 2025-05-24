use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, Symbol, Val, symbol_short};

use crate::Campaign;
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConfig {
    pub admin: Address,
    pub flash: Address,
    pub usdc: Address,
    pub core_pair: Option<Address>,
    pub next: u32,
    pub surplus_bps: u32,
    pub ttl_thresh: u32,
    pub ttl_bump: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveCampaignInfo {
    pub campaign_id: u32,
    pub end_ledger: u32,
}

pub const PREFIX_UCNT: &str = "UC";
pub const PREFIX_CAMP: &str = "C";
pub const PREFIX_UPOS: &str = "U";

pub fn camp_key(e:&Env,id:u32)->Val { (PREFIX_CAMP,id).into_val(e) }
pub fn upos_key(e:&Env,id:u32,w:&Address)->Val { (PREFIX_UPOS,id,w).into_val(e) }
pub fn ucnt_key(e: &Env, id: u32) -> Val { (PREFIX_UCNT, id).into_val(e) }

pub fn load_camp(e:&Env,id:u32)->Campaign {
    e.storage().instance().get::<Val,Campaign>(&camp_key(e,id)).unwrap_optimized()
}
pub fn save_camp(e:&Env,id:u32,c:&Campaign){
    e.storage().instance().set::<Val,Campaign>(&camp_key(e,id),c)
}


pub fn set_core_config(e: &Env, config: &CoreConfig) {
    e.storage().instance().set(&KEY_CORE_CONFIG, config);
}
pub fn get_core_config(e: &Env) -> CoreConfig {
    e.storage().instance().get(&KEY_CORE_CONFIG).unwrap_optimized()
}
#[allow(dead_code)]
pub fn update_core_config<F: FnOnce(&mut CoreConfig)>(e: &Env, f: F) {
    let mut config = get_core_config(e);
    f(&mut config);
    set_core_config(e, &config);
}

pub fn set_active_campaign_for_pair(e: &Env, pair: &Address, info: &ActiveCampaignInfo) {
    e.storage().instance().set(pair, info);
}

pub fn get_active_campaign_for_pair(e: &Env, pair: &Address) -> Option<ActiveCampaignInfo> {
    e.storage().instance().get(pair)
}

#[allow(dead_code)]
pub fn clear_active_campaign_for_pair(e: &Env, pair: &Address) {
    e.storage().instance().remove(pair);
}

pub const KEY_CORE_CONFIG: Symbol = symbol_short!("CONFIG");