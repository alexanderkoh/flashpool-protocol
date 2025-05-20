//! flash_campaign_manager
//! -------------------------------------------------------------
//! Verbose build – every major step emits a plain-text log.

#![no_std]
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::xdr::{
    AccountId, AlphaNum4, Asset, AssetCode4, ContractExecutable, ContractIdPreimage,
    CreateContractArgs, HostFunction, PublicKey, ScAddress, Uint256,
};
#[allow(unused_imports)]
// -------------------------------------------------------------
// Imports
//
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log as _log, panic_with_error, token,
    token::Client as TokenClient, unwrap::UnwrapOptimized, Address, Env, IntoVal, String, Symbol,
    TryIntoVal, Val,
};
// bring the real Soroswap pair WASM (for on-chain build)
pub mod pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
}

mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_factory.wasm");
}

mod rewards;
mod storage;

use storage::*;
//extern crate std;

// -------------------------------------------------------------
// Errors
// -------------------------------------------------------------
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FlashErr {
    AlreadyInit = 1,
    Math = 2,
    BpsOutOfRange = 3,
    TooEarly = 4,
    NothingToClaim = 5,
    NotAdmin = 6,
    Maa = 7,
    Mas = 8,
    Mad = 9,
    Mae = 10,
    Maw = 11,
    Maq = 12,
    Mal = 13,
    Mak = 14,
    Maj = 15,
    Mah = 16,
    Mag = 17,
    Maf = 18,
    Mar = 19,
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

const MAX_BPS: u32 = 10_000;
const DEFAULT_SURPLUS_BPS: u32 = 500; // 5 %
const DEFAULT_TTL_THRESH: u32 = 172_800; // 10 days
const DEFAULT_TTL_BUMP: u32 = 241_920; // 14 days bump

// -------------------------------------------------------------
// Data types
// -------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pair: Address,
    duration: u32,
    end_ledger: u32,
    target_lp: i128,
    total_lp: i128,
    total_weight: i128,
    reward_flash: i128,
    bonus_flash: i128,
    stake_lp: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct UserPos {
    lp: i128,
    weight: i128,
    joined_ledger: u32, // track when user joined
    rank: u32,          // track user rank in campaign
}

// -------------------------------------------------------------
// Helpers
// -------------------------------------------------------------
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
/*/
fn s(e:&Env,k:&'static str)->Symbol { Symbol::new(e,k) }
fn set_addr(e:&Env,k:&'static str,a:&Address){ e.storage().instance().set(&s(e,k),a) }
fn get_addr(e:&Env,k:&'static str)->Address { e.storage().instance().get(&s(e,k)).unwrap_optimized() }
fn set_u32(e:&Env,k:&'static str,v:u32){ e.storage().instance().set(&s(e,k),&v) }
fn get_u32(e:&Env,k:&'static str,d:u32)->u32{ e.storage().instance().get(&s(e,k)).unwrap_or(d) }
*/
fn bump(e: &Env) {
    e.storage().instance().extend_ttl(
        get_u32(e, KEY_TTLT, DEFAULT_TTL_THRESH),
        get_u32(e, KEY_TTLB, DEFAULT_TTL_BUMP),
    );
}
/*
fn camp_key(e:&Env,id:u32)->Val { (PREFIX_CAMP,id).into_val(e) }
fn upos_key(e:&Env,id:u32,w:&Address)->Val { (PREFIX_UPOS,id,w).into_val(e) }

fn load_camp(e:&Env,id:u32)->Campaign {
    e.storage().instance().get::<Val,Campaign>(&camp_key(e,id)).unwrap_optimized()
}
fn save_camp(e:&Env,id:u32,c:&Campaign){
    e.storage().instance().set::<Val,Campaign>(&camp_key(e,id),c)
}
*/
fn symbol_to_code_bytes(symbol: &soroban_sdk::String) -> [u8; 4] {
    let mut code_bytes = [0u8; 4];
    let len = symbol.len().min(4) as usize;
    if len > 0 {
        let mut tmp = [0u8; 4];
        symbol.copy_into_slice(&mut tmp[..len]);
        code_bytes[..len].copy_from_slice(&tmp[..len]);
    }
    code_bytes
}

/*
fn log_pair_creation(
    pair_addr: &Address,
    token0_label: &String,
    token0_addr: &Address,
    token1_label: &String,
    token1_addr: &Address,
) {
    //std::println!("     PAIR CREATED\n          {:?}", pair_addr);
    //std::println!("             token0: {:#?}\n             Address:    {:?}", token0_label, token0_addr);
    //std::println!("             token1: {:#?}\n             Address:    {:?}",        token1_label,        token1_addr,        );
}
 */

fn create_pair_ordered<'a>(
    factory: &soroswap_factory::Client<'a>,
    token_a: &token::Client<'a>,
    token_b: &token::Client<'a>,
    label_a: &String,
    label_b: &String,
) -> Address {
    let pairaddress: Address;
    //std::println!("[CREATE_PAIR_ORDERED] --- CREATING PAIR FOR {:?}/{:?} ---", label_a, label_b);
    if token_a.address < token_b.address {
        pairaddress = factory.create_pair(&token_a.address, &token_b.address);
        // log_pair_creation( &pairaddress,            label_a,            &token_a.address,            label_b,            &token_b.address,        );
    } else {
        pairaddress = factory.create_pair(&token_b.address, &token_a.address);
        // log_pair_creation(            &pairaddress,            label_b,            &token_b.address,            label_a,            &token_a.address,        );
    }
    pairaddress
}

