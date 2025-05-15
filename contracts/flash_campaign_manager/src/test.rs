#![cfg(test)]

use super::*;

use soroban_sdk::{
    symbol_short,
    testutils::{
        Address as _,
        AuthorizedFunction,
        AuthorizedInvocation,
        Ledger,               // provides `.with_mut`
        Logs,
    },
    token, Address, Env, IntoVal, Symbol, Val,
};
extern crate std;

/*───────────────────────────────────────────────────────────────*
 * helpers – deploy a reference token contract                   *
 *───────────────────────────────────────────────────────────────*/
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

/*───────────────────────────────────────────────────────────────*
 * a tiny in-memory AMM + LP-token pair                          *
 *───────────────────────────────────────────────────────────────*/
#[contract] pub struct MockPair;

#[contractimpl]
impl MockPair {
    /* storage keys */
    fn k_rf() -> Symbol { symbol_short!("rf") }   // FLASH reserve
    fn k_ru() -> Symbol { symbol_short!("ru") }   // USDC  reserve
    fn k_lp() -> Symbol { symbol_short!("lp") }   // total LP minted
    fn k_t0() -> Symbol { symbol_short!("t0") }
    fn k_t1() -> Symbol { symbol_short!("t1") }

    /* helpers */
    fn set<T: IntoVal<Env, Val>>(e:&Env,k:Symbol,v:T){ e.storage().instance().set(&k,&v) }
    fn geti(e:&Env,k:Symbol)->i128    { e.storage().instance().get(&k).unwrap() }
    fn geta(e:&Env,k:Symbol)->Address { e.storage().instance().get(&k).unwrap() }

    /* bootstrap from the tests */
    pub fn init(e:Env,t0:Address,t1:Address,rf:i128,ru:i128){
        Self::set(&e,Self::k_t0(),t0);
        Self::set(&e,Self::k_t1(),t1);
        Self::set(&e,Self::k_rf(),rf);
        Self::set(&e,Self::k_ru(),ru);
        Self::set(&e,Self::k_lp(),0_i128);
    }

    /* interface expected by the manager */
    pub fn token_0(e:Env)->Address { Self::geta(&e,Self::k_t0()) }
    pub fn token_1(e:Env)->Address { Self::geta(&e,Self::k_t1()) }
    pub fn get_reserves(e:Env)->(i128,i128){
        (Self::geti(&e,Self::k_rf()), Self::geti(&e,Self::k_ru()))
    }

    pub fn swap(e:Env,out0:i128,out1:i128,to:Address){
        let t0=Self::geta(&e,Self::k_t0());
        let t1=Self::geta(&e,Self::k_t1());

        Self::set(&e,Self::k_rf(),Self::geti(&e,Self::k_rf())-out0);
        Self::set(&e,Self::k_ru(),Self::geti(&e,Self::k_ru())-out1);

        let me = e.current_contract_address();
        if out0>0 { token::Client::new(&e,&t0).transfer(&me,&to,&out0); }
        if out1>0 { token::Client::new(&e,&t1).transfer(&me,&to,&out1); }
    }

    pub fn deposit(e:Env,_to:Address)->i128{
        let minted = 42;
        Self::set(&e,Self::k_lp(),Self::geti(&e,Self::k_lp())+minted);
        minted
    }

    pub fn withdraw(e:Env,_from:Address)->(i128,i128){
        let r = Self::get_reserves(e.clone());
        Self::set(&e,Self::k_lp(),0);
        r
    }

    /* minimal token-like interface so `TokenClient::transfer`
       in FlashCampaignManager::claim succeeds. */
    pub fn transfer(_e:Env,_from:Address,_to:Address,_amount:i128){
        /* no-op – good enough for unit tests */
    }
}

/*───────────────────────────────────────────────────────────────*
 * test-bed bootstrap                                            *
 *───────────────────────────────────────────────────────────────*/
