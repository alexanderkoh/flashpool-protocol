#![cfg(test)]
use std::format;
use std::vec::Vec;

#[allow(unused_imports)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, Logs, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal, String as SorobanString, Val, Vec,
};

extern crate std;



pub mod campaign_manager {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/flash_campaign_manager.wasm");
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
// use soroswap_factory::SoroswapFactoryClient;

// For tests: Use 7 decimals for tokens!
pub const DECIMALS: u32 = 7;
pub const TOKEN_UNIT: i128 = 10i128.pow(DECIMALS);

fn create_pair_ordered<'a>(
    factory: &soroswap_factory::Client<'a>,
    token_a: &token::Client<'a>,
    token_b: &token::Client<'a>,
) -> Address {
    let pairaddress: Address;
    std::println!("[CREATE_PAIR_ORDERED] --- CREATING PAIR FOR {}/{} ---", label_a, label_b);
    if token_a.address < token_b.address {
        pairaddress = factory.create_pair(&token_a.address, &token_b.address);
        log_pair_creation(
            &pairaddress,
            label_a,
            &token_a.address,
            label_b,
            &token_b.address,
        );
    } else {
        pairaddress = factory.create_pair(&token_b.address, &token_a.address);
        log_pair_creation(
            &pairaddress,
            label_b,
            &token_b.address,
            label_a,
            &token_a.address,
        );
    }
    pairaddress
}

pub fn fresh_env<'a>(
    caller: &str,
) -> (
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
    std::println!(
        "\n── [FRESH_ENV] - tests - setup requested by {} ──",
        caller
    );

    let e = Env::default();
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    let god = Address::generate(&e);
    let god1 = generate_account_address(&e, 42);
let user_str = god1.to_string();
let user_std_str = soroban_string_to_std(&user_str);
std::println!("Generated user address: {}", user_std_str);
    let (my_token, my_token_admin) = register_custom_stellar_asset_contract(&e, &god1, "TEST");
    std::println!(
        "Created asset contract for user {} with address {:?} with name: {:?} and symbol {:?}",
        user_std_str,
        my_token.address,
        my_token.name(),
        my_token.symbol()
    );

    let contract_id = e.register(FlashCampaignManager, ());
    let contract_addr = contract_id.clone(); // This is the contract's address
    // set_sac_admin_to_contract(&e, &god1, &my_token_admin, &contract_addr);

    
    let mut users = Vec::new(&e);
    for _ in 0..10 {
        let user = Address::generate(&e);
        users.push_back(user.clone());
    }

    let (flash, flash_admin) = create_token(&e, &god);
    let (usdc, usdc_admin) = create_token(&e, &god);
    let (eurc, eurc_admin) = create_token(&e, &god);
    let flashsymbol = flash.symbol();
    let flashname = flash.name();
    let usdcsymbol = usdc.symbol();
    let usdcname = usdc.name();
    std::println!(
        "[FRESH_ENV]\ntokens created: \n {:?} {:?} {:?} \n {:?} {:?} {:?}",
        flash.address, flashsymbol, flashname, usdc.address, usdcsymbol, usdcname
    );
    // Mint with full decimals
    flash_admin.mint(&god, &(10_000_000 * TOKEN_UNIT));
    usdc_admin.mint(&god, &(1_250_000 * TOKEN_UNIT));
    eurc_admin.mint(&god, &(1_000_000 * TOKEN_UNIT));

    // Give each user proportional USDC/EURC (simplified for now)
    for (i, user) in users.iter().enumerate() {
        usdc_admin.mint(&user, &(12_500 * TOKEN_UNIT));
        eurc_admin.mint(&user, &(10_000 * TOKEN_UNIT));

        // Shorten address for log
        let user_str = user.to_string();
        let user_std_str = soroban_string_to_std(&user_str);
        let short = &user_std_str[user_std_str.len().saturating_sub(4)..];
        std::println!(
            "[minting tokens] user{}=...{} usdc={} eurc={}",
            i,
            short,
            usdc.balance(&user) / TOKEN_UNIT,
            eurc.balance(&user) / TOKEN_UNIT
        );
    }

    // Deploy factory and pairs
    let pair_wasm_hash = e.deployer().upload_contract_wasm(soroswap_pair::WASM);
    
    let factory_addr = e.register(soroswap_factory::WASM, ());
    let factory = SoroswapFactoryClient::new(&e, &factory_addr);
    factory.initialize(&god, &pair_wasm_hash);
        
    let flash_usdc_pair = create_pair_ordered(&factory, &flash, &usdc);
    let usdc_eurc_pair = create_pair_ordered(&factory, &usdc, &eurc);
    std::println!("[FRESH_ENV] ---- CREATING PAIRS ----");
    std::println!("     PAIR CREATED\n          {:?}", flash_usdc_pair);
    std::println!("             token0: FLASH\n             Address:    {:?}", flash.address);
    std::println!("             token1: USDC\n             Address:    {:?}", usdc.address);
    std::println!("     PAIR CREATED\n          {:?}", usdc_eurc_pair);
    std::println!("             token0: USDC\n             Address:    {:?}", usdc.address);
    std::println!("             token1: EURC\n             Address:    {:?}", eurc.address);

    setup_and_log_pair_liquidity(
        &e, &god, &flash, &usdc, &flash_usdc_pair, "FLASH/USDC",
        1_000_000 * TOKEN_UNIT, 500_000 * TOKEN_UNIT
    );

    setup_and_log_pair_liquidity(
        &e, &god, &usdc, &eurc, &usdc_eurc_pair, "USDC/EURC",
        250_000 * TOKEN_UNIT, 312_500 * TOKEN_UNIT
    );

//    let mgr_id = e.register(FlashCampaignManager, ());
  //  let mgr = FlashCampaignManagerClient::new(&e, &mgr_id);
    let manager_address = e.register(campaign_manager::WASM, ());
    let manager = campaign_manager::FlashCampaignManagerClient::new(&e, &manager_address);
    
    let initial_flash: i128 = 100_000 * TOKEN_UNIT;
    let initial_usdc: i128 = 250 * TOKEN_UNIT;
    std::println!("[TESTS - FRESH_ENV]\n    [INITIALIZE]\n        initializing the manager contract at address {:?} with initial flash={} and usdc={}", manager_address, initial_flash / TOKEN_UNIT, initial_usdc / TOKEN_UNIT);
    manager.initialize(&god, &flash.address, &usdc.address, &initial_flash, &initial_usdc, &factory_addr);
    std::println!("[TESTS - FRESH_ENV]\n    [INITIALIZE]\n        flash_usdc_pair={:?}", flash_usdc_pair);
    (
        e,
        manager,
        god,
        users,
        flash,
        usdc,
        eurc,
        factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
    )
}