/// Transfers admin of a Stellar Asset Contract to this contract.
pub fn claim_sac_admin(e: Env, sac_contract: Address, current_admin: Address) {
    current_admin.require_auth();
    let sac = StellarAssetClient::new(&e, &sac_contract);
    sac.set_admin(&e.current_contract_address());
}

// -------------------------------------------------------------
// Swap helper (USDC -> FLASH)
// -------------------------------------------------------------
fn swap_usdc_to_flash(
    e: &Env,
    pair: &Address,
    usdc_amt: i128,
    flash: Address,
    usdc: Address,
) -> i128 {
    if usdc_amt == 0 {
        return 0;
    }
    let p = pair::Client::new(e, pair);
    let (rf, ru) = p.get_reserves();
    TokenClient::new(e, &usdc).transfer(&e.current_contract_address(), pair, &usdc_amt);
    let out = rf
        .checked_mul(usdc_amt)
        .unwrap()
        .checked_div(ru + usdc_amt)
        .unwrap();
    let (o0, o1) = if flash < usdc { (out, 0) } else { (0, out) };
    p.swap(&o0, &o1, &e.current_contract_address());
    //log!(&e, "[SWAP] usdc_to_flash {} USDC -> {} FLASH (rf {}, ru {})", usdc_amt, out, rf, ru);
    out
}

// -------------------------------------------------------------
// Contract interface
// -------------------------------------------------------------
pub trait Manager {
    fn initialize(
        e: Env,
        admin: Address,
        flash: Address,
        usdc: Address,
        initial_flash: i128,
        initial_usdc: i128,
        soroswap_factory: Address,
    ) -> Address;

    fn create_campaign(
        e: Env,
        fee_usdc: i128,
        target_pool: Address,
        unlock: u32,
        target_lp: i128,
        bonus_flash: i128,
        creator: Address,
    ) -> u32;
    fn ucnt_key(e: &Env, id: u32) -> Val;
    fn join_campaign(e: Env, id: u32, token0_amt: i128, user: Address);
    fn compound(e: Env, id: u32);
    fn claim(e: Env, id: u32, user: Address);

    fn set_surplus_bps(e: Env, admin: Address, bps: u32);
    fn set_ttl(e: Env, admin: Address, threshold: u32, bump: u32);
}

