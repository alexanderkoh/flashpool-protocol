//! flash_campaign_manager
//! -------------------------------------------------------------
//! Verbose build – every major step emits a plain-text log.

#![no_std]

// -------------------------------------------------------------
// Imports
// -------------------------------------------------------------
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, panic_with_error,
    token::Client as TokenClient,
    Address, Env, IntoVal, Symbol, Val, unwrap::UnwrapOptimized,
};

// bring the real Soroswap pair WASM (for on-chain build)
pub mod pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
}
extern crate std;

// -------------------------------------------------------------
// Errors
// -------------------------------------------------------------
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FlashErr {
    AlreadyInit    = 1,
    Math           = 2,
    BpsOutOfRange  = 3,
    TooEarly       = 4,
    NothingToClaim = 5,
    NotAdmin       = 6,
}

/// assert-style helper that logs **before** panicking
macro_rules! ensure {
    ($env:expr, $cond:expr, $err:expr) => {
        if !$cond {
            //log!($env, "ensure! failed – code {}", $err as u32);
            panic_with_error!($env, $err)
        }
    };
}

// -------------------------------------------------------------
// Storage keys / constants
// -------------------------------------------------------------
const KEY_ADMIN: &str = "A";
const KEY_FLASH: &str = "F";
const KEY_USDC : &str = "U";
const KEY_NEXT : &str = "N";
const KEY_SURP : &str = "S";
const KEY_TTLT : &str = "T";
const KEY_TTLB : &str = "B";

const PREFIX_CAMP: &str = "C";
const PREFIX_UPOS: &str = "U";

const MAX_BPS:             u32 = 10_000;
const DEFAULT_SURPLUS_BPS: u32 = 500;      // 5 %
const DEFAULT_TTL_THRESH:  u32 = 172_800;  // 10 days
const DEFAULT_TTL_BUMP:    u32 = 241_920;  // 14 days bump

// -------------------------------------------------------------
// Data types
// -------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pair:          Address,
    duration:      u32,
    end_ledger:    u32,
    target_lp:     i128,
    total_lp:      i128,
    total_weight:  i128,
    reward_flash:  i128,
    bonus_flash:   i128,
    stake_lp:      i128,
}

#[contracttype]
#[derive(Clone)]
pub struct UserPos { lp: i128, weight: i128 }

// -------------------------------------------------------------
// Helpers
// -------------------------------------------------------------
fn int_sqrt(x: u128) -> u128 {
    if x <= 1 { return x }
    let (mut z, mut y) = (x, (x >> 1) + 1);
    while y < z { z = y; y = (x / y + y) >> 1; }
    z
}