fn setup_and_log_pair_liquidity<'a>(
    e: &Env,
    god: &Address,
    token0: &token::Client<'a>,
    token0_label: &str,
    token1: &token::Client<'a>,
    token1_label: &str,
    pair_addr: &Address,
    deposit0: i128,
    deposit1: i128,
) {
    let pair_client = SoroswapPairClient::new(e, pair_addr);
    // Order tokens and labels to match the pair contract's logic
    let (token_a, label_a, token_b, label_b, deposit_a, deposit_b) = order_tokens_and_labels(token0, token0_label, token1, token1_label, deposit0, deposit1);
    std::println!("[SETUP PAIR LIQUIDITY] --- {:?} {t0}/{t1} ---\n \n     Depositing:\n            {t0}:{:?} -     {:.7}\n            {t1}:{:?}      {:.7}",
        pair_addr,
        format!("{:?}", token_a.address.to_string()),
        deposit_a as f64 / TOKEN_UNIT as f64,
        token_b.address.to_string(),
        deposit_b as f64 / TOKEN_UNIT as f64,
        t0=label_a,
        t1=label_b
    );
    let (r0, r1) = pair_client.get_reserves();

    std::println!(
        "[FRESH_ENV] --- SETUP PAIRS --- \n[{t0}/{t1}]\n     BALANCE DIAG:\n        BEFORE TRANSFER\n            via token.balance($pairaddress):\n                {t0}:        {:.7}\n                {t1}:        {:.7}\n            via get_reserves:\n                {t0}:        {:.7}\n                {t1}:        {:.7}",
        token_a.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        token_b.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        r0 as f64 / TOKEN_UNIT as f64,
        r1 as f64 / TOKEN_UNIT as f64,
        t0=label_a,
        t1=label_b
    );
    std::println!("    User god address: {:?}\n    balances:\n        token0: {:.7}\n        token1: {:.7}", god, token_a.balance(god) as f64 / TOKEN_UNIT as f64, token_b.balance(god) as f64 / TOKEN_UNIT as f64);

    token_a.approve(god, pair_addr, &deposit_a, &0);
    token_a.transfer(god, pair_addr, &deposit_a);

    token_b.approve(god, pair_addr, &deposit_a, &0);
    token_b.transfer(god, pair_addr, &deposit_b);

    let (r0, r1) = pair_client.get_reserves();
    std::println!(
        "        AFTER TRANSFER (BEFORE DEPOSIT)\n            via token.balance($pairaddress):\n                {t0}:        {:.7}\n                {t1}:        {:.7}\n            via get_reserves:\n                {t0}:        {:.7}\n                {t1}:        {:.7}",
        token_a.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        token_b.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        r0 as f64 / TOKEN_UNIT as f64,
        r1 as f64 / TOKEN_UNIT as f64,
        t0=label_a,
        t1=label_b
    );

    pair_client.deposit(god);
    let (r0, r1) = pair_client.get_reserves();
    std::println!(
        "        AFTER DEPOSIT\n            via token.balance($pairaddress):\n                {t0}:        {:.7}\n                {t1}:        {:.7}\n            via get_reserves:\n                {t0}:        {:.7}\n                {t1}:        {:.7}",
        token_a.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        token_b.balance(pair_addr) as f64 / TOKEN_UNIT as f64,
        r0 as f64 / TOKEN_UNIT as f64,
        r1 as f64 / TOKEN_UNIT as f64,
        t0=label_a,
        t1=label_b
    );
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
use soroban_sdk::{Env, Address};
use soroban_sdk::xdr::{Asset, AssetCode4, AlphaNum4, WriteXdr};

fn generate_account_address(e: &Env, seed: u8) -> Address {
    use soroban_sdk::xdr::{AccountId, PublicKey, Uint256, ScAddress};
    let mut pk = [0u8; 32];
    pk[0] = seed; // Use seed to make addresses unique for each user
    let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(pk)));
    let sc_addr = ScAddress::Account(account_id);
    Address::try_from_val(e, &sc_addr).unwrap()
}