#[contract]
pub struct FlashCampaignManager;
#[contractimpl]
impl Manager for FlashCampaignManager {
    // ------- init -------
    // initialize takes the admin address, the flash address which is the token to be used as reward, the usdc address, which is the token to be used as the base against which the reward token is meant to grow in value and liquidity.  the initial flash and initial usdc are the amoounts of the two tokens to be used in creating it's initial liquidity pool.  the soroswap factory address is the address of the soroswap factory contract which will be used to create the liquidity pool for the two tokens.

    fn initialize(
        e: Env,
        admin: Address,
        flash: Address,
        usdc: Address,
        initial_flash: i128,
        initial_usdc: i128,
        soroswap_factory: Address,
    ) -> Address {
        let flash_token = TokenClient::new(&e, &flash);
        let usdc_token = TokenClient::new(&e, &usdc);
        let flash_decimals = flash_token.decimals();
        let usdc_decimals = usdc_token.decimals();
        ensure!(&e, flash_decimals == 7, FlashErr::Maa);
        ensure!(&e, usdc_decimals == 7, FlashErr::Mas);
        ensure!(&e, initial_flash > 0, FlashErr::Mad);
        ensure!(&e, initial_usdc > 0, FlashErr::Maf);
        // ensure!(&e, initial_flash <= 1_000_000, FlashErr::Mah);
        //ensure!(&e, initial_usdc <= 10_000, FlashErr::Mag);

        //std::println!("[CAMPAIGN_MANAGER_CONTRACT]\n    [INITIALIZE] -- initialize(admin {:?}, flash {:?}, usdc {:?}, initial_flash {:.7}, initial_usdc {:.7} )", admin, flash, usdc, initial_flash as f64 / flash_decimals as f64, initial_usdc as f64 / usdc_decimals as f64);
        bump(&e);
        ensure!(
            &e,
            !e.storage().instance().has(&s(&e, KEY_ADMIN)),
            FlashErr::AlreadyInit
        );

        admin.require_auth();

        // we need to make sure we are setting the storage values correctly i don't think we do right now because these are just consts...
        set_addr(&e, KEY_ADMIN, &admin);
        set_addr(&e, KEY_FLASH, &flash);
        set_addr(&e, KEY_USDC, &usdc);
        set_u32(&e, KEY_NEXT, 0);
        set_u32(&e, KEY_SURP, DEFAULT_SURPLUS_BPS);
        set_u32(&e, KEY_TTLT, DEFAULT_TTL_THRESH);
        set_u32(&e, KEY_TTLB, DEFAULT_TTL_BUMP);

        let flash_bal = flash_token.balance(&admin);
        let usdc_bal = usdc_token.balance(&admin);
        ensure!(&e, flash_bal >= initial_flash, FlashErr::Maj);
        ensure!(&e, usdc_bal >= initial_usdc, FlashErr::Mak);

        //std::println!("     BALANCES FOR {:?}\n        FLASH: {:.7}\n        USDC:  {:.7}", admin, (flash_bal / flash_decimals as i128), usdc_bal / usdc_decimals as i128);

        flash_token.transfer(&admin, &e.current_contract_address(), &flash_bal);
        usdc_token.transfer(&admin, &e.current_contract_address(), &initial_usdc);

        //log!(&e, "[INITIALIZE] pulled {} FLASH from admin", bal);

        // now we should setup the pair contract for the usdc/flash pair
        let pair_factory = soroswap_factory::Client::new(&e, &soroswap_factory);
        let t0_symbol = flash_token.symbol();
        let t1_symbol = usdc_token.symbol();
        let core_pair_address = create_pair_ordered(
            &pair_factory,
            &flash_token,
            &usdc_token,
            &t0_symbol,
            &t1_symbol,
        );
        //std::println!("     PAIR CREATED\n          {:?}", core_pair_address);
        // Store the core pair address
        set_addr(&e, KEY_CORE_PAIR, &core_pair_address);
        // Approve the pair to spend tokens
        flash_token.approve(
            &e.current_contract_address(),
            &core_pair_address,
            &initial_flash,
            &0,
        );
        usdc_token.approve(
            &e.current_contract_address(),
            &core_pair_address,
            &initial_usdc,
            &0,
        );

        // Transfer tokens to the pair
        flash_token.transfer(
            &e.current_contract_address(),
            &core_pair_address,
            &initial_flash,
        );
        usdc_token.transfer(
            &e.current_contract_address(),
            &core_pair_address,
            &initial_usdc,
        );

        // Deposit to mint LP tokens and seed the pool
        let pair = pair::Client::new(&e, &core_pair_address);
        let lp_minted = pair.deposit(&e.current_contract_address());
        ensure!(&e, lp_minted > 0, FlashErr::Mar);

        core_pair_address
        // store the core pair address in the contract storage as we will need it when creating campaigns.
    }

