/* what should the tests do?

Each test line in the plan below gets a comment checkbox; update as they pass/fail.
    1. Setup:
        - [x] create the admin account (god)
        - [x] create ten user accounts (alice, bob, charlie, dave, eve, frank, grace, heidi, ivan, judy)
        - [x] create three tokens (flash, usdc, and eurc)
        - [x] mint 10_000_000 flash, 1,000,000 usdc, and 1,250,000 eurc.
        - [x] distribute all the flash to god, and give a proportional amount of usdc and eurc to each user, with the user with the least receiving 100% less than the user with the most, and at a ratio of 1:1.25 usdc to eurc.  the maximum amount one user can receive is 5000 us and 6250 eurc.
    - deploy a soroswap factory, and create two pairs, flash/usdc and usdc/eurc

2. initialize the flash campaign manager contract.
    - ensure that it transfers the flash token to the contract.
    - ensure it transfers the correct amount of usdc to the contract.
    - ensure it deposits it into the usdc/flash pair.
3. create a campaign
    - create a campaign with the following parameters:
    - 500 usdc.
    - target pair is to be the usdc/eurc pair.
    - target amount is 75,000 usdc worth of liquidity added to the pair.
    - make sure the campaign correctly purchases flash with the usdc.
    - ensure the campaign creation then deeposits the remaining usdc along with the purchased flash into the usdc/flash pair.
    - ensure the campaign allocates the correct amount of flash to the campaign.
    - the amount of flash allocated to the campaign is the amount of flash that would be needed to sell to the usdc/flash pair to cause the price to drop to below what the price was before the campaign was created.
    - well the actual amount is 5% more than that required amount.
    - setup the campaigns so that they are able to end right after all the users join for the tests, so just increment the ledger by the number of blocks required.
4. join the campaign.
    - complete a camapgin with only one user.
    - complete a campaign with all users, but do not meet the target amount.
    - complete a campaign with all users, and meet the target amount.
5. compond a campaign.
    - to compond a camapgin there needs to have been volume to create fees against the target pair.
    - so after the users have joined the campaign, we need to create trades against the target pair, and generate fees.
    - then we need to make sure that calling compond on the campaign claims those fees, and then purchases more flash with the fees.
    - this flash should then be allocated to the users in the campaign and should be able to be claimed.
6. claim the rewards.
    - make sure that the users can claim their rewards after the campaign has ended.
    - make sure that the users can claim their rewards after the campaign has been compounded.
    - make sure that the users can claim their rewards after the campaign has been compounded and the campaign has ended.
    - make sure that users can not claim their rewardsd or withdraw their liquidity before the campaign has ended

    */
#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, Logs},
    token, Address, Env, IntoVal, Val, Vec,
};
extern crate std;

mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
}
mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_factory.wasm");
}

// For tests: Use 7 decimals for tokens!
const DECIMALS: u32 = 7;
const TOKEN_UNIT: i128 = 10i128.pow(DECIMALS);

fn dump(e: &Env, label: &str) {
    std::println!("── logs after {label} ─────────────────────────────");
    for l in e.logs().all() {
        std::println!("{l}");
    }
    let res = e.cost_estimate().resources();
    std::println!("Budget/Resources: instructions={}, mem_bytes={}, read_entries={}, write_entries={}, read_bytes={}, write_bytes={}",
        res.instructions, res.mem_bytes, res.read_entries, res.write_entries, res.read_bytes, res.write_bytes
    );
    std::println!("──────────────────────────────────────────────────\n");
}

