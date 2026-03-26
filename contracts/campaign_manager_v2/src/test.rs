#![cfg(test)]

use soroban_sdk::Env;
use crate::rewards;

#[test]
fn test_rank_weight_first() {
    let env = Env::default();
    let w = rewards::compute_rank_weight(1, 2);
    assert_eq!(w, 100_000_000);
}

#[test]
fn test_rank_weight_tenth() {
    let env = Env::default();
    let w = rewards::compute_rank_weight(10, 2);
    assert_eq!(w, 1_000_000);
}

#[test]
fn test_rank_weight_first_is_100x_tenth() {
    let env = Env::default();
    let w1 = rewards::compute_rank_weight(1, 2);
    let w10 = rewards::compute_rank_weight(10, 2);
    assert_eq!(w1 / w10, 100);
}

#[test]
fn test_rank_weight_zero_returns_zero() {
    let env = Env::default();
    assert_eq!(rewards::compute_rank_weight(0, 2), 0);
}

#[test]
fn test_contrib_weight_at_target() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(1000, 1000);
    assert_eq!(w, 10_000);
}

#[test]
fn test_contrib_weight_half_target() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(500, 1000);
    assert_eq!(w, 5_000);
}

#[test]
fn test_contrib_weight_over_target_capped() {
    let env = Env::default();
    let w = rewards::compute_contribution_weight(2000, 1000);
    assert_eq!(w, 10_000);
}

#[test]
fn test_contrib_weight_zero_deposit() {
    let env = Env::default();
    assert_eq!(rewards::compute_contribution_weight(0, 1000), 0);
}

#[test]
fn test_contrib_weight_zero_target() {
    let env = Env::default();
    assert_eq!(rewards::compute_contribution_weight(1000, 0), 0);
}

#[test]
fn test_compute_score() {
    let env = Env::default();
    let rank_w = rewards::compute_rank_weight(1, 2);
    let contrib_w = rewards::compute_contribution_weight(500, 1000);
    let score = rewards::compute_score(rank_w, contrib_w);
    assert_eq!(score, 50_000_000);
}

#[test]
fn test_compute_vested_pct_zero_elapsed() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(0, 100_000);
    assert_eq!(pct, 0);
}

#[test]
fn test_compute_vested_pct_half() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(50_000, 100_000);
    assert_eq!(pct, 5_000);
}

#[test]
fn test_compute_vested_pct_full() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(100_000, 100_000);
    assert_eq!(pct, 10_000);
}

#[test]
fn test_compute_vested_pct_over() {
    let env = Env::default();
    let pct = rewards::compute_vested_pct(200_000, 100_000);
    assert_eq!(pct, 10_000);
}

// --- Initialize / set_defaults tests ---

use soroban_sdk::{testutils::Address as _, Address};
use crate::types::*;
use crate::CampaignManagerClient;

fn default_defaults() -> ProtocolDefaults {
    ProtocolDefaults {
        gamma: 2,
        vesting_period: 100_000,
        surplus_bps: 500,
        ttl_thresh: 172_800,
        ttl_bump: 241_920,
    }
}

fn setup() -> (Env, Address, Address, Address, Address, Address, Address, CampaignManagerClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let flash = env.register_stellar_asset_contract_v2(admin.clone()).address().clone();
    let usdc = env.register_stellar_asset_contract_v2(admin.clone()).address().clone();
    let treasury = Address::generate(&env);
    let tinyamm = Address::generate(&env);
    let hoops_router = Address::generate(&env);

    let contract_id = env.register(crate::CampaignManager, ());
    let client = CampaignManagerClient::new(&env, &contract_id);

    (env, admin, flash, usdc, treasury, tinyamm, hoops_router, client)
}