    // -------------------------------------- create_campaign -------
    fn create_campaign(
        e: Env,
        fee_usdc: i128,
        target_pool: Address,
        unlock: u32,
        target_lp: i128,
        bonus_flash: i128,
        creator: Address,
    ) -> u32 {
        //std::println!("── THIS IS THE CREATE CAMPAIGN LOGGING IN STD-──");

        bump(&e);
        creator.require_auth();
        let core_pair_address = get_addr(&e, KEY_CORE_PAIR);
        let core_pair = pair::Client::new(&e, &core_pair_address);

        let flash_address = get_addr(&e, KEY_FLASH);
        let usdc_address = get_addr(&e, KEY_USDC);
        let surplus_bps = get_u32(&e, KEY_SURP, DEFAULT_SURPLUS_BPS);
        ensure!(&e, surplus_bps < MAX_BPS, FlashErr::BpsOutOfRange);
        let usd_cli = TokenClient::new(&e, &usdc_address);
        //std::println!("Fee incoming: {fee_usdc} USDC from {creator:?}");

        // Take all USDC from creator

        usd_cli.transfer(&creator, &e.current_contract_address(), &fee_usdc);

        // we need to get the flash/usdc pair address from the contract storage.
        // Get pool reserves before

        let (flash_reserves_before, usdc_reserves_before) = core_pair.get_reserves();
        //std::println!("Pair reserves before: FLASH={flash_reserves_before}, USDC={usdc_reserves_before}");

        // Fee split
        let s_min = int_sqrt(
            (usdc_reserves_before as u128) * (usdc_reserves_before as u128 + fee_usdc as u128),
        ) as i128
            - usdc_reserves_before;
        let smin_log = s_min as f32 * 1e-7;
        let usdc_fee_log = fee_usdc as f32 * 1e-7;
        //std::println!("s_min={smin_log}, surplus_bps={surplus_bps}, fee_usdc={usdc_fee_log}");
        let swap_amount = (s_min
            + (fee_usdc as i128 * surplus_bps as i128) as i128 / MAX_BPS as i128)
            .min(fee_usdc);
        let add_liq_amount = fee_usdc - swap_amount;

        let swap_amount_pretty = swap_amount as f32 * 1e-7;
        let add_liq_amount_pretty = add_liq_amount as f32 * 1e-7;
        let total_amount = swap_amount + add_liq_amount;
        let total_amount_pretty = total_amount as f32 * 1e-7;
        //std::println!("Total amount: {total_amount} USDC  ||| Total amount (pretty): {total_amount_pretty} USDC");
        //std::println!("Fee split: {surplus_bps} bps, swap_amount={swap_amount_pretty} USDC, add_liq_amount={add_liq_amount_pretty} USDC");
        //std::println!("swap_amount={swap_amount_pretty}, add_liq_amount={add_liq_amount_pretty}");
        //std::println!("swap_amount={swap_amount}, add_liq_amount={add_liq_amount}");
        //std::println!("Fee split: swap {swap_amount} USDC, add_liq {add_liq_amount} USDC (s_min={s_min})");

        // Swap: USDC for FLASH
        usd_cli.transfer(
            &e.current_contract_address(),
            &core_pair_address,
            &swap_amount,
        );
        let token_0 = core_pair.token_0();
        let token_1 = core_pair.token_1();

        // Determine which is USDC and which is FLASH in the pair
        // store the usdc address in the storage key and fetch it from there to prevent a spoofer
        let (reserve_in, reserve_out, is_token0_usdc) = if token_0 == usdc_address {
            (usdc_reserves_before, flash_reserves_before, true)
        } else {
            (flash_reserves_before, usdc_reserves_before, false)
        };

        // AMM swap math with 0.3% fee (997/1000)
        let swap_amount_with_fee = swap_amount.checked_mul(997).unwrap();
        let numerator = swap_amount_with_fee.checked_mul(reserve_out).unwrap();
        let denominator = reserve_in
            .checked_mul(1000)
            .unwrap()
            .checked_add(swap_amount_with_fee)
            .unwrap();
        let flash_out = numerator.checked_div(denominator).unwrap();

        //  let flash_out = flash_reserves_before.checked_mul(swap_amount).unwrap().checked_div(usdc_reserves_before + swap_amount).unwrap();
        ensure!(&e, flash_out > 0 && flash_out < reserve_out, FlashErr::Math);

        // this should work but lets check against the pair to be sure.
        //let (swap_out_0, swap_out_1) = if flash_address < usdc_address { (flash_out, 0) } else { (0, flash_out) };
        //std::println!("Swapping swap_amount={swap_amount} for flash_out={flash_out}, out0={swap_out_0}, out1={swap_out_1}");

        let (swap_out_0, swap_out_1) = if is_token0_usdc {
            (0, flash_out) // token0 = USDC, token1 = FLASH
        } else {
            (flash_out, 0) // token0 = FLASH, token1 = USDC
        };

        core_pair.swap(&swap_out_0, &swap_out_1, &e.current_contract_address());

        // After swap
        let usdc_reserves_post_swap = usdc_reserves_before + swap_amount;
        let flash_reserves_post_swap = flash_reserves_before - flash_out;
        //std::println!("Post-swap reserves: FLASH={flash_reserves_post_swap}, USDC={usdc_reserves_post_swap}");

        // Flash needed for add_liq
        ensure!(&e, usdc_reserves_post_swap > 0, FlashErr::Math);

        let flash_needed = add_liq_amount
            .checked_mul(flash_reserves_post_swap)
            .unwrap()
            .checked_div(usdc_reserves_post_swap)
            .unwrap();

        //let flash_needed_pretty = flash_needed as f32 * 1e-7;
        //let add_liq_amount_pretty = add_liq_amount as f32 * 1e-7;
        //std::println!("Calculated flash_needed={flash_needed_pretty} for add_liq_amount={add_liq_amount_pretty} USDC (flash_reserves_post_swap={flash_reserves_post_swap}, usdc_reserves_post_swap={usdc_reserves_post_swap})");

        // Figure out which token is token_0, which is token_1
        // change this for deployment so that it doesn't need to do a fetch we can just check that token0>token1
        let token_0 = core_pair.token_0();
        let token_1 = core_pair.token_1();

        // Map the amounts to the right tokens:
        // we need to determine their order first here!
        let (amount_0, amount_1) = match (&token_0 == &flash_address, &token_1 == &usdc_address) {
            (true, true) => (flash_needed, add_liq_amount), // token_0 = FLASH, token_1 = USDC
            (false, true) => (add_liq_amount, flash_needed), // token_0 = USDC,  token_1 = FLASH
            (true, false) => (flash_needed, add_liq_amount), // token_0 = FLASH, token_1 = USDC
            (false, false) => (add_liq_amount, flash_needed), // token_0 = USDC,  token_1 = FLASH
        };

        // Transfer correct amounts to pool
        if amount_0 > 0 {
            TokenClient::new(&e, &token_0).transfer(
                &e.current_contract_address(),
                &core_pair_address,
                &amount_0,
            );
        }
        if amount_1 > 0 {
            TokenClient::new(&e, &token_1).transfer(
                &e.current_contract_address(),
                &core_pair_address,
                &amount_1,
            );
        }
        //std::println!("Transferred: token_0={token_0:?} amount_0={amount_0}, token_1={token_1:?} amount_1={amount_1}");

        // Log contract's pool-related token balances before deposit
        let t0_balance = TokenClient::new(&e, &token_0).balance(&e.current_contract_address());
        let t1_balance = TokenClient::new(&e, &token_1).balance(&e.current_contract_address());
        //std::println!("Contract balances before approve: token_0={:?}={:?}, token_1={:?}={:?}", token_0, t0_balance, token_1, t1_balance);

        // Approve for deposit
        TokenClient::new(&e, &token_0).approve(
            &e.current_contract_address(),
            &core_pair_address,
            &t0_balance,
            &0,
        );
        TokenClient::new(&e, &token_1).approve(
            &e.current_contract_address(),
            &core_pair_address,
            &t1_balance,
            &0,
        );
        // we need to do the transfer here before we call deposit! we only approve right now.

        // Log args for deposit
        //std::println!("Calling deposit: contract={:?}, to={:?}", pool_address, e.current_contract_address());

        // Deposit
        let flash_lp_minted = core_pair.deposit(&e.current_contract_address());
        ensure!(&e, flash_lp_minted > 0, FlashErr::Mal);

        //std::println!("Deposit complete. lp_minted={lp_minted}");

        // 7. reward pool calc
        let actual_usdc = TokenClient::new(&e, &usdc_address).balance(&core_pair_address)
            - usdc_reserves_post_swap;
        let actual_flash = TokenClient::new(&e, &flash_address).balance(&core_pair_address)
            - flash_reserves_post_swap;
        //std::println!("Actual deposit amounts: actual_usdc={actual_usdc}, actual_flash={actual_flash}");

        let ru1 = usdc_reserves_post_swap + actual_usdc;
        let rf1 = flash_reserves_post_swap + actual_flash;
        let root = int_sqrt(
            (ru1 as u128 * rf1 as u128 * flash_reserves_before as u128)
                / (usdc_reserves_before as u128),
        );
        let x_max = if root > rf1 as u128 {
            (root - rf1 as u128) as i128
        } else {
            0
        };

        let surplus = flash_out + actual_flash - flash_needed;
        let reward_flash = surplus.min(x_max);

        //std::println!("Reward pool calculation: ru1={ru1}, rf1={rf1}, root={root}, x_max={x_max}, surplus={surplus}, reward_flash={reward_flash}");

        //total lp is the total lp that was minted by users joining the pool, at this point it is zero.  same with stake lp... it should get updated later when users join the pool... however it can't seem to find it.  maybe we need current lp also, finally i am not sure what weight is supposed to be right now. i believe it's to do with the weight of everyone adding liquidity we'll have to figure that out later too
        let id = get_u32(&e, KEY_NEXT, 0) + 1;
        set_u32(&e, KEY_NEXT, id);
        save_camp(
            &e,
            id,
            &Campaign {
                pair: target_pool.clone(),
                duration: unlock,
                end_ledger: e.ledger().sequence() + unlock,
                target_lp,
                total_lp: 0,
                total_weight: 0,
                reward_flash,
                bonus_flash,
                stake_lp: 0,
            },
        );
        //std::println!("Campaign created: id={id}, pair={pool_address:?}, reward_flash={reward_flash}, total_lp={lp_minted}");

        id
    }

