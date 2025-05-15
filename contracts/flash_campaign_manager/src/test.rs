#![cfg(test)]

use super::*;

use soroban_sdk::{
    symbol_short,
    testutils::{
        Address as _,
        AuthorizedFunction,
        Ledger,
        Logs,
    },
    token, Address, Env, IntoVal, Symbol, Val,
};
extern crate std;

/*───────────────────────────────── helpers – token ───────────────────────────*/
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

/*───────────────────────────── tiny MockPair implementation ──────────────────*/
#[contract] pub struct MockPair;

#[contractimpl]
impl MockPair {
    fn k_rf() -> Symbol { symbol_short!("rf") }
    fn k_ru() -> Symbol { symbol_short!("ru") }
    fn k_lp() -> Symbol { symbol_short!("lp") }
    fn k_t0() -> Symbol { symbol_short!("t0") }
    fn k_t1() -> Symbol { symbol_short!("t1") }

    fn set<T: IntoVal<Env, Val>>(e: &Env, k: Symbol, v: T) { e.storage().instance().set(&k,&v) }
    fn geti(e: &Env, k: Symbol) -> i128    { e.storage().instance().get(&k).unwrap() }
    fn geta(e: &Env, k: Symbol) -> Address { e.storage().instance().get(&k).unwrap() }

    pub fn init(e: Env, t0: Address, t1: Address, rf: i128, ru: i128) {
        Self::set(&e, Self::k_t0(), t0);
        Self::set(&e, Self::k_t1(), t1);
        Self::set(&e, Self::k_rf(), rf);
        Self::set(&e, Self::k_ru(), ru);
        Self::set(&e, Self::k_lp(), 0_i128);
    }

    pub fn token_0(e: Env) -> Address { Self::geta(&e, Self::k_t0()) }
    pub fn token_1(e: Env) -> Address { Self::geta(&e, Self::k_t1()) }
    pub fn get_reserves(e: Env) -> (i128,i128){
        (Self::geti(&e,Self::k_rf()), Self::geti(&e,Self::k_ru()))
    }
    pub fn swap(e:Env,out0:i128,out1:i128,to:Address){
        let t0=Self::geta(&e,Self::k_t0());
        let t1=Self::geta(&e,Self::k_t1());
        Self::set(&e,Self::k_rf(),Self::geti(&e,Self::k_rf())-out0);
        Self::set(&e,Self::k_ru(),Self::geti(&e,Self::k_ru())-out1);
        let me=e.current_contract_address();
        if out0>0 { token::Client::new(&e,&t0).transfer(&me,&to,&out0); }
        if out1>0 { token::Client::new(&e,&t1).transfer(&me,&to,&out1); }
    }
    pub fn deposit(e:Env,_to:Address)->i128{
        let minted=42;
        Self::set(&e,Self::k_lp(),Self::geti(&e,Self::k_lp())+minted);
        minted
    }
    pub fn withdraw(e:Env,to:Address)->(i128,i128){
        let (rf,ru)=Self::get_reserves(e.clone());
        let t0=Self::geta(&e,Self::k_t0());
        let t1=Self::geta(&e,Self::k_t1());
        let me=e.current_contract_address();
        if rf>0 { token::Client::new(&e,&t0).transfer(&me,&to,&rf); }
        if ru>0 { token::Client::new(&e,&t1).transfer(&me,&to,&ru); }
        Self::set(&e,Self::k_rf(),0);
        Self::set(&e,Self::k_ru(),0);
        (rf,ru)
    }
    pub fn transfer(_e:Env,_from:Address,_to:Address,_amount:i128){}
}