#[allow(clippy::type_complexity)]
fn setup<'a>() -> (
    Env,
    FlashCampaignManagerClient<'a>,
    Address, Address,                    // alice, bob
    token::Client<'a>, token::Client<'a>,// flash, usdc
    Address                              // pair addr
){
    let e = Env::default();
    e.mock_all_auths();   // every `require_auth` succeeds & is recorded

    /* tokens */
    let (flash, flash_admin) = create_token(&e,&Address::generate(&e));
    let (usdc , usdc_admin ) = create_token(&e,&Address::generate(&e));

    let alice = Address::generate(&e);
    let bob   = Address::generate(&e);

    flash_admin.mint(&alice,&1_000_000);
    flash_admin.mint(&bob  ,&1_000_000);
    usdc_admin .mint(&alice,&1_000_000);
    usdc_admin .mint(&bob  ,&1_000_000);

    /* pair */
    let pair = e.register(MockPair, ());
    e.as_contract(&pair, || {
        MockPair::init(
            e.clone(),
            flash.address.clone(),
            usdc.address.clone(),
            100_000,
            100,
        );
    });
    flash_admin.mint(&pair,&100_000);
    usdc_admin .mint(&pair,&100);

    /* manager */
    let mgr_addr = e.register(FlashCampaignManager, ());
    let mgr      = FlashCampaignManagerClient::new(&e,&mgr_addr);
    mgr.initialize(&alice,&flash.address,&usdc.address);

    (e,mgr,alice,bob,flash,usdc,pair)
}

/*───────────────────────────────────────────────────────────────*
 * helpers – log dump                                            *
 *───────────────────────────────────────────────────────────────*/
fn dump(e:&Env,label:&str){
    std::println!("── logs after {label} ─────────────────────────────");
    for l in e.logs().all(){ std::println!("{l}"); }
    std::println!("──────────────────────────────────────────────────\n");
}

/*───────────────────────────────────────────────────────────────*
 * 1. create + join                                              *
 *───────────────────────────────────────────────────────────────*/
#[test]
fn create_and_join_campaign(){
    let (e,mgr,alice,bob,_,_,pair)=setup();

    let cid = mgr.create_campaign(&1_000,&pair,&10,&0,&0,&alice);
    dump(&e,"create_campaign");

    mgr.join_campaign(&cid,&2_000,&bob);
    dump(&e,"join_campaign");

    /* UserPos key exists */
    let key:Val = (PREFIX_UPOS,cid,bob.clone()).into_val(&e);
    e.as_contract(&mgr.address,|| assert!(e.storage().instance().has(&key)));

    /* authorisation vector contains exactly one entry for the join */
    let expected = std::vec![(
        bob.clone(),
        AuthorizedInvocation{
            function: AuthorizedFunction::Contract((
                mgr.address.clone(),
                Symbol::new(&e,"join_campaign"),
                (&cid,2_000_i128,&bob).into_val(&e),
            )),
            sub_invocations: std::vec![]
        }
    )];
    assert_eq!(e.auths(), expected);
}

/*───────────────────────────────────────────────────────────────*
 * 2. compound runs without panic                                *
 *───────────────────────────────────────────────────────────────*/
#[test]
fn compound_updates_reward_pool(){
    let (e,mgr,alice,bob,flash,usdc,pair)=setup();

    /* give the manager some tokens so internal fee transfers succeed */
    usdc.transfer(&pair,&mgr.address,&1_000);
    flash.transfer(&pair,&mgr.address,&1_000);

    let cid = mgr.create_campaign(&1_000,&pair,&5,&0,&0,&alice);
    mgr.join_campaign(&cid,&2_000,&bob);

    mgr.compound(&cid);          // should not panic
    dump(&e,"compound");
}

/*───────────────────────────────────────────────────────────────*
 * 3. claim after unlock                                         *
 *───────────────────────────────────────────────────────────────*/
#[test]
fn claim_after_unlock(){
    let (e,mgr,alice,bob,flash,usdc,pair)=setup();

    /* ensure manager has some LP + token balances */
    usdc.transfer(&pair,&mgr.address,&1_000);
    flash.transfer(&pair,&mgr.address,&1_000);

    let cid = mgr.create_campaign(&1_000,&pair,&5,&0,&0,&alice);
    mgr.join_campaign(&cid,&2_000,&bob);

    /* fast-forward 10 ledgers */
    e.ledger().with_mut(|li| li.sequence_number += 10 );

    mgr.claim(&cid,&bob);
    dump(&e,"claim");

    /* UserPos removed */
    let key:Val = (PREFIX_UPOS,cid,bob.clone()).into_val(&e);
    e.as_contract(&mgr.address,|| assert!(!e.storage().instance().has(&key)));
}
