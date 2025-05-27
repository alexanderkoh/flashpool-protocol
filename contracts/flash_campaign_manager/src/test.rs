#[allow(unused_imports)]
use super::*;

use crate::storage::upos_key;
use crate::tests::utils::{dump, fresh_env, fresh_env_native};
// use soroban_sdk::InvokeError;
use soroban_sdk::{ bytes, token, Address, Env, String, Vec };
//use soroban_sdk::{ testutils::{Address as _, Ledger, Logs, MockAuth, MockAuthInvoke}, token, Address, Env, IntoVal, String as SorobanString, TryFromVal, TryIntoVal, Val, Vec,};
extern crate std;

pub mod campaign_manager {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/flash_campaign_manager.wasm"
    );
    pub type FlashCampaignManagerClient<'a> = Client<'a>;
}
use campaign_manager::FlashCampaignManagerClient;
pub mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
use soroswap_pair::SoroswapPairClient;
pub mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_factory.wasm");
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
pub mod account_contract {
    use soroban_sdk::auth::Context;
    use smart_wallet_interface::types::Signatures;
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/account.wasm");
    pub type AccountClient<'a> = Client<'a>;
}
use account_contract::AccountClient;
// use soroswap_factory::SoroswapFactoryClient;

// For tests: Use 7 decimals for tokens!
pub const DECIMALS: u32 = 7;
pub const TOKEN_UNIT: i128 = 10i128.pow(DECIMALS);

// Helper: user joins a campaign
fn join_campaign_for_test<'a>(
    mgr: &FlashCampaignManagerClient<'a>,
    cid: &u32,
    amount: i128,
    user: &Address,
) {
    mgr.join_campaign(cid, &amount, user);
}

// Helper: creates a campaign and returns the campaign id
fn create_campaign_for_test<'a>(
    _e: &Env,
    mgr: &FlashCampaignManagerClient<'a>,
    pair: &Address,
    creator: &Address,
) -> Result<u32, FlashErr> {
    let fee_usdc    = 500     * TOKEN_UNIT;
    let unlock      = 10;
    let target_lp   = 100_000 * TOKEN_UNIT;
    let bonus_flash = 0;

    // 1) call the low-level stub; compiler infers its exact nested `Result<_,_>`
    let nested = mgr.try_create_campaign(
        &fee_usdc,
        pair,
        &unlock,
        &target_lp,
        &bonus_flash,
        creator,
    );
    // 2) panic if the host invocation itself failed
    let inner = nested.unwrap_or_else(|e| panic!("host invoke failed: {:?}", e));
    // 3) panic if the raw Val→u32 conversion failed
    let id = inner.unwrap_or_else(|e| panic!("conversion failed: {:?}", e));
    // 4) wrap the OK(u32) in your FlashErr‐Result
    Ok(id)
}


#[allow(dead_code)]
fn generate_volume_on_pair(
    e: &Env,
    pair: &Address,
    token_a: &token::Client,
    token_b: &token::Client,
    user: &Address,
) {
    token_a.transfer(user, pair, &(1_000 * TOKEN_UNIT));
    let pair_client = SoroswapPairClient::new(e, pair);
    let before = pair_client.get_reserves();
    pair_client.swap(&(1_000 * TOKEN_UNIT), &0, user);
    let after = pair_client.get_reserves();
    std::println!(
        "[VOLUME] swap token_a->b | before: {:?}, after: {:?}",
        before,
        after
    );

    token_b.transfer(user, pair, &(800 * TOKEN_UNIT));
    let before = pair_client.get_reserves();
    pair_client.swap(&0, &(800 * TOKEN_UNIT), user);
    let after = pair_client.get_reserves();
    std::println!(
        "[VOLUME] swap token_b->a | before: {:?}, after: {:?}",
        before,
        after
    );
}


#[derive(Debug, Clone)]
struct CampaignMathResult {
    swap_amount: i128,   // USDC swapped for FLASH
    flash_out: i128,     // FLASH received from swap
    flash_needed: i128,  // FLASH needed for liquidity add
    reward_flash: i128,  // FLASH sent to reward pool
}

