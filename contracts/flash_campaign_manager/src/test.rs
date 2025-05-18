#![cfg(test)]

use super::*;

use soroban_sdk::{
    testutils::{Address as _, Ledger, Logs}, // ← BytesN no longer needed
    token, Address, Env, IntoVal, Val,
};
extern crate std;

/*────────────── bring the *real* Soroswap pair ───────────────*/
mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
}

/*──────────────────────── helpers – token ────────────────────*/
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

/*────────────────── shared test-bed bootstrap ─────────────────*/
#[allow(clippy::type_complexity)]
fn setup<'a>() -> (
    Env,
    FlashCampaignManagerClient<'a>,
    Address, Address,
    token::Client<'a>, token::Client<'a>,
    Address           // ← pair
) {
    let e = Env::default();
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    /* two arbitrary ERC-20-style tokens */
    let (flash, flash_admin) = create_token(&e, &Address::generate(&e));
    let (usdc , usdc_admin ) = create_token(&e, &Address::generate(&e));

    /* actors */
    let alice = Address::generate(&e);
    let bob   = Address::generate(&e);

    flash_admin.mint(&alice, &1_000_000);
    flash_admin.mint(&bob  , &1_000_000);
    usdc_admin .mint(&alice, &1_000_000);
    usdc_admin .mint(&bob  , &1_000_000);

    /* deploy real Soroswap pair */
    let pair = e.register(soroswap_pair::WASM, ());
    let pair_client = soroswap_pair::Client::new(&e, &pair);
    pair_client.initialize(&flash.address, &usdc.address, &alice);

    /* seed initial reserves: 100 000 FLASH : 100 USDC */
    token::Client::new(&e, &flash.address).transfer(&alice, &pair, &100_000);
    token::Client::new(&e, &usdc .address).transfer(&alice, &pair, &100);
    pair_client.deposit(&alice);           // mints LP to alice; updates reserves

    /* campaign manager under test */
    let mgr_id = e.register(FlashCampaignManager, ());
    let mgr    = FlashCampaignManagerClient::new(&e, &mgr_id);
    mgr.initialize(&alice, &flash.address, &usdc.address);

    (e, mgr, alice, bob, flash, usdc, pair)
}

/*────────────── helper to print logs + resources ──────────────*/
fn dump(e: &Env, label: &str) {
    std::println!("── logs after {label} ─────────────────────────────");
    for l in e.logs().all() {
        std::println!("{l}");
    }
    std::println!("{:#?}", e.cost_estimate().resources());
    std::println!("──────────────────────────────────────────────────\n");
}

/*──────────────────────────── test 1 ──────────────────────────*/
#[test]
fn create_and_join_campaign() {
    let (e, mgr, alice, bob, _, _, pair) = setup();

    let cid = mgr.create_campaign(&1_000, &pair, &10, &0, &0, &alice);
    dump(&e, "create_campaign");

    mgr.join_campaign(&cid, &2_000, &bob);
    dump(&e, "join_campaign");

    /* UserPos exists */
    let key: Val = (PREFIX_UPOS, cid, bob.clone()).into_val(&e);
    e.as_contract(&mgr.address, || assert!(e.storage().instance().has(&key)));
}

/*──────────────────────────── test 2 ──────────────────────────*/
#[test]
fn compound_updates_reward_pool() {
    let (e, mgr, alice, bob, flash, usdc, pair) = setup();

    token::StellarAssetClient::new(&e, &flash.address).mint(&mgr.address, &1_000_000);
    token::StellarAssetClient::new(&e, &usdc .address).mint(&mgr.address, &1_000_000);

    let cid = mgr.create_campaign(&1_000, &pair, &5, &0, &0, &alice);
    mgr.join_campaign(&cid, &2_000, &bob);

    mgr.compound(&cid);
    dump(&e, "compound");
}

/*──────────────────────────── test 3 ──────────────────────────*/
#[test]
fn claim_after_unlock() {
    let (e, mgr, alice, bob, flash, usdc, pair) = setup();

    token::StellarAssetClient::new(&e, &flash.address).mint(&mgr.address, &1_000_000);
    token::StellarAssetClient::new(&e, &usdc .address).mint(&mgr.address, &1_000_000);

    let cid = mgr.create_campaign(&1_000, &pair, &5, &0, &0, &alice);
    mgr.join_campaign(&cid, &2_000, &bob);

    e.ledger().with_mut(|li| li.sequence_number += 10);

    mgr.claim(&cid, &bob);
    dump(&e, "claim");

    let key: Val = (PREFIX_UPOS, cid, bob.clone()).into_val(&e);
    e.as_contract(&mgr.address, || assert!(!e.storage().instance().has(&key)));
}
