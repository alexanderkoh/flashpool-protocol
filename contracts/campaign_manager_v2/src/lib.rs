#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, IntoVal, Symbol};

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
        // Transfer USDC to TinyAMM first, then call swap_received (no nested auth needed)
        usdc_client.transfer(&cca, &config.tinyamm, &swap_amount);
        let flash_out = Self::tinyamm_swap_received(
            &e, &config.tinyamm, &config.usdc, swap_amount, 0, &cca,
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
        // Use `deposit` — tokens pre-transferred, no auth needed
        if flash_needed > 0 && liq_amount > 0 {
            flash_client.transfer(&cca, &config.tinyamm, &flash_needed);
            usdc_client.transfer(&cca, &config.tinyamm, &liq_amount);
            Self::tinyamm_deposit(
                &e, &config.tinyamm, &cca,
                flash_needed, liq_amount,
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

        // 2. Swap half and deposit both sides into target pool
        let lp_received = Self::swap_and_deposit(
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

    pub fn claim(e: Env, id: u32, user: Address) -> Result<(), FlashError> {
        if !has_config(&e) {
            return Err(FlashError::NotInitialized);
        }
        user.require_auth();
        bump_instance(&e);

        let config = get_config(&e);
        let campaign = load_campaign(&e, id).ok_or(FlashError::CampaignNotFound)?;

        if e.ledger().sequence() < campaign.end_ledger {
            return Err(FlashError::CampaignNotEnded);
        }

        let mut pos = load_user_pos(&e, id, &user).ok_or(FlashError::PositionNotFound)?;

        if pos.weight <= 0 {
            return Err(FlashError::NothingToClaim);
        }

        // Calculate total reward on first claim
        if pos.total_reward == 0 {
            let base = campaign
                .reward_flash
                .checked_mul(pos.weight)
                .unwrap_or(0)
                .checked_div(campaign.total_weight)
                .unwrap_or(0);

            let bonus = if campaign.total_lp >= campaign.target_lp {
                campaign
                    .bonus_flash
                    .checked_mul(pos.weight)
                    .unwrap_or(0)
                    .checked_div(campaign.total_weight)
                    .unwrap_or(0)
            } else {
                0
            };

            pos.total_reward = base + bonus;
        }

        // Calculate vested amount
        let elapsed = e.ledger().sequence().saturating_sub(campaign.end_ledger);
        let vesting_period = campaign.vesting_end.saturating_sub(campaign.end_ledger);
        let vested_pct = rewards::compute_vested_pct(elapsed, vesting_period);
        let vested = pos
            .total_reward
            .checked_mul(vested_pct)
            .unwrap_or(0)
            .checked_div(10_000)
            .unwrap_or(0);
        let claimable = vested - pos.claimed;

        if claimable <= 0 {
            return Err(FlashError::NothingToClaim);
        }

        // Transfer FLASH rewards (manager holds these)
        let cca = e.current_contract_address();
        let flash_client = soroban_sdk::token::Client::new(&e, &config.flash);
        flash_client.transfer(&cca, &user, &claimable);

        pos.claimed += claimable;
        let fully_vested = pos.claimed >= pos.total_reward;

        if fully_vested {
            // Return LP tokens via pool's lp_transfer
            e.invoke_contract::<()>(
                &campaign.pool_address,
                &Symbol::new(&e, "lp_transfer"),
                soroban_sdk::vec![
                    &e,
                    cca.clone().into_val(&e),
                    user.clone().into_val(&e),
                    pos.lp.into_val(&e),
                ],
            );

            // Mark as claimed (set weight to 0) but keep record for future rolling campaigns
            pos.weight = 0;
            save_user_pos(&e, id, &user, &pos);
        } else {
            save_user_pos(&e, id, &user, &pos);
        }

        // Emit event
        events::emit_reward_claimed(&e, events::RewardClaimedEvent {
            campaign_id: id,
            user,
            flash_amount: claimable,
            fully_vested,
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

    fn tinyamm_swap_received(
        e: &Env,
        tinyamm: &Address,
        sell_token: &Address,
        sell_amount: i128,
        min_buy: i128,
        recipient: &Address,
    ) -> i128 {
        e.invoke_contract::<i128>(
            tinyamm,
            &Symbol::new(e, "swap_received"),
            soroban_sdk::vec![
                e,
                sell_token.into_val(e),
                sell_amount.into_val(e),
                min_buy.into_val(e),
                recipient.into_val(e),
            ],
        )
    }

    fn tinyamm_deposit(
        e: &Env,
        tinyamm: &Address,
        to: &Address,
        amt0: i128,
        amt1: i128,
    ) -> (i128, i128, i128) {
        e.invoke_contract::<(i128, i128, i128)>(
            tinyamm,
            &Symbol::new(e, "deposit"),
            soroban_sdk::vec![
                e,
                to.into_val(e),
                amt0.into_val(e),
                amt1.into_val(e),
            ],
        )
    }

    // --- Pool swap + deposit helper ---
    // adapter_id 0 = direct TinyAMM pool (swap_received + deposit)
    // adapter_id 1+ = Hoops adapter (future)

    fn swap_and_deposit(
        e: &Env,
        config: &CoreConfig,
        campaign: &Campaign,
        sell_token: &Address,
        total_amount: i128,
    ) -> Result<i128, FlashError> {
        let cca = e.current_contract_address();
        let token_client = soroban_sdk::token::Client::new(e, sell_token);
        let pool = &campaign.pool_address;

        let half = total_amount / 2;
        let remaining = total_amount - half;

        if campaign.adapter_id == 0 {
            // Direct TinyAMM pool: use swap_received + deposit
            // 1. Transfer half to pool for swap
            token_client.transfer(&cca, pool, &half);

            // 2. Swap via pool's swap_received (no auth needed)
            let buy_amount: i128 = e.invoke_contract(
                pool,
                &Symbol::new(e, "swap_received"),
                soroban_sdk::vec![
                    e,
                    sell_token.into_val(e),
                    half.into_val(e),
                    0i128.into_val(e),
                    cca.clone().into_val(e),
                ],
            );

            // 3. Get pool tokens to figure out which is which
            let (pool_token0, pool_token1): (Address, Address) = e.invoke_contract(
                pool,
                &Symbol::new(e, "get_tokens"),
                soroban_sdk::vec![e],
            );
            let buy_token = if *sell_token == pool_token0 { pool_token1 } else { pool_token0 };

            // 4. Transfer both tokens to pool for deposit
            token_client.transfer(&cca, pool, &remaining);
            soroban_sdk::token::Client::new(e, &buy_token).transfer(&cca, pool, &buy_amount);

            // 5. Track LP balance before deposit
            let lp_before = Self::get_lp_balance(e, pool, &cca);

            // 6. Deposit (no auth needed)
            e.invoke_contract::<(i128, i128, i128)>(
                pool,
                &Symbol::new(e, "deposit"),
                soroban_sdk::vec![
                    e,
                    cca.clone().into_val(e),
                    remaining.into_val(e),
                    buy_amount.into_val(e),
                ],
            );

            let lp_after = Self::get_lp_balance(e, pool, &cca);
            Ok(lp_after - lp_before)
        } else {
            // Hoops adapter path (future — requires deployed adapters)
            Err(FlashError::AdapterCallFailed)
        }
    }

    fn get_lp_balance(e: &Env, pool: &Address, owner: &Address) -> i128 {
        e.invoke_contract::<i128>(
            pool,
            &Symbol::new(e, "lp_balance"),
            soroban_sdk::vec![e, owner.into_val(e)],
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
