use soroban_sdk::{symbol_short, Address, Env, Symbol, Val, IntoVal};
use crate::types::*;

// Instance storage key
pub const KEY_CONFIG: Symbol = symbol_short!("CONFIG");

// Persistent storage key prefixes
const PREFIX_CAMP: &str = "C";
const PREFIX_UPOS: &str = "U";

// --- Instance storage (CoreConfig) ---

pub fn set_config(e: &Env, config: &CoreConfig) {
    e.storage().instance().set(&KEY_CONFIG, config);
}

pub fn get_config(e: &Env) -> CoreConfig {
    e.storage().instance().get(&KEY_CONFIG).unwrap()
}

pub fn has_config(e: &Env) -> bool {
    e.storage().instance().has(&KEY_CONFIG)
}

pub fn bump_instance(e: &Env) {
    let config = get_config(e);
    e.storage()
        .instance()
        .extend_ttl(config.defaults.ttl_thresh, config.defaults.ttl_bump);
}

// --- Persistent storage (Campaign) ---

fn camp_key(e: &Env, id: u32) -> Val {
    (PREFIX_CAMP, id).into_val(e)
}

pub fn save_campaign(e: &Env, id: u32, c: &Campaign) {
    let key = camp_key(e, id);
    e.storage().persistent().set::<Val, Campaign>(&key, c);
}

pub fn load_campaign(e: &Env, id: u32) -> Option<Campaign> {
    let key = camp_key(e, id);
    e.storage().persistent().get::<Val, Campaign>(&key)
}

pub fn bump_campaign(e: &Env, id: u32, thresh: u32, bump: u32) {
    let key = camp_key(e, id);
    if e.storage().persistent().has(&key) {
        e.storage().persistent().extend_ttl(&key, thresh, bump);
    }
}

// --- Persistent storage (UserPos) ---

fn upos_key(e: &Env, id: u32, user: &Address) -> Val {
    (PREFIX_UPOS, id, user).into_val(e)
}

pub fn save_user_pos(e: &Env, id: u32, user: &Address, pos: &UserPos) {
    let key = upos_key(e, id, user);
    e.storage().persistent().set::<Val, UserPos>(&key, pos);
}

pub fn load_user_pos(e: &Env, id: u32, user: &Address) -> Option<UserPos> {
    let key = upos_key(e, id, user);
    e.storage().persistent().get::<Val, UserPos>(&key)
}

pub fn remove_user_pos(e: &Env, id: u32, user: &Address) {
    let key = upos_key(e, id, user);
    e.storage().persistent().remove(&key);
}

pub fn bump_user_pos(e: &Env, id: u32, user: &Address, thresh: u32, bump: u32) {
    let key = upos_key(e, id, user);
    if e.storage().persistent().has(&key) {
        e.storage().persistent().extend_ttl(&key, thresh, bump);
    }
}

// --- Active campaign per pool (instance storage) ---

pub fn set_active_campaign(e: &Env, pool: &Address, info: &ActiveCampaignInfo) {
    e.storage().instance().set(pool, info);
}

pub fn get_active_campaign(e: &Env, pool: &Address) -> Option<ActiveCampaignInfo> {
    e.storage().instance().get(pool)
}