    fn ucnt_key(e: &Env, id: u32) -> Val {
        (PREFIX_UCNT, id).into_val(e)
    }

    // ------------------------------------------- join_campaign ---
    fn join_campaign(e: Env, id: u32, token0_amt: i128, user: Address) {
        bump(&e);
        user.require_auth();
        ensure!(&e, token0_amt > 0, FlashErr::Maq);

        let mut c = load_camp(&e, id);
        let pcli = pair::Client::new(&e, &c.pair);
        let t0 = pcli.token_0();
        let t1 = pcli.token_1();
        let t0cli = TokenClient::new(&e, &t0);
        let t1cli = TokenClient::new(&e, &t1);
        t0cli.transfer(&user, &e.current_contract_address(), &token0_amt);
        //log!(&e, "[JOIN CAMPAIGN] deposit {} of token0 from {:?}", token0_amt, user);

        let half = token0_amt / 2;
        t0cli.transfer(&e.current_contract_address(), &c.pair, &half);
        let (r0, r1) = pcli.get_reserves();
        let t1_out = r1
            .checked_mul(half)
            .unwrap()
            .checked_div(r0 + half)
            .unwrap();
        let (o0, o1) = if t0 < t1 { (0, t1_out) } else { (t1_out, 0) };
        pcli.swap(&o0, &o1, &e.current_contract_address());
        //log!(&e, "[JOIN CAMPAIGN] swap_half: {} token0 => {} token1", half, t1_out);

        t0cli.transfer(&e.current_contract_address(), &c.pair, &(token0_amt - half));
        t1cli.transfer(&e.current_contract_address(), &c.pair, &t1_out);

        // **APPROVE BEFORE DEPOSIT**
        let t0_bal = t0cli.balance(&e.current_contract_address());
        let t1_bal = t1cli.balance(&e.current_contract_address());
        t0cli.approve(&e.current_contract_address(), &c.pair, &t0_bal, &0);
        t1cli.approve(&e.current_contract_address(), &c.pair, &t1_bal, &0);

        let lp = pcli.deposit(&e.current_contract_address());
        ensure!(&e, lp > 0, FlashErr::Maw);

        //log!(&e, "[JOIN CAMPAIGN] minted {} LP for user {:?}", lp, user);

        let now = e.ledger().sequence();
        /*
            let w   = lp * (c.end_ledger-now) as i128 / c.duration as i128;

            c.total_lp     += lp;
            c.total_weight += w;

            save_camp(&e,id,&c);
        */
        // --- Track user count for rank ---
        let ucnt_val = e
            .storage()
            .instance()
            .get::<Val, u32>(&ucnt_key(&e, id))
            .unwrap_or(0);
        let rank = ucnt_val + 1;
        e.storage().instance().set(&ucnt_key(&e, id), &rank);

        // --- Compute weights using rewards.rs ---
        let contrib_weight = rewards::compute_contribution_weight(&e, lp, c.target_lp);
        let gamma = 2; // or configurable
        let rank_weight = rewards::compute_rank_weight(&e, rank, gamma);
        let weight = rewards::compute_score(&e, rank_weight, contrib_weight);

        c.total_lp += lp;
        c.total_weight += weight;
        save_camp(&e, id, &c);

        let key = upos_key(&e, id, &user);
        //let mut up:UserPos = e.storage().instance().get(&key).unwrap_or(UserPos{lp:0,weight:0});
        //up.lp+=lp; up.weight+=w;
        let up = UserPos {
            lp,
            weight,
            joined_ledger: now,
            rank,
        };

        e.storage().instance().set(&key, &up);

        //log!(&e, "[JOIN CAMPAIGN] join(id {}) user {:?} lp {} weight {}", id, user, lp, w);
    }

