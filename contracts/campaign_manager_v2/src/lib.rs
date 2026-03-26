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

    pub fn create_campaign(
        e: Env,
        fee_usdc: i128,
        adapter_id: i128,
        pool_address: Address,
        lp_token: Address,
        duration: u32,
        target_lp: i128,
        bonus_flash: i128,
        min_reward_flash: i128,
        creator: Address,
    ) -> Result<u32, FlashError> {
        if !has_config(&e) {
            return Err(FlashError::NotInitialized);
        }
        creator.require_auth();
        bump_instance(&e);

        let mut config = get_config(&e);

        if fee_usdc <= 0 {
            return Err(FlashError::InsufficientAmount);
        }
        if duration == 0 {
            return Err(FlashError::InvalidDuration);
        }
        if target_lp <= 0 {
            return Err(FlashError::InvalidTarget);
        }

        // Check no active campaign for this pool
        if let Some(info) = get_active_campaign(&e, &pool_address) {
            if info.end_ledger > e.ledger().sequence() {
                return Err(FlashError::CampaignActiveForPool);
            }
        }

        let cca = e.current_contract_address();
        let usdc_client = soroban_sdk::token::Client::new(&e, &config.usdc);
        let flash_client = soroban_sdk::token::Client::new(&e, &config.flash);

        // 1. Take USDC from creator
        usdc_client.transfer(&creator, &cca, &fee_usdc);

        // 2. Calculate split
        let surplus_bps = config.defaults.surplus_bps;
        let (reserve_usdc, reserve_flash) = Self::get_tinyamm_reserves(&e, &config);
        let s_min = int_sqrt(
            (reserve_usdc as u128) * (reserve_usdc as u128 + fee_usdc as u128),
        ) as i128
            - reserve_usdc;
        let swap_amount = (s_min + fee_usdc * surplus_bps as i128 / 10_000i128)
            .min(fee_usdc);
        let liq_amount = fee_usdc - swap_amount;

        // 3. Swap USDC → FLASH via TinyAMM
        usdc_client.approve(&cca, &config.tinyamm, &swap_amount, &(e.ledger().sequence() + 1000));
        let flash_out = Self::tinyamm_swap(
            &e, &config.tinyamm, &cca, &config.usdc, swap_amount, 0,
        );
        if flash_out <= 0 {
            return Err(FlashError::SwapFailed);
        }

        // 4. Calculate balanced liquidity amounts
        let (r_usdc_post, r_flash_post) = Self::get_tinyamm_reserves(&e, &config);
        let flash_needed = if r_usdc_post > 0 {
            liq_amount
                .checked_mul(r_flash_post)
                .unwrap_or(0)
                .checked_div(r_usdc_post)
                .unwrap_or(0)
        } else {
            0
        };

        // 5. Add liquidity to TinyAMM (deepen core pair)
        if flash_needed > 0 && liq_amount > 0 {
            usdc_client.approve(&cca, &config.tinyamm, &liq_amount, &(e.ledger().sequence() + 1000));
            flash_client.approve(&cca, &config.tinyamm, &flash_needed, &(e.ledger().sequence() + 1000));
            Self::tinyamm_add_liquidity(
                &e, &config.tinyamm, &cca,
                flash_needed, liq_amount, 0, 0,
            );
        }

        // 6. Surplus = reward pool
        let surplus = flash_out - flash_needed;
        if surplus < min_reward_flash {
            return Err(FlashError::SlippageExceeded);
        }

        // 7. Pull bonus FLASH from treasury
        if bonus_flash > 0 {
            flash_client.transfer_from(
                &cca,
                &config.treasury,
                &cca,
                &bonus_flash,
            );
        }

        // 8. Store campaign
        let id = config.next_campaign_id + 1;
        config.next_campaign_id = id;
        set_config(&e, &config);

        let now = e.ledger().sequence();
        let campaign = Campaign {
            adapter_id,
            pool_address: pool_address.clone(),
            lp_token,
            creator: creator.clone(),
            end_ledger: now + duration,
            vesting_end: now + duration + config.defaults.vesting_period,
            target_lp,
            total_lp: 0,
            total_weight: 0,
            reward_flash: surplus,
            bonus_flash,
            user_count: 0,
        };
        save_campaign(&e, id, &campaign);

        let info = ActiveCampaignInfo {
            campaign_id: id,
            end_ledger: now + duration,
        };
        set_active_campaign(&e, &pool_address, &info);

        // 9. Emit event
        events::emit_campaign_created(&e, events::CampaignCreatedEvent {
            id,
            creator,
            adapter_id,
            pool_address,
            reward_flash: surplus,
            bonus_flash,
            duration,
            target_lp,
        });

        Ok(id)
    }

    // --- TinyAMM cross-contract helpers ---

    fn get_tinyamm_reserves(e: &Env, config: &CoreConfig) -> (i128, i128) {
        e.invoke_contract::<(i128, i128)>(
            &config.tinyamm,
            &soroban_sdk::Symbol::new(e, "get_reserves"),
            soroban_sdk::vec![e],
        )
    }

    fn tinyamm_swap(
        e: &Env,
        tinyamm: &Address,
        trader: &Address,
        sell_token: &Address,
        sell_amount: i128,
        min_buy: i128,
    ) -> i128 {
        use soroban_sdk::IntoVal;
        e.invoke_contract::<i128>(
            tinyamm,
            &soroban_sdk::Symbol::new(e, "swap_exact_in"),
            soroban_sdk::vec![
                e,
                trader.into_val(e),
                sell_token.into_val(e),
                sell_amount.into_val(e),
                min_buy.into_val(e),
            ],
        )
    }

    fn tinyamm_add_liquidity(
        e: &Env,
        tinyamm: &Address,
        to: &Address,
        amt0: i128,
        amt1: i128,
        min0: i128,
        min1: i128,
    ) -> (i128, i128, i128) {
        use soroban_sdk::IntoVal;
        e.invoke_contract::<(i128, i128, i128)>(
            tinyamm,
            &soroban_sdk::Symbol::new(e, "add_liquidity"),
            soroban_sdk::vec![
                e,
                to.into_val(e),
                amt0.into_val(e),
                amt1.into_val(e),
                min0.into_val(e),
                min1.into_val(e),
            ],
        )
    }
}

fn int_sqrt(x: u128) -> u128 {
    if x <= 1 {
        return x;
    }
    let (mut z, mut y) = (x, (x >> 1) + 1);
    while y < z {
        z = y;
        y = (x / y + y) >> 1;
    }
    z
}

#[cfg(test)]
mod test;