fn fresh_env<'a>() -> (
    Env,
    FlashCampaignManagerClient<'a>,
    Address,           // god
    Vec<Address>,      // users
    token::Client<'a>, // flash
    token::Client<'a>, // usdc
    token::Client<'a>, // eurc
    Address,           // factory
    Address,           // flash_usdc_pair
    Address,           // usdc_eurc_pair
) {
    let e = Env::default();
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    let god = Address::generate(&e);
    let mut users = Vec::new(&e);
    for _ in 0..10 {
        users.push_back(Address::generate(&e));
    }

    let (flash, flash_admin) = create_token(&e, &god);
    let (usdc, usdc_admin) = create_token(&e, &god);
    let (eurc, eurc_admin) = create_token(&e, &god);

    // Mint with full decimals
    flash_admin.mint(&god, &(10_000_000 * TOKEN_UNIT));
    usdc_admin.mint(&god, &(1_250_000 * TOKEN_UNIT));
    eurc_admin.mint(&god, &(1_000_000 * TOKEN_UNIT));

    // Give each user proportional USDC/EURC (simplified for now)
    for user in users.iter() {
        usdc_admin.mint(&user, &(12_500 * TOKEN_UNIT));
        eurc_admin.mint(&user, &(10_000 * TOKEN_UNIT));
        std::println!("[minting tokens] user={:?} usdc={:?} eurc={:?}", user, usdc.balance(&user) / TOKEN_UNIT, eurc.balance(&user) / TOKEN_UNIT);
    }

    // Deploy factory and pairs
    let pair_wasm_hash = e.deployer().upload_contract_wasm(soroswap_pair::WASM);
    let factory_addr = e.register(soroswap_factory::WASM, ());
    let factory = soroswap_factory::Client::new(&e, &factory_addr);
    factory.initialize(&god, &pair_wasm_hash);

    let flash_usdc_pair = factory.create_pair(&flash.address, &usdc.address);
    let usdc_eurc_pair = factory.create_pair(&usdc.address, &eurc.address);

    // -- Begin VERBOSE LOGGING for flash/usdc pair liquidity
    let flash_deposit = 1_000_000 * TOKEN_UNIT;
    let usdc_deposit  = 500_000 * TOKEN_UNIT;
    std::println!(
        "[flash/usdc] BEFORE approve/transfer: flash on pair={} usdc on pair={}",
        flash.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&flash_usdc_pair) / TOKEN_UNIT
    );
    flash.approve(&god, &flash_usdc_pair, &flash_deposit, &0);
    usdc.approve(&god, &flash_usdc_pair, &usdc_deposit, &0);
    flash.transfer(&god, &flash_usdc_pair, &flash_deposit);
    usdc.transfer(&god, &flash_usdc_pair, &usdc_deposit);
    std::println!(
        "[flash/usdc] AFTER transfer: flash on pair={} usdc on pair={}",
        flash.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&flash_usdc_pair) / TOKEN_UNIT
    );
    let flash_usdc_pair_cli = soroswap_pair::Client::new(&e, &flash_usdc_pair);
    let before_deposit = flash_usdc_pair_cli.get_reserves();
    std::println!(
        "[flash/usdc] reserves BEFORE deposit: flash={} usdc={}",
        before_deposit.0 / TOKEN_UNIT,
        before_deposit.1 / TOKEN_UNIT
    );
    flash_usdc_pair_cli.deposit(&god);
    let after_deposit = flash_usdc_pair_cli.get_reserves();
    std::println!(
        "[flash/usdc] reserves AFTER deposit: flash={} usdc={}",
        after_deposit.0 / TOKEN_UNIT,
        after_deposit.1 / TOKEN_UNIT
    );

    // -- VERBOSE LOGGING for usdc/eurc pair liquidity
    let usdc_eurc_usdc = 250_000 * TOKEN_UNIT;
    let usdc_eurc_eurc = 312_500 * TOKEN_UNIT;
    std::println!(
        "[usdc/eurc] BEFORE approve/transfer: usdc on pair={} eurc on pair={}",
        usdc.balance(&usdc_eurc_pair) / TOKEN_UNIT,
        eurc.balance(&usdc_eurc_pair) / TOKEN_UNIT
    );
    usdc.approve(&god, &usdc_eurc_pair, &usdc_eurc_usdc, &0);
    eurc.approve(&god, &usdc_eurc_pair, &usdc_eurc_eurc, &0);
    usdc.transfer(&god, &usdc_eurc_pair, &usdc_eurc_usdc);
    eurc.transfer(&god, &usdc_eurc_pair, &usdc_eurc_eurc);
    std::println!(
        "[usdc/eurc] AFTER transfer: usdc on pair={} eurc on pair={}",
        usdc.balance(&usdc_eurc_pair) / TOKEN_UNIT,
        eurc.balance(&usdc_eurc_pair) / TOKEN_UNIT
    );
    let usdc_eurc_pair_cli = soroswap_pair::Client::new(&e, &usdc_eurc_pair);
    let before_deposit2 = usdc_eurc_pair_cli.get_reserves();
    std::println!(
        "[usdc/eurc] reserves BEFORE deposit: usdc={} eurc={}",
        before_deposit2.0 / TOKEN_UNIT,
        before_deposit2.1 / TOKEN_UNIT
    );
    usdc_eurc_pair_cli.deposit(&god);
    let after_deposit2 = usdc_eurc_pair_cli.get_reserves();
    std::println!(
        "[usdc/eurc] reserves AFTER deposit: usdc={} eurc={}",
        after_deposit2.0 / TOKEN_UNIT,
        after_deposit2.1 / TOKEN_UNIT
    );

    let mgr_id = e.register(FlashCampaignManager, ());
    let mgr = FlashCampaignManagerClient::new(&e, &mgr_id);
    mgr.initialize(&god, &flash.address, &usdc.address);

    (e, mgr, god, users, flash, usdc, eurc, factory_addr, flash_usdc_pair, usdc_eurc_pair)
}