    // ------------------------------------------------------ claim --
    fn claim(e: Env, id: u32, user: Address) {
        bump(&e);
        user.require_auth();

        let c = load_camp(&e, id);
        ensure!(
            &e,
            e.ledger().sequence() >= c.end_ledger,
            FlashErr::TooEarly
        );

        let key = upos_key(&e, id, &user);
        let up: UserPos = e.storage().instance().get(&key).unwrap_optimized();
        ensure!(&e, up.weight > 0, FlashErr::NothingToClaim);

        let base = c.reward_flash * up.weight / c.total_weight;
        let bonus = if c.total_lp >= c.target_lp {
            c.bonus_flash * up.weight / c.total_weight
        } else {
            0
        };
        let total = base + bonus;

        TokenClient::new(&e, &get_addr(&e, KEY_FLASH)).transfer(
            &e.current_contract_address(),
            &user,
            &total,
        );

        TokenClient::new(&e, &c.pair).transfer(&e.current_contract_address(), &user, &up.lp);

        e.storage().instance().remove(&key);

        //log!(&e, "[CLAIM] id {:?} user {:?} flash {:?} lp {:?}", id, user, total, up.lp);
    }

    // -------------------------------------------------- compound ---
    fn compound(e: Env, id: u32) {
        bump(&e);
        let mut c = load_camp(&e, id);
        let pcli = pair::Client::new(&e, &c.pair);
        let (a0, a1) = pcli.withdraw(&e.current_contract_address());
        //log!(&e, "[COMPOUND] withdraw_from_pair token0 {} token1 {}", a0, a1);

        let t0 = pcli.token_0();
        let t1 = pcli.token_1();
        TokenClient::new(&e, &t0).transfer(&e.current_contract_address(), &c.pair, &a0);
        TokenClient::new(&e, &t1).transfer(&e.current_contract_address(), &c.pair, &a1);

        // **APPROVE BEFORE DEPOSIT**
        let t0_bal = TokenClient::new(&e, &t0).balance(&e.current_contract_address());
        let t1_bal = TokenClient::new(&e, &t1).balance(&e.current_contract_address());
        TokenClient::new(&e, &t0).approve(&e.current_contract_address(), &c.pair, &t0_bal, &0);
        TokenClient::new(&e, &t1).approve(&e.current_contract_address(), &c.pair, &t1_bal, &0);

        let lp_new = pcli.deposit(&e.current_contract_address());
        ensure!(&e, lp_new > 0, FlashErr::Mae);
        //log!(&e, "[COMPOUND] re-deposit minted {} LP", lp_new);

        let mut gain = 0i128;
        let fee_lp = lp_new - c.stake_lp;
        if fee_lp > 0 {
            TokenClient::new(&e, &c.pair).transfer(&e.current_contract_address(), &c.pair, &fee_lp);
            let (f0, f1) = pcli.withdraw(&e.current_contract_address());
            let flash = get_addr(&e, KEY_FLASH);
            let usdc = get_addr(&e, KEY_USDC);
            if t0 == flash {
                gain += f0
            } else if t0 == usdc {
                gain += swap_usdc_to_flash(&e, &c.pair, f0, flash.clone(), usdc.clone())
            }
            if t1 == flash {
                gain += f1
            } else if t1 == usdc {
                gain += swap_usdc_to_flash(&e, &c.pair, f1, flash, usdc)
            }
            c.reward_flash += gain;
            //log!(&e, "[COMPOUND] performance_fee +{} FLASH into pool", gain);
        }
        c.stake_lp = lp_new;
        save_camp(&e, id, &c);

        //log!(&e, "[COMPOUND] compound(id {}) fee_lp {} gain {}", id, fee_lp, gain);
    }

    // ------------------------------------------- admin helpers ----
    fn set_surplus_bps(e: Env, admin: Address, bps: u32) {
        admin.require_auth();
        ensure!(&e, admin == get_addr(&e, KEY_ADMIN), FlashErr::NotAdmin);
        ensure!(&e, bps < MAX_BPS, FlashErr::BpsOutOfRange);
        set_u32(&e, KEY_SURP, bps);
        //log!(&e, "[ADMIN] surplus_bps set to {}", bps);
    }

    fn set_ttl(e: Env, admin: Address, threshold: u32, bump_: u32) {
        admin.require_auth();
        ensure!(&e, admin == get_addr(&e, KEY_ADMIN), FlashErr::NotAdmin);
        set_u32(&e, KEY_TTLT, threshold);
        set_u32(&e, KEY_TTLB, bump_);
        //log!(&e, "[ADMIN] ttl threshold {} bump {}", threshold, bump_);
    }
}

// -------------------------------------------------------------
#[cfg(test)]
mod test;