/*────────────────────────────── test-bed bootstrap ───────────────────────────*/
#[allow(clippy::type_complexity)]
fn setup<'a>() -> (
    Env,
    FlashCampaignManagerClient<'a>,
    Address, Address,
    token::Client<'a>, token::Client<'a>,
    Address
){
    let e = Env::default();
    e.mock_all_auths();

    let (flash, flash_admin) = create_token(&e,&Address::generate(&e));
    let (usdc , usdc_admin ) = create_token(&e,&Address::generate(&e));

    let alice = Address::generate(&e);
    let bob   = Address::generate(&e);

    flash_admin.mint(&alice,&1_000_000);
    flash_admin.mint(&bob  ,&1_000_000);
    usdc_admin .mint(&alice,&1_000_000);
    usdc_admin .mint(&bob  ,&1_000_000);

    let pair = e.register(MockPair, ());
    e.as_contract(&pair, || {
        MockPair::init(
            e.clone(),
            flash.address.clone(),
            usdc.address.clone(),
            100_000, 100,
        );
    });
    flash_admin.mint(&pair,&100_000);
    usdc_admin .mint(&pair,&100);

    let mgr_addr = e.register(FlashCampaignManager, ());
    let mgr      = FlashCampaignManagerClient::new(&e,&mgr_addr);
    mgr.initialize(&alice,&flash.address,&usdc.address);

    (e,mgr,alice,bob,flash,usdc,pair)
}

/*────────────────────────── pretty log dump helper ───────────────────────────*/
fn dump(e:&Env,label:&str){
    std::println!("── logs after {label} ─────────────────────────────");
    for l in e.logs().all(){ std::println!("{l}"); }
    std::println!("──────────────────────────────────────────────────\n");
}

/*──────────────────────────────── test 1 ─────────────────────────────────────*/
#[test]
fn create_and_join_campaign(){
    let (e,mgr,alice,bob,_,_,pair)=setup();

    let cid = mgr.create_campaign(&1_000,&pair,&10,&0,&0,&alice);
    dump(&e,"create_campaign");

    /* join (auth capture already enabled from first mock_all_auths) */
    mgr.join_campaign(&cid,&2_000,&bob);
    dump(&e,"join_campaign");

    /* UserPos present */
    let key:Val = (PREFIX_UPOS,cid,bob.clone()).into_val(&e);
    e.as_contract(&mgr.address,|| assert!(e.storage().instance().has(&key)));

    /* ensure SOME auth entry corresponds to join_campaign */
    assert!(
        e.auths().iter().any(|(_,inv)| matches!(
            &inv.function,
            AuthorizedFunction::Contract((addr,sym,_))
                if addr==&mgr.address && *sym==Symbol::new(&e,"join_campaign")
        )),
        "no join_campaign auth captured"
    );
}

/*──────────────────────────────── test 2 ─────────────────────────────────────*/
#[test]
fn compound_updates_reward_pool(){
    let (e,mgr,alice,bob,flash,usdc,pair)=setup();

    /* fund manager so compound’s fee-transfer path can succeed */
    token::StellarAssetClient::new(&e,&flash.address).mint(&mgr.address,&1_000_000);
    token::StellarAssetClient::new(&e,&usdc.address) .mint(&mgr.address,&1_000_000);

    let cid = mgr.create_campaign(&1_000,&pair,&5,&0,&0,&alice);
    mgr.join_campaign(&cid,&2_000,&bob);

    mgr.compound(&cid);          // must not panic
    dump(&e,"compound");
}

/*──────────────────────────────── test 3 ─────────────────────────────────────*/
#[test]
fn claim_after_unlock(){
    let (e,mgr,alice,bob,flash,usdc,pair)=setup();

    token::StellarAssetClient::new(&e,&flash.address).mint(&mgr.address,&1_000_000);
    token::StellarAssetClient::new(&e,&usdc.address) .mint(&mgr.address,&1_000_000);

    let cid = mgr.create_campaign(&1_000,&pair,&5,&0,&0,&alice);
    mgr.join_campaign(&cid,&2_000,&bob);

    e.ledger().with_mut(|li| li.sequence_number += 10 );

    mgr.claim(&cid,&bob);
    dump(&e,"claim");

    let key:Val = (PREFIX_UPOS,cid,bob.clone()).into_val(&e);
    e.as_contract(&mgr.address,|| assert!(!e.storage().instance().has(&key)));
}