fn replicate_campaign_math(
    fee_usdc: i128,
    surplus_bps: u32,
    reserve_usdc_before: i128,
    reserve_flash_before: i128,
) -> CampaignMathResult {
    // s_min = sqrt(resUsdc*(resUsdc+fee)) - resUsdc
    /*let s_min = int_sqrt(
        (reserve_usdc_before as u128) * ((reserve_usdc_before + fee_usdc) as u128),
    ) as i128
        - reserve_usdc_before;
*/
let s_min = 1;
    // swap_amount = s_min + (fee_usdc * surplus_bps / 10000)
    let swap_amount = (s_min + fee_usdc * surplus_bps as i128 / 10_000).min(fee_usdc);
    let usdc_liq = fee_usdc - swap_amount;

    // flash_out ~ (resFlash * swap_amount) / (resUsdc + swap_amount)
    let flash_out = (reserve_flash_before * swap_amount)
        / (reserve_usdc_before + swap_amount).max(1);

    // flash_needed ~ usdc_liq*(resFlash - flash_out)/(resUsdc + swap_amount)
    let flash_needed = if reserve_usdc_before + swap_amount > 0 {
        usdc_liq * (reserve_flash_before - flash_out)
            / (reserve_usdc_before + swap_amount)
    } else {
        0
    };

    let surplus = flash_out - flash_needed;
    let reward_flash = surplus.max(0);

    CampaignMathResult {
        swap_amount,
        flash_out,
        flash_needed,
        reward_flash,
    }
}
#[test]
fn test_simulate_circulating_supply_growth() {
    use std::string::String;
    use std::vec::Vec;
    use std::fmt::Write;

    // Constants
    let total_supply = 10_000_000 * TOKEN_UNIT;
    let initial_manager_balance = 0; // All tokens in LP at start
    let initial_lp_flash_balance = 10_000_000 * TOKEN_UNIT;
    let initial_lp_usdc_balance = 250 * TOKEN_UNIT;

    // Use bps as 1000..9000 for 10%..90%
    let bps_values: Vec<u32> = (100..=9000).step_by(250).collect();
    let num_campaigns = 10;

    let mut output = String::new();
    writeln!(
        &mut output,
        "bps | camp | deposit_usdc | swap_usdc | swap_flash | liq_usdc | liq_flash | reward_flash | lp_flash | lp_usdc | reward_pool | circ_supply | price(USD/FLASH)"
    ).ok();

    for &bps in &bps_values {
        let mut lp_flash_balance = initial_lp_flash_balance;
        let mut lp_usdc_balance = initial_lp_usdc_balance;
        let mut reward_pool_balance = 0i128;

        let mut deposit_usdc = 100 * TOKEN_UNIT; // Start at $100

        for campaign in 1..=num_campaigns {
            let math = replicate_campaign_math(
                deposit_usdc,
                bps,
                lp_usdc_balance,
                lp_flash_balance,
            );

            let swap_usdc = math.swap_amount;
            let liq_usdc = deposit_usdc - swap_usdc;
            let liq_flash = math.flash_needed;
            let reward_flash = math.reward_flash;

            lp_usdc_balance += swap_usdc + liq_usdc;
            lp_flash_balance = lp_flash_balance - math.flash_out + liq_flash;
            reward_pool_balance += reward_flash;

            let circulating_supply = lp_flash_balance + reward_pool_balance;
            let total = lp_flash_balance + reward_pool_balance; // manager_balance is always zero

            let price = if lp_flash_balance > 0 {
                (lp_usdc_balance as f64) / (lp_flash_balance as f64)
            } else {
                0.0
            };

            let mismatch = (total_supply - total).abs();

            let mut warn = String::new();
            if lp_flash_balance < 0 { warn.push_str("LP_FLASH_NEG "); }
            if lp_usdc_balance < 0 { warn.push_str("LP_USDC_NEG "); }
            if reward_pool_balance < 0 { warn.push_str("REWARD_NEG "); }
            if total != total_supply { warn.push_str("TOTAL_SUPPLY_MISMATCH "); }

            writeln!(
                &mut output,
                "{:4} | {:4} | {:12.2} | {:9.2} | {:10.2} | {:9.2} | {:9.2} | {:12.2} | {:9.2} | {:8.2} | {:11.2} | {:11.2} | {:9.6} | {:9.7} | {}",
                bps,
                campaign,
                (deposit_usdc as f64) * 1e-7,
                (swap_usdc as f64) * 1e-7,
                (math.flash_out as f64) * 1e-7,
                (liq_usdc as f64) * 1e-7,
                (liq_flash as f64) * 1e-7,
                (reward_flash as f64) * 1e-7,
                (lp_flash_balance as f64) * 1e-7,
                (lp_usdc_balance as f64) * 1e-7,
                (reward_pool_balance as f64) * 1e-7,
                (circulating_supply as f64) * 1e-7,
                price,
                (mismatch as f64) * 1e-7,
                warn
            ).ok();

            deposit_usdc = (deposit_usdc as f64 * 1.25) as i128;
        }
        writeln!(&mut output, "-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------").ok();
    }

    std::println!("\n{}", output);
}