fn address_to_account_id(e: &Env, addr: &Address) -> AccountId {
    let sc_addr: ScAddress = addr.into(); // Uses From<&Address> for ScAddress
    match sc_addr {
        ScAddress::Account(account_id) => account_id,
        _ => panic!("address must be Ed25519 for asset issuer"),
    }
}



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
    usdc_eurc_pair: &Address,
    creator: &Address,
) -> Result<u32, FlashErr> {
    let fee_usdc = 500 * TOKEN_UNIT;
    let unlock = 10;
    let target_lp = 100_000 * TOKEN_UNIT;
    let bonus_flash = 0;
    mgr.create_campaign(&fee_usdc, usdc_eurc_pair, &unlock, &target_lp, &bonus_flash, creator)
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
    let (e, manager, god, users, flash, usdc, eurc, some, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair) = fresh_env_native("my_test");
    std::println!(
        "[test_init_native] manager={:?} god={:?} flash={:?} usdc={:?} eurc={:?} some={:?} factory_addr={:?} flash_usdc_pair={:?} usdc_eurc_pair={:?} usdc_some_pair={:?}",
        manager, god, flash.address, usdc.address, eurc.address, some.address, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair
    );

    let (e, mgr, god, users, flash, usdc, eurc, some, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair) = fresh_env("my_test");
    std::println!(
        "[test_init_wasm] manager={:?} god={:?} flash={:?} usdc={:?} eurc={:?} some={:?} factory_addr={:?} flash_usdc_pair={:?} usdc_eurc_pair={:?} usdc_some_pair={:?}",
        mgr.address, god, flash.address, usdc.address, eurc.address, some.address, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair
    );
}