#[test]
fn test_initialize() {
    let (_env, admin, flash, usdc, treasury, tinyamm, hoops_router, client) = setup();
    client.initialize(&admin, &flash, &usdc, &treasury, &tinyamm, &hoops_router);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize() {
    let (_env, admin, flash, usdc, treasury, tinyamm, hoops_router, client) = setup();
    client.initialize(&admin, &flash, &usdc, &treasury, &tinyamm, &hoops_router);
    client.initialize(&admin, &flash, &usdc, &treasury, &tinyamm, &hoops_router);
}

#[test]
fn test_set_defaults() {
    let (_env, admin, flash, usdc, treasury, tinyamm, hoops_router, client) = setup();
    client.initialize(&admin, &flash, &usdc, &treasury, &tinyamm, &hoops_router);

    let new_defaults = ProtocolDefaults {
        gamma: 3,
        vesting_period: 200_000,
        surplus_bps: 1000,
        ttl_thresh: 172_800,
        ttl_bump: 241_920,
    };
    client.set_defaults(&admin, &new_defaults);
}

#[test]
fn test_create_campaign_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(crate::CampaignManager, ());
    let client = CampaignManagerClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool = Address::generate(&env);
    let lp_token = Address::generate(&env);

    let result = client.try_create_campaign(
        &100_000i128, &3i128, &pool, &lp_token,
        &100_000u32, &50_000i128, &0i128, &0i128, &creator,
    );
    assert!(result.is_err());
}

#[test]
fn test_claim_vesting_math() {
    let _env = Env::default();

    let total_reward: i128 = 100_000;
    let vesting_period: u32 = 100_000;

    // At 0% elapsed
    let pct = crate::rewards::compute_vested_pct(0, vesting_period);
    let claimable = total_reward * pct / 10_000 - 0;
    assert_eq!(claimable, 0);

    // At 50% elapsed
    let pct = crate::rewards::compute_vested_pct(50_000, vesting_period);
    let claimable = total_reward * pct / 10_000 - 0;
    assert_eq!(claimable, 50_000);

    // At 100% elapsed
    let pct = crate::rewards::compute_vested_pct(100_000, vesting_period);
    let claimable = total_reward * pct / 10_000 - 0;
    assert_eq!(claimable, 100_000);

    // After claiming 50_000, claim remaining at 100%
    let pct = crate::rewards::compute_vested_pct(100_000, vesting_period);
    let claimable = total_reward * pct / 10_000 - 50_000;
    assert_eq!(claimable, 50_000);
}

#[test]
fn test_claim_before_campaign_ends() {
    let (_env, admin, flash, usdc, treasury, tinyamm, hoops_router, client) = setup();
    client.initialize(&admin, &flash, &usdc, &treasury, &tinyamm, &hoops_router);

    let user = Address::generate(&_env);
    let result = client.try_claim(&1u32, &user);
    assert!(result.is_err());
}

#[test]
fn test_reward_distribution_proportional() {
    let _env = Env::default();

    let r1 = crate::rewards::compute_rank_weight(1, 2);
    let c1 = crate::rewards::compute_contribution_weight(1000, 1000);
    let w1 = crate::rewards::compute_score(r1, c1);

    let r2 = crate::rewards::compute_rank_weight(2, 2);
    let c2 = crate::rewards::compute_contribution_weight(1000, 1000);
    let w2 = crate::rewards::compute_score(r2, c2);

    let r3 = crate::rewards::compute_rank_weight(5, 2);
    let c3 = crate::rewards::compute_contribution_weight(500, 1000);
    let w3 = crate::rewards::compute_score(r3, c3);

    let total_weight = w1 + w2 + w3;
    let reward_pool: i128 = 100_000;

    let reward1 = reward_pool * w1 / total_weight;
    let reward2 = reward_pool * w2 / total_weight;
    let reward3 = reward_pool * w3 / total_weight;

    assert!(reward1 > reward2);
    assert!(reward2 > reward3);
    assert!(reward1 > 70_000);
    assert!(reward1 + reward2 + reward3 <= reward_pool);
    assert!(reward1 + reward2 + reward3 > reward_pool - 10);
}

#[test]
fn test_vesting_schedule_multi_claim() {
    let _env = Env::default();
    let total_reward: i128 = 120_000;
    let vesting_period: u32 = 120_000;

    let pct_25 = crate::rewards::compute_vested_pct(30_000, vesting_period);
    let claim_1 = total_reward * pct_25 / 10_000;
    assert_eq!(claim_1, 30_000);

    let pct_75 = crate::rewards::compute_vested_pct(90_000, vesting_period);
    let vested_75 = total_reward * pct_75 / 10_000;
    let claim_2 = vested_75 - claim_1;
    assert_eq!(claim_2, 60_000);

    let pct_100 = crate::rewards::compute_vested_pct(120_000, vesting_period);
    let vested_100 = total_reward * pct_100 / 10_000;
    let claim_3 = vested_100 - (claim_1 + claim_2);
    assert_eq!(claim_3, 30_000);

    assert_eq!(claim_1 + claim_2 + claim_3, total_reward);
}