fn s(e:&Env,k:&'static str)->Symbol { Symbol::new(e,k) }
fn set_addr(e:&Env,k:&'static str,a:&Address){ e.storage().instance().set(&s(e,k),a) }
fn get_addr(e:&Env,k:&'static str)->Address { e.storage().instance().get(&s(e,k)).unwrap_optimized() }
fn set_u32(e:&Env,k:&'static str,v:u32){ e.storage().instance().set(&s(e,k),&v) }
fn get_u32(e:&Env,k:&'static str,d:u32)->u32{ e.storage().instance().get(&s(e,k)).unwrap_or(d) }

fn bump(e:&Env){
    e.storage().instance().extend_ttl(
        get_u32(e,KEY_TTLT,DEFAULT_TTL_THRESH),
        get_u32(e,KEY_TTLB,DEFAULT_TTL_BUMP));
}

fn camp_key(e:&Env,id:u32)->Val { (PREFIX_CAMP,id).into_val(e) }
fn upos_key(e:&Env,id:u32,w:&Address)->Val { (PREFIX_UPOS,id,w).into_val(e) }

fn load_camp(e:&Env,id:u32)->Campaign {
    e.storage().instance().get::<Val,Campaign>(&camp_key(e,id)).unwrap_optimized()
}
fn save_camp(e:&Env,id:u32,c:&Campaign){
    e.storage().instance().set::<Val,Campaign>(&camp_key(e,id),c)
}

// -------------------------------------------------------------
// Swap helper (USDC -> FLASH)
// -------------------------------------------------------------
fn swap_usdc_to_flash(
    e:&Env, pair:&Address, usdc_amt:i128, flash:Address, usdc:Address
)->i128{
    if usdc_amt == 0 { return 0 }
    let p = pair::Client::new(e,pair);
    let (rf,ru) = p.get_reserves();
    TokenClient::new(e,&usdc).transfer(&e.current_contract_address(),pair,&usdc_amt);
    let out = rf.checked_mul(usdc_amt).unwrap().checked_div(ru+usdc_amt).unwrap();
    let (o0,o1) = if flash<usdc {(out,0)} else {(0,out)};
    p.swap(&o0,&o1,&e.current_contract_address());
    //log!(&e, "[SWAP] usdc_to_flash {} USDC -> {} FLASH (rf {}, ru {})", usdc_amt, out, rf, ru);
    out
}

// -------------------------------------------------------------
// Contract interface
// -------------------------------------------------------------
pub trait Manager {
    fn initialize         (e:Env, admin:Address, flash:Address, usdc:Address);

    fn create_campaign    (e:Env, fee_usdc:i128, pair:Address,
                           unlock:u32, target_lp:i128, bonus_flash:i128,
                           creator:Address) -> u32;

    fn join_campaign      (e:Env,id:u32, token0_amt:i128, user:Address);
    fn compound           (e:Env,id:u32);
    fn claim              (e:Env,id:u32, user:Address);

    fn set_surplus_bps    (e:Env, admin:Address, bps:u32);
    fn set_ttl            (e:Env, admin:Address, threshold:u32, bump:u32);
}

#[contract] pub struct FlashCampaignManager;

// -------------------------------------------------------------
// Implementation
// -------------------------------------------------------------
#[contractimpl]
impl Manager for FlashCampaignManager {
// ------------------------------------------------- init -------
fn initialize(e:Env, admin:Address, flash:Address, usdc:Address){
    bump(&e);
    ensure!(&e, !e.storage().instance().has(&s(&e,KEY_ADMIN)), FlashErr::AlreadyInit);

    admin.require_auth();
    set_addr(&e,KEY_ADMIN,&admin);
    set_addr(&e,KEY_FLASH,&flash);
    set_addr(&e,KEY_USDC ,&usdc );
    set_u32(&e,KEY_NEXT,0);
    set_u32(&e,KEY_SURP,DEFAULT_SURPLUS_BPS);
    set_u32(&e,KEY_TTLT,DEFAULT_TTL_THRESH);
    set_u32(&e,KEY_TTLB,DEFAULT_TTL_BUMP);

    //log!(&e, "[INITIALIZE] initialize(admin {:?})", admin);

    let bal = TokenClient::new(&e,&flash).balance(&admin);
    if bal > 0 {
        TokenClient::new(&e,&flash)
            .transfer(&admin,&e.current_contract_address(),&bal);
        //log!(&e, "[INITIALIZE] pulled {} FLASH from admin", bal);
    }
}

// -------------------------------------- create_campaign -------
fn create_campaign(
    e: Env, fee_usdc: i128, target_pair: Address,
    unlock: u32, target_lp: i128, bonus_flash: i128, creator: Address
) -> u32 {
    std::println!("── THIS IS THE CREATE CAMPAIGN LOGGING IN STD-──");

    bump(&e);
    creator.require_auth();

    let flash  = get_addr(&e, KEY_FLASH);
    let usdc   = get_addr(&e, KEY_USDC);
    let surplus_bps = get_u32(&e, KEY_SURP, DEFAULT_SURPLUS_BPS);
    ensure!(&e, surplus_bps < MAX_BPS, FlashErr::BpsOutOfRange);

    std::println!("Fee incoming: {fee_usdc} USDC from {creator:?}");

    // 1. fee: take all USDC from creator
    TokenClient::new(&e, &usdc)
        .transfer(&creator, &e.current_contract_address(), &fee_usdc);

    // 2. reserves before
    let pcli = pair::Client::new(&e, &target_pair);
    let (rf0, ru0) = pcli.get_reserves();
    std::println!("Pair reserves before: FLASH={rf0}, USDC={ru0}");

    // 3. split fee: s = amount to swap, l = amount to add directly
    let s_min = int_sqrt((ru0 as u128) * (ru0 as u128 + fee_usdc as u128)) as i128 - ru0;
    let s     = (s_min + fee_usdc * surplus_bps as i128 / MAX_BPS as i128).min(fee_usdc);
    let l     = fee_usdc - s;
    std::println!("Fee split: swap {s} USDC, add_liq {l} USDC (s_min={s_min})");

    // 4. swap: swap `s` USDC for flash, result = flash_out
    TokenClient::new(&e, &usdc)
        .transfer(&e.current_contract_address(), &target_pair, &s);

    let flash_out = rf0.checked_mul(s).unwrap().checked_div(ru0 + s).unwrap();
    let (o0, o1) = if flash < usdc { (flash_out, 0) } else { (0, flash_out) };
    std::println!("Swapping s={s} for flash_out={flash_out}, o0={o0}, o1={o1}");
    pcli.swap(&o0, &o1, &e.current_contract_address());

    // 5. calculate amounts to deposit for liquidity (matching pool ratio)
    let ru_swap = ru0 + s;           // reserves of USDC after swap
    let rf_swap = rf0 - flash_out;   // reserves of FLASH after swap
    std::println!("Post-swap reserves: FLASH={rf_swap}, USDC={ru_swap}");

    // How much flash needs to be deposited to match ratio with l USDC?
    let flash_needed = if ru_swap > 0 {
        l.checked_mul(rf_swap).unwrap().checked_div(ru_swap).unwrap_or(0)
    } else {
        0
    };
    std::println!("Calculated flash_needed={flash_needed} for l={l} USDC (rf_swap={rf_swap}, ru_swap={ru_swap})");
    // flash_out is already on the pair, so only deposit the difference (never negative)
    let flash_to_deposit = flash_needed.saturating_sub(flash_out);
    std::println!("flash_to_deposit (actual needed to transfer) = {flash_to_deposit}");

    // Actually transfer tokens for deposit
    if flash_to_deposit > 0 {
        std::println!("Transferring extra flash_to_deposit={flash_to_deposit} to pair");
        TokenClient::new(&e, &flash)
            .transfer(&e.current_contract_address(), &target_pair, &flash_to_deposit);
    }
    if l > 0 {
        std::println!("Transferring l={l} USDC to pair");
        TokenClient::new(&e, &usdc)
            .transfer(&e.current_contract_address(), &target_pair, &l);
    }

    // For debugging: get how much was actually transferred
    let t0 = pcli.token_0();
    let t1 = pcli.token_1();
    let t0_bal = TokenClient::new(&e, &t0).balance(&e.current_contract_address());
    let t1_bal = TokenClient::new(&e, &t1).balance(&e.current_contract_address());
    std::println!("Contract balances before approve: t0({t0:?})={t0_bal}, t1({t1:?})={t1_bal}");

    // Approve all for deposit
    TokenClient::new(&e, &t0).approve(&e.current_contract_address(), &target_pair, &t0_bal, &0);
    TokenClient::new(&e, &t1).approve(&e.current_contract_address(), &target_pair, &t1_bal, &0);

    // 6. deposit: this will mint LP
    std::println!("Depositing into pair...");
    let lp_minted = pcli.deposit(&e.current_contract_address());
    ensure!(&e, lp_minted > 0, FlashErr::Math);

    std::println!("Deposit complete. lp_minted={lp_minted}");

    // 7. reward pool calculation (using actual amounts)
    // The reserves before this deposit (after swap) are rf_swap/ru_swap.
    // The actual deposited amounts:
    let actual_usdc = TokenClient::new(&e, &usdc).balance(&target_pair) - ru_swap;
    let actual_flash = TokenClient::new(&e, &flash).balance(&target_pair) - rf_swap;
    std::println!("Actual deposit amounts: actual_usdc={actual_usdc}, actual_flash={actual_flash}");

    let ru1 = ru_swap + actual_usdc;
    let rf1 = rf_swap + actual_flash;
    let root = int_sqrt((ru1 as u128 * rf1 as u128 * rf0 as u128) / (ru0 as u128));
    let x_max = if root > rf1 as u128 { (root - rf1 as u128) as i128 } else { 0 };

    // Surplus = how much flash (from swap + deposit) is left after matching the pool ratio
    let surplus = flash_out + actual_flash - flash_to_deposit;
    let reward_flash = surplus.min(x_max);

    std::println!(
        "Reward pool calculation: ru1={ru1}, rf1={rf1}, root={root}, x_max={x_max}, surplus={surplus}, reward_flash={reward_flash}"
    );

    // 8. persist
    let id = get_u32(&e, KEY_NEXT, 0) + 1;
    set_u32(&e, KEY_NEXT, id);
    save_camp(&e, id, &Campaign{
        pair: target_pair.clone(), duration: unlock,
        end_ledger: e.ledger().sequence() + unlock,
        target_lp, total_lp: lp_minted, total_weight: 0,
        reward_flash, bonus_flash, stake_lp: lp_minted
    });
    std::println!("Campaign created: id={id}, pair={target_pair:?}, reward_flash={reward_flash}, total_lp={lp_minted}");

    id
}




// ------------------------------------------- join_campaign ---
fn join_campaign(e:Env,id:u32, token0_amt:i128, user:Address){
    bump(&e);
    user.require_auth();
    ensure!(&e, token0_amt>0, FlashErr::Math);

    let mut c   = load_camp(&e,id);
    let pcli    = pair::Client::new(&e,&c.pair);
    let t0      = pcli.token_0();
    let t1      = pcli.token_1();
    let t0cli   = TokenClient::new(&e,&t0);

    t0cli.transfer(&user,&e.current_contract_address(),&token0_amt);
    //log!(&e, "[JOIN CAMPAIGN] deposit {} of token0 from {:?}", token0_amt, user);

    let half  = token0_amt/2;
    t0cli.transfer(&e.current_contract_address(),&c.pair,&half);
    let (r0,r1)= pcli.get_reserves();
    let t1_out = r1.checked_mul(half).unwrap().checked_div(r0+half).unwrap();
    let (o0,o1)= if t0<t1 {(0,t1_out)} else {(t1_out,0)};
    pcli.swap(&o0,&o1,&e.current_contract_address());
    //log!(&e, "[JOIN CAMPAIGN] swap_half: {} token0 => {} token1", half, t1_out);

    t0cli.transfer(&e.current_contract_address(),&c.pair,&(token0_amt-half));
    TokenClient::new(&e,&t1).transfer(&e.current_contract_address(),&c.pair,&t1_out);

    // **APPROVE BEFORE DEPOSIT**
    let t0_bal = TokenClient::new(&e, &t0).balance(&e.current_contract_address());
    let t1_bal = TokenClient::new(&e, &t1).balance(&e.current_contract_address());
    TokenClient::new(&e, &t0).approve(&e.current_contract_address(), &c.pair, &t0_bal, &0);
    TokenClient::new(&e, &t1).approve(&e.current_contract_address(), &c.pair, &t1_bal, &0);

    let lp = pcli.deposit(&e.current_contract_address());
    ensure!(&e, lp>0, FlashErr::Math);
    //log!(&e, "[JOIN CAMPAIGN] minted {} LP for user {:?}", lp, user);

    let now = e.ledger().sequence();
    let w   = lp * (c.end_ledger-now) as i128 / c.duration as i128;
    c.total_lp     += lp;
    c.total_weight += w;
    save_camp(&e,id,&c);

    let key = upos_key(&e,id,&user);
    let mut up:UserPos = e.storage().instance().get(&key).unwrap_or(UserPos{lp:0,weight:0});
    up.lp+=lp; up.weight+=w;
    e.storage().instance().set(&key,&up);

    //log!(&e, "[JOIN CAMPAIGN] join(id {}) user {:?} lp {} weight {}", id, user, lp, w);
}

// -------------------------------------------------- compound ---
fn compound(e:Env,id:u32){
    bump(&e);
    let mut c  = load_camp(&e,id);
    let pcli   = pair::Client::new(&e,&c.pair);
    let (a0,a1)= pcli.withdraw(&e.current_contract_address());
    //log!(&e, "[COMPOUND] withdraw_from_pair token0 {} token1 {}", a0, a1);

    let t0 = pcli.token_0(); let t1 = pcli.token_1();
    TokenClient::new(&e,&t0).transfer(&e.current_contract_address(),&c.pair,&a0);
    TokenClient::new(&e,&t1).transfer(&e.current_contract_address(),&c.pair,&a1);

    // **APPROVE BEFORE DEPOSIT**
    let t0_bal = TokenClient::new(&e, &t0).balance(&e.current_contract_address());
    let t1_bal = TokenClient::new(&e, &t1).balance(&e.current_contract_address());
    TokenClient::new(&e, &t0).approve(&e.current_contract_address(), &c.pair, &t0_bal, &0);
    TokenClient::new(&e, &t1).approve(&e.current_contract_address(), &c.pair, &t1_bal, &0);

    let lp_new = pcli.deposit(&e.current_contract_address());
    ensure!(&e, lp_new>0, FlashErr::Math);
    //log!(&e, "[COMPOUND] re-deposit minted {} LP", lp_new);

    let mut gain = 0i128;
    let fee_lp   = lp_new - c.stake_lp;
    if fee_lp>0 {
        TokenClient::new(&e,&c.pair).transfer(
            &e.current_contract_address(),&c.pair,&fee_lp);
        let (f0,f1)= pcli.withdraw(&e.current_contract_address());
        let flash  = get_addr(&e,KEY_FLASH);
        let usdc   = get_addr(&e,KEY_USDC);
        if t0==flash { gain += f0 } else if t0==usdc { gain+=swap_usdc_to_flash(&e,&c.pair,f0,flash.clone(),usdc.clone()) }
        if t1==flash { gain += f1 } else if t1==usdc { gain+=swap_usdc_to_flash(&e,&c.pair,f1,flash,usdc) }
        c.reward_flash += gain;
        //log!(&e, "[COMPOUND] performance_fee +{} FLASH into pool", gain);
    }
    c.stake_lp = lp_new;
    save_camp(&e,id,&c);

    //log!(&e, "[COMPOUND] compound(id {}) fee_lp {} gain {}", id, fee_lp, gain);
}

// ------------------------------------------------------ claim --
fn claim(e:Env,id:u32, user:Address){
    bump(&e);
    user.require_auth();

    let c  = load_camp(&e,id);
    ensure!(&e, e.ledger().sequence()>=c.end_ledger, FlashErr::TooEarly);

    let key = upos_key(&e,id,&user);
    let up :UserPos = e.storage().instance().get(&key).unwrap_optimized();
    ensure!(&e, up.weight>0, FlashErr::NothingToClaim);

    let base  = c.reward_flash * up.weight / c.total_weight;
    let bonus = if c.total_lp>=c.target_lp { c.bonus_flash * up.weight / c.total_weight } else {0};
    let total = base+bonus;

    TokenClient::new(&e,&get_addr(&e,KEY_FLASH)).transfer(&e.current_contract_address(),&user,&total);
    TokenClient::new(&e,&c.pair).transfer(&e.current_contract_address(),&user,&up.lp);
    e.storage().instance().remove(&key);

    //log!(&e, "[CLAIM] id {:?} user {:?} flash {:?} lp {:?}", id, user, total, up.lp);
}

// ------------------------------------------- admin helpers ----
fn set_surplus_bps(e:Env, admin:Address, bps:u32){
    admin.require_auth();
    ensure!(&e, admin == get_addr(&e,KEY_ADMIN), FlashErr::NotAdmin);
    ensure!(&e, bps<MAX_BPS, FlashErr::BpsOutOfRange);
    set_u32(&e,KEY_SURP,bps);
    //log!(&e, "[ADMIN] surplus_bps set to {}", bps);
}

fn set_ttl(e:Env, admin:Address, threshold:u32, bump_:u32){
    admin.require_auth();
    ensure!(&e, admin == get_addr(&e,KEY_ADMIN), FlashErr::NotAdmin);
    set_u32(&e,KEY_TTLT,threshold);
    set_u32(&e,KEY_TTLB,bump_);
    //log!(&e, "[ADMIN] ttl threshold {} bump {}", threshold, bump_);
}
}

// -------------------------------------------------------------
#[cfg(test)]
mod test;