fn create_token<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let wasm = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &wasm.address()),
        token::StellarAssetClient::new(e, &wasm.address()),
    )
}

fn generate_volume_on_pair(
    e: &Env,
    pair: &Address,
    token_a: &token::Client,
    token_b: &token::Client,
    user: &Address,
) {
    token_a.transfer(user, pair, &(1_000 * TOKEN_UNIT));
    let pair_cli = soroswap_pair::Client::new(e, pair);
    let before = pair_cli.get_reserves();
    pair_cli.swap(&(1_000 * TOKEN_UNIT), &0, user);
    let after = pair_cli.get_reserves();
    std::println!("[VOLUME] swap token_a->b | before: {:?}, after: {:?}", before, after);

    token_b.transfer(user, pair, &(800 * TOKEN_UNIT));
    let before = pair_cli.get_reserves();
    pair_cli.swap(&0, &(800 * TOKEN_UNIT), user);
    let after = pair_cli.get_reserves();
    std::println!("[VOLUME] swap token_b->a | before: {:?}, after: {:?}", before, after);
}

// The test cases:
#[test]
fn test_initialize_contract_and_seeding() {
    let (e, mgr, god, _users, flash, _usdc, _eurc, _factory_addr, flash_usdc_pair, usdc_eurc_pair) =
        fresh_env();

    assert!(flash.balance(&mgr.address) > 0);

    let flash_usdc_pair_cli = soroswap_pair::Client::new(&e, &flash_usdc_pair);
    let (rf, ru) = flash_usdc_pair_cli.get_reserves();
    assert!(rf > 0 && ru > 0);

    let usdc_eurc_pair_cli = soroswap_pair::Client::new(&e, &usdc_eurc_pair);
    let (r0, r1) = usdc_eurc_pair_cli.get_reserves();
    assert!(r0 > 0 && r1 > 0);

    dump(&e, "setup_and_initialize");
}

#[test]
fn test_create_campaign_single_user() {
    let (e, mgr, _god, users, _flash, usdc, _eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
        fresh_env();
    let alice = users.get(0).unwrap();
    std::println!(
        "[TEST] Balances: alice usdc={}",
        usdc.balance(&alice) / TOKEN_UNIT
    );
    let cid = mgr.create_campaign(
        &(500 * TOKEN_UNIT),
        &usdc_eurc_pair,
        &10,
        &0,
        &0,
        &alice,
    );
    dump(&e, "create_campaign_single_user");
    let key: Val = (PREFIX_UPOS, cid, alice.clone()).into_val(&e);
    assert!(!e.storage().instance().has(&key));
}
/*
#[test]
fn test_create_and_join_campaign_two_users() {
    let (e, mgr, _god, users, _flash, usdc, _eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
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