#[test]
fn test_init_native() {
    let (
        _e,
        manager,
        god,
        _users,
        flash,
        usdc,
        eurc,
        some,
        factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    ) = fresh_env_native("my_test");
    std::println!(
        "[test_init_native] manager={:?} god={:?} flash={:?} usdc={:?} eurc={:?} some={:?} factory_addr={:?} flash_usdc_pair={:?} usdc_eurc_pair={:?} usdc_some_pair={:?}",
        manager, god, flash.address, usdc.address, eurc.address, some.address, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair
    );

    let (
        _e,
        mgr,
        god,
        _users,
        flash,
        usdc,
        eurc,
        some,
        factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    ) = fresh_env("my_test");
    std::println!(
        "[test_init_wasm] manager={:?} god={:?} flash={:?} usdc={:?} eurc={:?} some={:?} factory_addr={:?} flash_usdc_pair={:?} usdc_eurc_pair={:?} usdc_some_pair={:?}",
        mgr.address, god, flash.address, usdc.address, eurc.address, some.address, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair
    );
}

// The test cases:
#[test]
fn test_initialize_contract_and_seeding() {
    let (
        e,
        mgr,
        god,
        _users,
        flash,
        usdc,
        eurc,
        some,
        _factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    ) = fresh_env("test_initialize_contract_and_seeding");

    let manager_address = &mgr.address;
    let usdc_address = &usdc.address;
    let eurc_address = &eurc.address;
    let flash_address = &flash.address;
    let some_address = &some.address;
    std::println!(
        "[test_initialize_contract_and_seeding]\ninitialized an env:\n  Addresses:\n   manager:       {:?}\n   god address=   {:?}\n   flash address= {:?}\n   usdc address=  {:?}\n   eurc address=  {:?}\n   some address=  {:?}\n   factory=       {:?}\n   flash_usdc=    {:?}\n   usdc_eurc=     {:?}\n   usdc_some=     {:?}",
        manager_address, god, flash_address, usdc_address, eurc_address, some_address, _factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair
    );

    // Log contract's balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for contract:\n   flash=    {}\n   usdc=     {}\n   eurc=     {}\n    some= {}",
        flash.balance(&mgr.address) / TOKEN_UNIT,
        usdc.balance(&mgr.address) / TOKEN_UNIT,
        eurc.balance(&mgr.address) / TOKEN_UNIT,
        some.balance(&mgr.address) / TOKEN_UNIT
    );

    // Log god's balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for god:\n   flash=    {}\n   usdc=     {}\n   eurc=     {}\n    some= {}",
        flash.balance(&god) / TOKEN_UNIT,
        usdc.balance(&god) / TOKEN_UNIT,
        eurc.balance(&god) / TOKEN_UNIT,
        some.balance(&god) / TOKEN_UNIT
    );

    // Log pair balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for pairs:\n   flash_usdc: flash={} usdc={}\n   usdc_eurc: usdc={} eurc={}\n    usdc_some: usdc={} some={}",
        flash.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&usdc_eurc_pair) / TOKEN_UNIT,
        eurc.balance(&usdc_eurc_pair) / TOKEN_UNIT,
        usdc.balance(&usdc_some_pair) / TOKEN_UNIT,
        some.balance(&usdc_some_pair) / TOKEN_UNIT
    );

    //let flash_usdc_pair_client = soroswap_pair::Client::new(&e, &flash_usdc_pair);
    let flash_usdc_pair_client = SoroswapPairClient::new(&e, &flash_usdc_pair);

    let (rf, ru) = flash_usdc_pair_client.get_reserves();
    std::println!(
        "        flash_usdc_pair reserves: r0={:.7} r1={:.7}",
        (rf / TOKEN_UNIT) as f64,
        (ru / TOKEN_UNIT) as f64
    );
    assert!(rf > 0 && ru > 0);

    let usdc_eurc_pair_client = SoroswapPairClient::new(&e, &usdc_eurc_pair);
    let (r0, r1) = usdc_eurc_pair_client.get_reserves();
    std::println!(
        "        usdc_eurc_pair reserves: r0={:.7} r1={:.7}",
        (rf / TOKEN_UNIT) as f64,
        (ru / TOKEN_UNIT) as f64
    );

    assert!(r0 > 0 && r1 > 0);

    dump(&e, "setup_and_initialize");
}

