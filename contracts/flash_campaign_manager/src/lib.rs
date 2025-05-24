//! flash_campaign_manager
//! -------------------------------------------------------------
//! Verbose build – every major step emits a plain-text log.

#![no_std]

// -------------------------------------------------------------
// Imports
//
use soroban_sdk::{
    contract,
    contracterror,
    contractimpl,
    contracttype,
    token::Client as TokenClient,
    unwrap::UnwrapOptimized,
    Address,
    Env,
    IntoVal,
    // String, Symbol, TryIntoVal, BytesN, token, token::StellarAssetClient, log as _log
    Val,
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
mod utils;
use storage::*;
use utils::*;

#[cfg(all(not(target_family = "wasm")))]
extern crate std;

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
    InvalidToken = 20,
    CampaignActiveForPair = 21,
    NoCorePair = 22,
}

/// assert-style helper that logs **before** panicking
macro_rules! ensure {
    ($env:expr, $cond:expr, $err:expr) => {
        if !$cond {
            #[cfg(all(not(target_family = "wasm")))]
            std::println!(
                "[CAMPAIGN_MANAGER_CONTRACT]\n    [ensure!] -- failed -- code {}",
                $err as u32
            );
            //log!($env, "ensure! failed – code {}", $err as u32);
            //panic_with_error!($env, $err)
            // return an Err instead of panic!
            return Err($err);
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
    #[cfg(all(not(target_family = "wasm")))]
    std::println!("[CAMPAIGN_MANAGER_CONTRACT]\n    [bump] -- trying to bump ttl");
    let config = get_core_config(e);
    e.storage()
        .instance()
        .extend_ttl(config.ttl_thresh, config.ttl_bump);
}

#[allow(dead_code)]
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
    ) -> Result<Address, FlashErr>;

    fn create_campaign(
        e: Env,
        fee_usdc: i128,
        target_pool: Address,
        unlock: u32,
        target_lp: i128,
        bonus_flash: i128,
        creator: Address,
    ) -> Result<u32, FlashErr>;
    fn ucnt_key(e: &Env, id: u32) -> Result<Val, FlashErr>;
    fn join_campaign(e: Env, id: u32, token0_amt: i128, user: Address) -> Result<(), FlashErr>;
    fn compound(e: Env, id: u32) -> Result<(), FlashErr>;
    fn claim(e: Env, id: u32, user: Address) -> Result<(), FlashErr>;

    fn set_surplus_bps(e: Env, admin: Address, bps: u32) -> Result<(), FlashErr>;
    fn set_ttl(e: Env, admin: Address, threshold: u32, bump: u32) -> Result<(), FlashErr>;
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
    ) -> Result<Address, FlashErr> {
        let flash_token = TokenClient::new(&e, &flash);
        let usdc_token = TokenClient::new(&e, &usdc);

        // THIS DOESN'T NEED TO BE IN THE FINAL CONTRACT
        let flash_decimals = flash_token.decimals();
        let _flash_scale = 10i128.pow(flash_decimals as u32);
        let usdc_decimals = usdc_token.decimals();
        let _usdc_scale = 10i128.pow(usdc_decimals as u32);

        //ensure!(&e, flash_decimals == 7, FlashErr::Maa);
        //ensure!(&e, usdc_decimals == 7, FlashErr::Mas);
        ensure!(&e, initial_flash > 0, FlashErr::Mad);
        ensure!(&e, initial_usdc > 0, FlashErr::Maf);
        // ensure!(&e, initial_flash <= 1_000_000, FlashErr::Mah);
        //ensure!(&e, initial_usdc <= 10_000, FlashErr::Mag);
        #[cfg(all(not(target_family = "wasm")))]
        std::println!("[CAMPAIGN_MANAGER_CONTRACT]\n    [INITIALIZE] -- initialize(admin {:?}, flash {:?}, usdc {:?}, initial_flash {:.7}, initial_usdc {:.7} )", admin, flash, usdc, initial_flash as f64 / _flash_scale as f64, initial_usdc as f64 / _usdc_scale as f64);

        //bump(&e);

        ensure!(
            &e,
            !e.storage().instance().has(&KEY_CORE_CONFIG),
            FlashErr::AlreadyInit
        );

        admin.require_auth();

        let flash_bal_a = flash_token.balance(&admin);
        let usdc_bal_a = usdc_token.balance(&admin);
        ensure!(&e, flash_bal_a >= initial_flash, FlashErr::Maj);
        ensure!(&e, usdc_bal_a >= initial_usdc, FlashErr::Mak);
        #[cfg(all(not(target_family = "wasm")))]
        std::println!(
            "     Admin BALANCES FOR {:?}\n        FLASH: {:.7}\n        USDC:  {:.7}",
            admin,
            (flash_bal_a / _flash_scale) as f64,
            (usdc_bal_a / _usdc_scale) as f64
        );
        #[cfg(all(not(target_family = "wasm")))]
        {
            let flash_bal_c = flash_token.balance(&e.current_contract_address());
            let usdc_bal_c = usdc_token.balance(&e.current_contract_address());
            std::println!(
                "      1 contract BALANCES FOR {:?}\n        FLASH: {:.7}\n        USDC:  {:.7}",
                e.current_contract_address(),
                (flash_bal_c / _flash_scale) as f64,
                (usdc_bal_c / _usdc_scale) as f64
            );
        }
        // transfer from admin to manager contract
        flash_token.transfer(&admin, &e.current_contract_address(), &flash_bal_a);
        usdc_token.transfer(&admin, &e.current_contract_address(), &initial_usdc);
        #[cfg(all(not(target_family = "wasm")))]
        {
            let flash_bal_c1 = flash_token.balance(&e.current_contract_address());
            let usdc_bal_c1 = usdc_token.balance(&e.current_contract_address());
            std::println!(
        "      2 contract BALANCES FOR {:?} AFTER\n        FLASH: {:.7}\n        USDC:  {:.7}",
        e.current_contract_address(),
        (flash_bal_c1 / _flash_scale) as f64,
        (usdc_bal_c1 / _usdc_scale) as f64
    );
            let flash_bal_a2 = flash_token.balance(&admin);
            let usdc_bal_a2 = usdc_token.balance(&admin);
            std::println!(
                "     2 Admin BALANCES FOR {:?} AFTER\n        FLASH: {:.7}\n        USDC:  {:.7}",
                admin,
                (flash_bal_a2 / _flash_scale) as f64,
                (usdc_bal_a2 / _usdc_scale) as f64
            );
        }
        //log!(&e, "[INITIALIZE] pulled {} FLASH from admin", bal);

        // now we should setup the pair contract for the usdc/flash pair
        let pair_factory = soroswap_factory::Client::new(&e, &soroswap_factory);

        let core_pair_address = c_p(&e, &pair_factory, &flash, &usdc);
        #[cfg(all(not(target_family = "wasm")))]
        std::println!("     PAIR CREATED\n          {:?}", core_pair_address);

        // Store the core pair address
        //set_addr(&e, KEY_CORE_PAIR, &core_pair_address);

        let lp = d_t_p(
            &e,
            &core_pair_address,
            &flash,
            &usdc,
            initial_flash,
            initial_usdc,
        );

        #[cfg(all(not(target_family = "wasm")))]
        {
            use crate::pair::Client as PC;

            let flash_bal_c3 = flash_token.balance(&e.current_contract_address());
            let usdc_bal_c3 = usdc_token.balance(&e.current_contract_address());
            std::println!(
            "     3 contract BALANCES FOR {:?}\n        FLASH: {:.7}\n        USDC:  {:.7} AFTER lp deposit",
            e.current_contract_address(),
            (flash_bal_c3 / _flash_scale) as f64 ,
            (usdc_bal_c3 / _usdc_scale) as f64
        );
            let pc = PC::new(&e, &core_pair_address);
            let lp_balance = pc.balance(&e.current_contract_address());
            std::println!(
                "     3 contract BALANCES FOR {:?}\n        LP: {:.7}",
                e.current_contract_address(),
                (lp_balance / _flash_scale) as f64
            );
        }
        #[cfg(all(not(target_family = "wasm")))]
        {
            let flash_bal_core = flash_token.balance(&core_pair_address);
            let usdc_bal_core = usdc_token.balance(&core_pair_address);
            std::println!(
                "     3 pair BALANCES FOR {:?}\n        FLASH: {:.7}\n        USDC:  {:.7}",
                e.current_contract_address(),
                (flash_bal_core / _flash_scale) as f64,
                (usdc_bal_core / _usdc_scale) as f64
            );
        }
        // do we need to store the lp of flash amount i'm not sure yet
        ensure!(&e, lp > 0, FlashErr::Mar);

        // Store all config in a single struct
        let config = CoreConfig {
            admin: admin.clone(),
            flash: flash.clone(),
            usdc: usdc.clone(),
            core_pair: Some(core_pair_address.clone()),
            next: 0,
            surplus_bps: DEFAULT_SURPLUS_BPS,
            ttl_thresh: DEFAULT_TTL_THRESH,
            ttl_bump: DEFAULT_TTL_BUMP,
        };

        set_core_config(&e, &config);
        bump(&e);
        #[cfg(all(not(target_family = "wasm")))]
        {
            let stored_config = get_core_config(&e);
            std::println!("[initialize] CoreConfig stored: {:?}", stored_config);
        }
        Ok(core_pair_address)
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
    ) -> Result<u32, FlashErr> {
        
        #[cfg(all(not(target_family = "wasm")))]
        std::println!("── [FCM - CREATE_CAMPAIGN] ──");
        let cca = &e.current_contract_address();
        if let Some(info) = get_active_campaign_for_pair(&e, &target_pool) {
            ensure!(
                &e,
                info.end_ledger <= e.ledger().sequence(),
                FlashErr::CampaignActiveForPair
            );
        }
        bump(&e);
        creator.require_auth();
        // Load config struct
        let mut config = get_core_config(&e);

        let core_pair_address = config
            .core_pair
            .clone()
           .ok_or(FlashErr::NoCorePair)?;
        let core_pair = pair::Client::new(&e, &core_pair_address);

        let flash_address = config.flash.clone();
        let usdc_address = config.usdc.clone();
        let surplus_bps = config.surplus_bps;
        ensure!(&e, surplus_bps < MAX_BPS, FlashErr::BpsOutOfRange);
        let balance_flash_start = TokenClient::new(&e, &flash_address).balance(&cca);
        let pair_balance_start = TokenClient::new(&e, &flash_address).balance(&core_pair_address);
        let usd_cli = TokenClient::new(&e, &usdc_address);
        #[cfg(all(not(target_family = "wasm")))]
        std::println!(
            "    {creator:?} creating campaign with {:.7} USDC",
            (fee_usdc / 10i128.pow(7)) as f64
        );

        // Take all USDC from creator
        usd_cli.transfer(&creator, &cca, &fee_usdc);

        let (r0, r1) = core_pair.get_reserves();
        let (t0, t1) = o_t_c(&e, &flash_address, &usdc_address);
        let t0_addr = t0.address.clone();
        let t1_addr = t1.address.clone();
        #[cfg(all(not(target_family = "wasm")))]
        std::println!(
            "    Raw pair reserves: {:?}={:.7}, {:?}={:.7}",
            t0_addr, (r0 as f64) * 1e-7,
            t1_addr, (r1 as f64) * 1e-7
        );
        // Map reserves so reserve_usdc_before / reserve_flash_before are correct
        let (reserve_usdc_before, reserve_flash_before) =
            if t0_addr == usdc_address && t1_addr == flash_address {
                (r0, r1)
            } else if t0_addr == flash_address && t1_addr == usdc_address {
                (r1, r0)
            } else {
                return Err(FlashErr::InvalidToken);
            };
#[cfg(all(not(target_family = "wasm")))]
        std::println!(
            "    Pair reserves before (USDC→FLASH):\n        USDC={:.7}\n        FLASH={:.7}",
            (reserve_usdc_before as f64) * 1e-7,
            (reserve_flash_before as f64) * 1e-7
        );

        
        // Fee split
        let s_min = int_sqrt(
            (reserve_usdc_before as u128) * (reserve_usdc_before as u128 + fee_usdc as u128),
        ) as i128
            - reserve_usdc_before;

        #[cfg(all(not(target_family = "wasm")))]
        {
            let smin_log = s_min as f64 * 1e-7;
            let usdc_fee_log = fee_usdc as f64 * 1e-7;
            std::println!(
                "    s_min={smin_log}, surplus_bps={surplus_bps}, fee_usdc={usdc_fee_log}"
            );
        }
        let swap_amount = (s_min
            + (fee_usdc as i128 * surplus_bps as i128) as i128 / MAX_BPS as i128)
            .min(fee_usdc);
        let usdc_liq = fee_usdc - swap_amount;
        #[cfg(not(target_family = "wasm"))] {
            let total_amt = swap_amount + usdc_liq;
            std::println!(
                "    Fee Split Calculation:\n        fee_usdc={:.7}\n        TotalAmount={:.7}\n        s_min={:.7}\n        surplus_bps={}\n        swap_amount={:.7} USDC\n        usdc_liq={:.7} USDC",
                (fee_usdc as f64)   * 1e-7,
                (total_amt as f64)  * 1e-7,
                (s_min as f64)      * 1e-7,
                surplus_bps,
                (swap_amount as f64)* 1e-7,
                (usdc_liq as f64)   * 1e-7
            );
        }
                
        let is_t0_usdc = t0_addr == usdc_address;
        // swap: send USDC_in then instruct pair.swap
        usd_cli.transfer(&cca, &core_pair_address, &swap_amount);
        
         let flash_out = c_s_o1(swap_amount, reserve_usdc_before, reserve_flash_before);

        //let flash_out = c_s_o(swap_amount, usdc_reserves_before, flash_reserves_before);

        ensure!(&e, flash_out > 0 && flash_out < reserve_flash_before, FlashErr::Math);

        //todo: make this more efficient, see utils::line5
        let (out_0, out_1) = if is_t0_usdc {
            (0, flash_out) // token0 = USDC, token1 = FLASH
        } else {
            (flash_out, 0) // token0 = FLASH, token1 = USDC
        };
        #[cfg(all(not(target_family = "wasm")))]
        std::println!("    Swapping\n        swap_amount={:.7} USDC\n        flash_out={:.7}\n        out0={out_0}\n        out1={out_1}", (swap_amount as f64 * 1e-7) as f64, (flash_out as f64 * 1e-7) as f64);

        core_pair.swap(&out_0, &out_1, &e.current_contract_address());

        // After swap
        let usdc_reserves_post_swap = reserve_usdc_before + swap_amount;
        let flash_reserves_post_swap = reserve_flash_before - flash_out;
        #[cfg(all(not(target_family = "wasm")))]
        {
        let (pr1, pr0) = core_pair.get_reserves();
        std::println!(
            "Post-swap reserves: \n   FLASH={:.7}\n    USDC={:.7}\n   pr0={:.7}\n   pr1={:.7}", flash_reserves_post_swap as f64 * 1e-7, usdc_reserves_post_swap as f64 * 1e-7, pr0 as f64 * 1e-7, pr1 as f64 * 1e-7
        );

        }
       
        // Flash needed for add_liq
        ensure!(&e, usdc_reserves_post_swap > 0, FlashErr::Math);

        let flash_needed = usdc_liq
            .checked_mul(flash_reserves_post_swap)
            .unwrap()
            .checked_div(usdc_reserves_post_swap)
            .unwrap();

        #[cfg(all(not(target_family = "wasm")))]
        {
            let flash_needed_pretty = flash_needed as f64 * 1e-7;

            let usdc_liq_amount_pretty = usdc_liq as f64 * 1e-7;
            std::println!("Calculated flash_needed={flash_needed_pretty} for usdc_liq={usdc_liq_amount_pretty} USDC (flash_reserves_post_swap={flash_reserves_post_swap}, usdc_reserves_post_swap={usdc_reserves_post_swap})");

            let (a0, a1) = if is_t0_usdc {
                (flash_needed, usdc_liq)
            } else {
                (usdc_liq, flash_needed)
            };

            std::println!(
                "Transferred: token_0={:?} amount_0={:.7}, token_1={:?} amount_1={:.7}",
                t0_addr,
                (a0 as f64 * 1e-7) as f64,
                t1_addr,
                (a1 as f64 * 1e-7) as f64
            );

            // Log contract's pool-related token balances before deposit
            let t0_balance = t0.balance(&e.current_contract_address());
            let t1_balance = t1.balance(&e.current_contract_address());
            std::println!(
                "Contract balances before xfer: \n    token_0={:?}={:.7}\n    token_1={:?}={:.7}",
                t0_addr,
                (t0_balance as f64 * 1e-7) as f64,
                t1_addr,
                (t1_balance as f64 * 1e-7) as f64
            );
        }

        let lpm = d_t_p(
            &e,
            &core_pair_address,
            &flash_address,
            &usdc_address,
            flash_needed,
            usdc_liq,
        );
        ensure!(&e, lpm > 0, FlashErr::Math);
        #[cfg(all(not(target_family = "wasm")))]
        std::println!("Deposit complete. flash/usdc lp minted={:.7}", lpm as f64 * 1e-7);

        // 7. reward pool calc
        let actual_usdc = TokenClient::new(&e, &usdc_address).balance(&core_pair_address)
            - usdc_reserves_post_swap;
        let actual_flash = TokenClient::new(&e, &flash_address).balance(&core_pair_address)
            - flash_reserves_post_swap;
        #[cfg(all(not(target_family = "wasm")))]
        std::println!(
            "Actual deposit amounts:\n    actual_flsh={:.7}\n    actual_usdc={:.7}",
            (actual_flash as f64 * 1e-7) as f64,
            (actual_usdc as f64 * 1e-7) as f64
        );
        // compare and verify reserves and balances and math is correct below.
        let ru1 = usdc_reserves_post_swap + actual_usdc;
        let rf1 = flash_reserves_post_swap + actual_flash;
        let root = int_sqrt(
            (ru1 as u128 * rf1 as u128 * reserve_flash_before as u128)
                / (reserve_usdc_before as u128),
        );
        let x_max = if root > rf1 as u128 {
            (root - rf1 as u128) as i128
        } else {
            0
        };

        let surplus = flash_out + actual_flash - flash_needed;
        let reward_flash = surplus.min(x_max);
        #[cfg(all(not(target_family = "wasm")))]{
        std::println!("Reward pool calculation: ru1={ru1}, rf1={rf1}, root={root}, x_max={x_max}, surplus={surplus}, reward_flash={reward_flash}");
        std::println!(
            "Reward pool calculation:\n    ru1={:.7}\n    rf1={:.7}\n    root={:.7}\n    x_max={:.7}\n    surplus={:.7}\n    reward_flash={:.7}",
            (ru1 as f64 * 1e-7) as f64,
            (rf1 as f64 * 1e-7) as f64,
            (root as f64 * 1e-7) as f64,
            (x_max as f64 * 1e-7) as f64,
            (surplus as f64 * 1e-7) as f64,
            (reward_flash as f64 * 1e-7) as f64
        );
        }
        //total lp is the total lp that was minted by users joining the pool, at this point it is zero.  same with stake lp... it should get updated later when users join the pool... however it can't seem to find it.  maybe we need current lp also, finally i am not sure what weight is supposed to be right now. i believe it's to do with the weight of everyone adding liquidity we'll have to figure that out later too
        let id = config.next + 1;
        config.next = id;
        set_core_config(&e, &config);

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
        let info = ActiveCampaignInfo {
            campaign_id: id,
            end_ledger: e.ledger().sequence() + unlock,
        };
        set_active_campaign_for_pair(&e, &target_pool, &info);

        #[cfg(all(not(target_family = "wasm")))]
        {
            std::println!("Campaign created:\n    id={id}\n    target_lp={target_lp}\n   pair={target_pool:?}\n    reward_flash={reward_flash}\n    bonus_flash={bonus_flash}");
            let camp = load_camp(&e, id);
            std::println!(
        "[create_campaign] Campaign from storage:\n{id}\n    pair={:?}\n    duration={}\n    end_ledger={}\n    target_lp={}\n    total_lp={:.7} \n    total_weight={}\n    reward_flash={:.7}\n    bonus_flash={:.7}\n    stake_lp={:.7}",
        camp.pair, camp.duration, camp.end_ledger, camp.target_lp, camp.total_lp, camp.total_weight as f64, camp.reward_flash as f64 * 1e-7, camp.bonus_flash as f64*1e-7, camp.stake_lp as f64 * 1e-7
    );
    let contract_bal = TokenClient::new(&e, &flash_address).balance(&e.current_contract_address());
    let pair_bal = TokenClient::new(&e, &flash_address).balance(&core_pair_address);
    let contractcirculatingbalance = (balance_flash_start - contract_bal) as f64 * 1e-7;
    let pairbalanceafter = (pair_balance_start - pair_bal) as f64 * 1e-7;
            std::println!(
                "[create_campaign] CoreConfig stored:,\n    balance_flash_start{:7}\n    contract_bal: {:.7}\n    pair_balance_start: {:.7}\n    pair_bal: {:.7}\n    pair after: {:.7}\n    cont after {:.7}", balance_flash_start, contract_bal, pair_balance_start, pair_bal, pairbalanceafter, contractcirculatingbalance);
        }
        Ok(id)
    }

    fn ucnt_key(e: &Env, id: u32) -> Result<Val, FlashErr> {
        Ok((PREFIX_UCNT, id).into_val(e))
    }

    // ------------------------------------------- join_campaign ---
    fn join_campaign(e: Env, id: u32, token0_amt: i128, user: Address) -> Result<(), FlashErr> {
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
        Ok(())
        //log!(&e, "[JOIN CAMPAIGN] join(id {}) user {:?} lp {} weight {}", id, user, lp, w);
    }

    // ------------------------------------------------------ claim --
    fn claim(e: Env, id: u32, user: Address) -> Result<(), FlashErr> {
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
        let config = get_core_config(&e);
        TokenClient::new(&e, &config.flash).transfer(&e.current_contract_address(), &user, &total);

        TokenClient::new(&e, &c.pair).transfer(&e.current_contract_address(), &user, &up.lp);

        e.storage().instance().remove(&key);
        Ok(())
        //log!(&e, "[CLAIM] id {:?} user {:?} flash {:?} lp {:?}", id, user, total, up.lp);
    }

    // -------------------------------------------------- compound ---
    fn compound(e: Env, id: u32) -> Result<(), FlashErr> {
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
            let config = get_core_config(&e);
            let flash = config.flash.clone();
            let usdc = config.usdc.clone();
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
        Ok(())
        //log!(&e, "[COMPOUND] compound(id {}) fee_lp {} gain {}", id, fee_lp, gain);
    }

    // ------------------------------------------- admin helpers ----
    fn set_surplus_bps(e: Env, admin: Address, bps: u32) -> Result<(), FlashErr> {
        admin.require_auth();
        let mut config = get_core_config(&e);
        ensure!(&e, admin == config.admin, FlashErr::NotAdmin);
        ensure!(&e, bps < MAX_BPS, FlashErr::BpsOutOfRange);
        config.surplus_bps = bps;
        set_core_config(&e, &config);
        Ok(())
        //log!(&e, "[ADMIN] surplus_bps set to {}", bps);
    }

    fn set_ttl(e: Env, admin: Address, threshold: u32, bump_: u32) -> Result<(), FlashErr> {
        admin.require_auth();
        let mut config = get_core_config(&e);
        ensure!(&e, admin == config.admin, FlashErr::NotAdmin);
        config.ttl_thresh = threshold;
        config.ttl_bump = bump_;
        set_core_config(&e, &config);
        Ok(())
        //log!(&e, "[ADMIN] ttl threshold {} bump {}", threshold, bump_);
    }
}

// -------------------------------------------------------------
#[cfg(test)]
pub mod test;
#[cfg(test)]
pub mod tests {
    pub mod log;
    pub mod pair;
    pub mod token;
    pub mod utils;
}
