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

    pub fn join_campaign(
        e: Env,
        id: u32,
        amount: i128,
        token: Address,
        user: Address,
    ) -> Result<(), FlashError> {
        if !has_config(&e) {
            return Err(FlashError::NotInitialized);
        }
        user.require_auth();
        bump_instance(&e);

        if amount <= 0 {
            return Err(FlashError::InsufficientAmount);
        }

        let config = get_config(&e);
        let mut campaign = load_campaign(&e, id).ok_or(FlashError::CampaignNotFound)?;

        if e.ledger().sequence() >= campaign.end_ledger {
            return Err(FlashError::CampaignAlreadyEnded);
        }

        let cca = e.current_contract_address();
        let token_client = soroban_sdk::token::Client::new(&e, &token);

        // 1. Transfer token from user to manager
        token_client.transfer(&user, &cca, &amount);

        // 2. Swap half via Hoops adapter and deposit both sides
        let lp_received = Self::adapter_swap_and_deposit(
            &e, &config, &campaign, &token, amount,
        )?;

        if lp_received <= 0 {
            return Err(FlashError::InsufficientAmount);
        }

        // 3. Assign rank
        campaign.user_count += 1;
        let rank = campaign.user_count;

        // 4. Compute weight
        let rank_weight = rewards::compute_rank_weight(rank, config.defaults.gamma);
        let contrib_weight = rewards::compute_contribution_weight(lp_received, campaign.target_lp);
        let new_weight = rewards::compute_score(rank_weight, contrib_weight);

        // 5. Handle re-join: accumulate
        let existing = load_user_pos(&e, id, &user);
        let (final_lp, final_weight, final_rank) = if let Some(old) = existing {
            let combined_lp = old.lp + lp_received;
            let combined_contrib = rewards::compute_contribution_weight(combined_lp, campaign.target_lp);
            let combined_weight = rewards::compute_score(
                rewards::compute_rank_weight(old.rank, config.defaults.gamma),
                combined_contrib,
            );
            // Adjust campaign total weight: remove old, add new
            campaign.total_weight -= old.weight;
            (combined_lp, combined_weight, old.rank)
        } else {
            (lp_received, new_weight, rank)
        };

        // 6. Update campaign totals
        campaign.total_lp += lp_received;
        campaign.total_weight += final_weight;
        save_campaign(&e, id, &campaign);

        // 7. Store user position
        let pos = UserPos {
            lp: final_lp,
            weight: final_weight,
            rank: final_rank,
            joined_ledger: e.ledger().sequence(),
            total_reward: 0,
            claimed: 0,
        };
        save_user_pos(&e, id, &user, &pos);

        // 8. Emit event
        events::emit_user_joined(&e, events::UserJoinedEvent {
            campaign_id: id,
            user,
            lp: final_lp,
            rank: final_rank,
            weight: final_weight,
        });

        Ok(())
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

    // --- Hoops adapter cross-contract helper ---

    fn adapter_swap_and_deposit(
        e: &Env,
        config: &CoreConfig,
        campaign: &Campaign,
        sell_token: &Address,
        total_amount: i128,
    ) -> Result<i128, FlashError> {
        use soroban_sdk::IntoVal;

        // Get adapter address from Hoops router
        let adapter_address: Address = e.invoke_contract(
            &config.hoops_router,
            &soroban_sdk::Symbol::new(e, "get_adapter"),
            soroban_sdk::vec![e, campaign.adapter_id.into_val(e)],
        );

        let cca = e.current_contract_address();
        let token_client = soroban_sdk::token::Client::new(e, sell_token);

        let half = total_amount / 2;
        let remaining = total_amount - half;

        // Transfer half to adapter for swap
        token_client.transfer(&cca, &adapter_address, &half);

        // Call adapter swap_in_pool
        let swap_out: i128 = e.invoke_contract(
            &adapter_address,
            &soroban_sdk::Symbol::new(e, "swap_in_pool"),
            soroban_sdk::vec![
                e,
                campaign.pool_address.clone().into_val(e),
                sell_token.into_val(e),
                half.into_val(e),
                0i128.into_val(e),        // min_out
                cca.clone().into_val(e),  // to
            ],
        );

        // Transfer remaining sell_token to adapter for liquidity
        token_client.transfer(&cca, &adapter_address, &remaining);

        // Track LP balance before deposit
        let lp_client = soroban_sdk::token::Client::new(e, &campaign.lp_token);
        let lp_before = lp_client.balance(&cca);

        // Call adapter add_liq_in_pool
        e.invoke_contract::<()>(
            &adapter_address,
            &soroban_sdk::Symbol::new(e, "add_liq_pool"),
            soroban_sdk::vec![
                e,
                campaign.pool_address.clone().into_val(e),
                remaining.into_val(e),
                swap_out.into_val(e),
                cca.clone().into_val(e),  // LP tokens sent here
            ],
        );

        let lp_after = lp_client.balance(&cca);
        let lp_received = lp_after - lp_before;

        Ok(lp_received)
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