#[test]
fn test_create_campaigns_native() {
    std::println!("[TEST] ---- test_create_campaign_no_bonus----");
    let (
        e,
        mgr,
        _god,
        users,
        _flash,
        usdc,
        eurc,
        some,
        _factory_addr,
        _flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    ) = fresh_env_native("test_create_campaigns_native");
    let alice = users.get(0).unwrap();
    std::println!(
        "[TEST] Balances: alice \n    usdc={:.7}\n    eurc={:.7}\n    some={:.7}",
        usdc.balance(&alice) / TOKEN_UNIT,
        eurc.balance(&alice) / TOKEN_UNIT,
        some.balance(&alice) / TOKEN_UNIT
    );
    let bob = users.get(0).unwrap();
    std::println!(
        "[TEST] Balances: bob\n    usdc={:.7}\n    eurc={:.7}\n    some={:.7}",
        usdc.balance(&bob) / TOKEN_UNIT,
        eurc.balance(&bob) / TOKEN_UNIT,
        some.balance(&bob) / TOKEN_UNIT
    );

    // Native call: Alice creates campaign for usdc_eurc_pair
    let cid1 = e.as_contract(&mgr, || {
        crate::FlashCampaignManager::create_campaign(
            e.clone(),
            500 * TOKEN_UNIT,
            usdc_eurc_pair.clone(),
            10,
            100_000 * TOKEN_UNIT,
            0,
            alice.clone(),
        )
    });
    match cid1 {
        Ok(id) => std::println!("[TEST_NATIVE] Campaign 1 created with cid={}", id),
        Err(e) => std::println!("[TEST_NATIVE] Campaign 1 creation failed: {:?}", e),
    }
    // Try to create a second campaign for the same pair (should fail)
    let cid2 = e.as_contract(&mgr, || {
        crate::FlashCampaignManager::create_campaign(
            e.clone(),
            500 * TOKEN_UNIT,
            usdc_eurc_pair.clone(),
            10,
            100_000 * TOKEN_UNIT,
            0,
            bob.clone(),
        )
    });
    assert_eq!(
        cid2,
        Err(FlashErr::CampaignActiveForPair),
        "Second campaign creation for the same pair should fail"
    );

    // Create campaign for a different pair (usdc_some_pair) - should succeed
    let cid3 = e.as_contract(&mgr, || {
        crate::FlashCampaignManager::create_campaign(
            e.clone(),
            500 * TOKEN_UNIT,
            usdc_some_pair.clone(),
            10,
            100_000 * TOKEN_UNIT,
            0,
            bob.clone(),
        )
    });
    match cid3 {
        Ok(id) => std::println!("[TEST_NATIVE] Campaign 1 created with cid={}", id),
        Err(e) => std::println!("[TEST_NATIVE] Campaign 1 creation failed: {:?}", e),
    }
    dump(&e, "create_campaign_single_user");
}
// Test: create campaign with single user, no bonus
#[test]
fn test_create_and_join_campaign() {
    use soroban_sdk::BytesN;
    let (
        e,
        mgr,
        _god,
        users,
        _flash,
        _usdc,
        _eurc,
        _some,
        _factory_addr,
        _flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    ) = fresh_env("test_create_single_user_campaign_no_bonus");
    let alice = users.get(0).unwrap();

    // Create campaign for usdc_eurc_pair
    let cid1 = create_campaign_for_test(&e, &mgr, &usdc_eurc_pair, &alice);
    assert!(cid1.is_ok(), "First campaign creation should succeed");
    let campaign_id = cid1.unwrap();
    std::println!("[TEST_NATIVE] Campaign 1 created with cid={}", campaign_id);

    let account_wasm = account_contract::WASM;
    let account_wasm_bytes = Bytes::from_slice(&e, account_wasm);
    let account_wasm_hash: BytesN<32> = e.crypto().sha256(&account_wasm_bytes).into(); 

    // Join campaign as single user via smart account
    let join_amount = 2000 * TOKEN_UNIT;
    let join_result = mgr.try_join_campaign_with_account(&campaign_id, &join_amount, &alice);
    assert!(join_result.is_ok(), "User should be able to join the campaign via smart account");

    // Check user position exists for smart account
    //let salt = alice.clone().into_val(&e);
    let user_bytes = &alice.clone().to_xdr(&e);
    let user_salt: BytesN<32> = BytesN::from_xdr(&e, &user_bytes).unwrap();
    let account_address = e.deployer().with_current_contract(user_salt).deployed_address();
    std::println!(
        "[TEST_NATIVE] User smart account address: {:?}",
        account_address
    );
    let key = upos_key(&e, campaign_id, &account_address);
    let upos: Option<UserPos> = e.as_contract(&mgr.address, || e.storage().instance().get(&key));
    std::println!("User position after join (smart account): {:?}", upos);
    assert!(upos.is_some());
}

