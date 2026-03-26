#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

pub mod errors;
pub mod types;
pub mod events;
pub mod rewards;
pub mod storage;

use errors::FlashError;
use types::*;
use storage::*;

#[contract]
pub struct CampaignManager;

const DEFAULT_GAMMA: u32 = 2;
const DEFAULT_VESTING: u32 = 100_800;   // ~7 days in ledgers
const DEFAULT_SURPLUS_BPS: u32 = 500;    // 5%
const DEFAULT_TTL_THRESH: u32 = 172_800; // ~10 days
const DEFAULT_TTL_BUMP: u32 = 241_920;   // ~14 days

#[contractimpl]
impl CampaignManager {
    pub fn initialize(
        e: Env,
        admin: Address,
        flash: Address,
        usdc: Address,
        treasury: Address,
        tinyamm: Address,
        hoops_router: Address,
    ) -> Result<(), FlashError> {
        if has_config(&e) {
            return Err(FlashError::AlreadyInitialized);
        }
        admin.require_auth();

        let config = CoreConfig {
            admin,
            flash,
            usdc,
            treasury,
            tinyamm,
            hoops_router,
            next_campaign_id: 0,
            defaults: ProtocolDefaults {
                gamma: DEFAULT_GAMMA,
                vesting_period: DEFAULT_VESTING,
                surplus_bps: DEFAULT_SURPLUS_BPS,
                ttl_thresh: DEFAULT_TTL_THRESH,
                ttl_bump: DEFAULT_TTL_BUMP,
            },
        };
        set_config(&e, &config);
        Ok(())
    }

    pub fn set_defaults(
        e: Env,
        admin: Address,
        defaults: ProtocolDefaults,
    ) -> Result<(), FlashError> {
        if !has_config(&e) {
            return Err(FlashError::NotInitialized);
        }
        let mut config = get_config(&e);
        if admin != config.admin {
            return Err(FlashError::NotAdmin);
        }
        admin.require_auth();

        if defaults.surplus_bps >= 10_000 {
            return Err(FlashError::InvalidBps);
        }

        config.defaults = defaults.clone();
        set_config(&e, &config);

        e.events().publish(
            (soroban_sdk::symbol_short!("defaults"),),
            defaults,
        );
        Ok(())
    }

    pub fn get_campaign(e: Env, id: u32) -> Result<Campaign, FlashError> {
        load_campaign(&e, id).ok_or(FlashError::CampaignNotFound)
    }

    pub fn get_user_position(e: Env, id: u32, user: Address) -> Result<UserPos, FlashError> {
        load_user_pos(&e, id, &user).ok_or(FlashError::PositionNotFound)
    }
}

#[cfg(test)]
mod test;