// The test cases:
#[test]
fn test_initialize_contract_and_seeding() {
    let (e, mgr, god, _users, flash, usdc, eurc, some, _factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair) =
        fresh_env("test_initialize_contract_and_seeding");

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
    std::println!("        flash_usdc_pair reserves: r0={:.7} r1={:.7}", (rf / TOKEN_UNIT) as f64, (ru / TOKEN_UNIT) as f64);
    assert!(rf > 0 && ru > 0);

    let usdc_eurc_pair_client = SoroswapPairClient::new(&e, &usdc_eurc_pair);
    let (r0, r1) = usdc_eurc_pair_client.get_reserves();
    std::println!("        usdc_eurc_pair reserves: r0={:.7} r1={:.7}", (rf / TOKEN_UNIT) as f64, (ru / TOKEN_UNIT) as f64);
   
   
    assert!(r0 > 0 && r1 > 0);

    dump(&e, "setup_and_initialize");
}

#[test]
fn test_create_campaigns_native() {
    std::println!("[TEST] ---- test_create_campaign_no_bonus----");
    let (e, mgr, god, users, flash, usdc, eurc, some, factory_addr, flash_usdc_pair, usdc_eurc_pair, usdc_some_pair) = fresh_env("test_create_campaign_single_user");
    let alice = users.get(0).unwrap();
    std::println!("[TEST] Balances: alice usdc={}", usdc.balance(&alice) / TOKEN_UNIT );
    let bob = users.get(0).unwrap();
    std::println!("[TEST] Balances: bob usdc={}", usdc.balance(&alice) / TOKEN_UNIT );

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
    let result = e.as_contract(&mgr, || {
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
    assert_eq!(result, Err(FlashErr::CampaignActiveForPair), "Second campaign creation for the same pair should fail");

     // Create campaign for a different pair (usdc_some_pair) - should succeed
    let cid2 = e.as_contract(&mgr, || {
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
    std::println!("[TEST_NATIVE] Campaign 2 (different pair) created with cid={}", cid2);
    dump(&e, "create_campaign_single_user");
    let key: Val = (PREFIX_UPOS, cid, alice.clone()).into_val(&e);
    
let upos: Option<UserPos> = e.as_contract(&mgr.address, || e.storage().instance().get(&key));
std::println!("User position: {:?}", upos);
}
// Test: create campaign with single user, no bonus
#[test]
fn test_create_single_user_campaign_no_bonus() {
    let (e, mgr, _god, users, _flash, _usdc, _eurc, some, _factory_addr, _flash_usdc_pair, usdc_eurc_pair, usdc_some_pair) =
        fresh_env("test_create_single_user_campaign_no_bonus");
    let alice = users.get(0).unwrap();

    // Create campaign for usdc_eurc_pair
    let cid1 = create_campaign_for_test(&e, &mgr, &usdc_eurc_pair, &alice);

    // Try to create a second campaign for the same pair (should fail)
    let result = std::panic::catch_unwind(|| {
        create_campaign_for_test(&e, &mgr, &usdc_eurc_pair, &alice)
    });
    assert!(result.is_err(), "Second campaign creation for the same pair should fail");

    // make a second campaign for a different pair.
    // Create campaign for a different pair (usdc_some_pair) - should succeed
    let cid2 = create_campaign_for_test(&e, &mgr, &usdc_some_pair, &alice);
    std::println!("[TEST] Campaign 2 (different pair) created with cid={}", cid2);
    
    // Join campaign as single user
    let join_amount = 2000 * TOKEN_UNIT;
    join_campaign_for_test(&mgr, &cid, join_amount, &alice);

    // Check user position exists
        /*
    let key = upos_key(&e, cid, &alice);
    
let upos: Option<UserPos> = e.as_contract(&mgr.address, || e.storage().instance().get(&key));
std::println!("User position: {:?}", upos);
*/    
    let key = upos_key(&e, cid, &alice);
    let upos: Option<UserPos> = e.as_contract(&mgr.address, || e.storage().instance().get(&key));
    std::println!("User position after join: {:?}", upos);

    assert!(upos.is_some());
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