#[test]
fn test_create_and_join_campaign_native() {
    use soroban_sdk::BytesN;
    std::println!("[TEST] ---- test_create_and_join_campaign_native ----");
    let (
        e,
        mgr,
        _god,
        users,
        _flash,
        _usdc,
        _eurc,
        _some,
        _factory_addr,
        _flash_usdc_pair,
        usdc_eurc_pair,
        _usdc_some_pair,
    ) = fresh_env_native("test_create_and_join_campaign_native");
    let alice = users.get(0).unwrap();

    // Create campaign for usdc_eurc_pair
    let campaign_id = e.as_contract(&mgr, || {
        crate::FlashCampaignManager::create_campaign(
            e.clone(),
            500 * TOKEN_UNIT,
            usdc_eurc_pair.clone(),
            10,
            100_000 * TOKEN_UNIT,
            0,
            alice.clone(),
        )
    }).expect("Campaign creation should succeed");
    std::println!("[TEST_NATIVE] Campaign created with cid={}", campaign_id);

    // Prepare smart account salt and address
    let user_bytes = &alice.clone().to_xdr(&e);
    std::println!("[join_native_test] User bytes: {:?}", user_bytes);
    //let user_salt: BytesN<32> = BytesN::from_xdr(&e, &user_bytes).unwrap();
    let user_salt: BytesN<32> = e.crypto().sha256(&user_bytes).into();
    std::println!("[join_native_test] User salt: {:?}", user_salt);
    let account_address = e.as_contract(&mgr, || {e.deployer().with_current_contract(user_salt.clone()).deployed_address()});
    std::println!(
        "[TEST_NATIVE] User smart account address: {:?}",
        account_address
    );

    // Deploy the account contract if not already deployed
    let account_wasm = account_contract::WASM;
    let account_wasm_bytes = soroban_sdk::Bytes::from_slice(&e, account_wasm);
    let account_wasm_hash: BytesN<32> = e.crypto().sha256(&account_wasm_bytes).into();

    // we need to first upload the wasm to the ledger. using the deployer.

    let uploaded_wasm_hash: BytesN<32> = e.as_contract(&mgr, || {
    e.deployer().upload_contract_wasm(account_wasm_bytes.clone())
});
    
    std::println!(
        "[TEST_NATIVE] Uploaded account contract wasm hash: {:?}",
        uploaded_wasm_hash
    );


    // Deploy with empty constructor args (or adjust if your account contract requires args)
    let deployed_account_address =  e.as_contract(&mgr, || {e.deployer().with_current_contract(user_salt.clone()).deploy_v2(account_wasm_hash.clone(), ())});
    std::println!(
        "[TEST_NATIVE] Deployed account contract at address: {:?}",
        deployed_account_address
    );

    // Optionally: Add the manager and user as authorized signers if required by your account contract
    //let account_client = account_contract::AccountClient::new(&e, &account_address);
    // Example: account_client.add_signer(&mgr.address); account_client.add_signer(&alice);
    // Uncomment and adjust the above if your contract requires explicit authorization setup

    // Join campaign as single user via smart account, natively (so logs are visible)
    let join_amount = 2000 * TOKEN_UNIT;
    let join_result = e.as_contract(&mgr, || {
        crate::FlashCampaignManager::join_campaign_with_account(
            e.clone(),
            campaign_id,
            join_amount,
            alice.clone(),
        )
    });
    assert!(join_result.is_ok(), "User should be able to join the campaign via smart account");
    std::println!("[TEST_NATIVE] User joined campaign via smart account: {:?}", join_result);

    // Optionally: Check user position exists for smart account
    // ...
}
/*
#[test]
fn test_create_and_join_campaign_two_users() {
    let (e, mgr, _god, users, _flash, usdc, eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
        fresh_env();
    let alice = users.get(0).unwrap();
    let bob = users.get(1).unwrap();

    std::println!(
        "[TEST] Balances: alice usdc={}, bob usdc={}",
        usdc.balance(&alice) / TOKEN_UNIT,
        usdc.balance(&bob) / TOKEN_UNIT
    );
    let cid = mgr.create_campaign(&(1000 * TOKEN_UNIT), &usdc_eurc_pair, &10, &0, &0, &alice);
    mgr.join_campaign(&cid, &(2000 * TOKEN_UNIT), &bob);

    let key: Val = (PREFIX_UPOS, cid, bob.clone()).into_val(&e);
    assert!(e.storage().instance().has(&key));
    dump(&e, "join_campaign_two_users");
}

#[test]
fn test_compound_with_volume() {
    let (e, mgr, _god, users, flash, usdc, eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
        fresh_env();

    let alice = users.get(0).unwrap();
    let bob = users.get(1).unwrap();

    std::println!(
        "[TEST] Balances: alice usdc={}, eurc={}",
        usdc.balance(&alice) / TOKEN_UNIT,
        eurc.balance(&alice) / TOKEN_UNIT
    );
    let cid = mgr.create_campaign(&(1000 * TOKEN_UNIT), &usdc_eurc_pair, &10, &0, &0, &alice);
    mgr.join_campaign(&cid, &(2000 * TOKEN_UNIT), &bob);

    generate_volume_on_pair(&e, &usdc_eurc_pair, &usdc, &eurc, &alice);

    mgr.compound(&cid);
    dump(&e, "compound_with_volume");
}

#[test]
fn test_claim_after_unlock() {
    let (e, mgr, _god, users, flash, usdc, eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
        fresh_env();

    let alice = users.get(0).unwrap();
    let bob = users.get(1).unwrap();

    let cid = mgr.create_campaign(&(1000 * TOKEN_UNIT), &usdc_eurc_pair, &5, &0, &0, &alice);
    mgr.join_campaign(&cid, &(2000 * TOKEN_UNIT), &bob);

    generate_volume_on_pair(&e, &usdc_eurc_pair, &usdc, &eurc, &alice);
    mgr.compound(&cid);

    e.ledger().with_mut(|li| li.sequence_number += 10);

    mgr.claim(&cid, &bob);

    let key: Val = (PREFIX_UPOS, cid, bob.clone()).into_val(&e);
    assert!(!e.storage().instance().has(&key));

    dump(&e, "claim_after_unlock");
}
*/
