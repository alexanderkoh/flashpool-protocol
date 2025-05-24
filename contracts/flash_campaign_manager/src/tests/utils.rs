#[allow(unused_imports)]
use crate::test::{
    campaign_manager::{FlashCampaignManagerClient, WASM as CM_WASM},
    soroswap_factory::{SoroswapFactoryClient as SS_FC, WASM as soroswap_factory_wasm},
    soroswap_pair::{SoroswapPairClient, WASM as PAIR_WASM},
    TOKEN_UNIT,
};
use crate::tests::{
    log::log_pair_creation, pair::setup_and_log_pair_liquidity, token::create_token,
};
use crate::FlashCampaignManager;
use crate::Manager;
use soroban_sdk::xdr::{AccountId, PublicKey, ScAddress, Uint256};
#[allow(unused_imports)]
use soroban_sdk::{
    testutils::{Address as _, Ledger, Logs, MockAuth, MockAuthInvoke},
    token,
    token::Client as TC,
    Address as A, Env as E, IntoVal, String as SS, TryFromVal, TryIntoVal, Val, Vec,
};
extern crate std;

#[cfg(not(target_family = "wasm"))]
fn soroban_string_to_std(s: &SS) -> std::string::String {
    use soroban_sdk::xdr::{ScString, ScVal};
    let sc_val: ScVal = s.try_into().unwrap();
    if let ScVal::String(ScString(inner)) = sc_val {
        inner.to_utf8_string().unwrap()
    } else {
        panic!("value is not a string");
    }
}

pub fn dump(e: &E, label: &str) {
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

pub fn order_tokens_and_labels<'a>(
    token0: &'a TC,
    token0_label: &'a str,
    token1: &'a TC,
    token1_label: &'a str,
    deposit0: i128,
    deposit1: i128,
) -> (&'a TC<'a>, &'a str, &'a TC<'a>, &'a str, i128, i128) {
    if token0.address < token1.address {
        (
            token0,
            token0_label,
            token1,
            token1_label,
            deposit0,
            deposit1,
        )
    } else {
        (
            token1,
            token1_label,
            token0,
            token0_label,
            deposit1,
            deposit0,
        )
    }
}

pub fn create_pair_ordered<'a>(
    factory: &SS_FC<'a>,
    token_a: &TC<'a>,
    token_b: &TC<'a>,
    label_a: &str,
    label_b: &str,
) -> A {
    let pairaddress: A;
    std::println!(
        "[CREATE_PAIR_ORDERED] --- CREATING PAIR FOR {}/{} ---",
        label_a,
        label_b
    );
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
    E,
    FlashCampaignManagerClient<'a>,
    A,      // god
    Vec<A>, // users
    TC<'a>, // flash
    TC<'a>, // usdc
    TC<'a>, // eurc
    TC<'a>, // some
    A,      // factory
    A,      // flash_usdc_pair
    A,      // usdc_eurc_pair
    A,      // usdc_some_pair
) {
    std::println!(
        "\n── [FRESH_ENV] - tests - setup requested by {} ──",
        caller
    );

    let e = E::default();
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    let god = A::generate(&e);
    let god1 = generate_account_address(&e, 42);
    let user_str = god1.to_string();
    let user_std_str = soroban_string_to_std(&user_str);
    std::println!("Generated user address: {}", user_std_str);
    //let (my_token, my_token_admin) = register_custom_stellar_asset_contract(&e, &god1, "TEST");
    /*std::println!(
            "Created asset contract for user {} with address {:?} with name: {:?} and symbol {:?}",
            user_std_str,
            my_token.address,
            my_token.name(),
            my_token.symbol()
        );
    */
    let mut users = Vec::new(&e);
    for _ in 0..10 {
        let user = A::generate(&e);
        users.push_back(user.clone());
    }
   
    // Setup tokens and pairs
    let (flash, usdc, eurc, some, factory_addr, usdc_eurc_pair, usdc_some_pair) =
        setup_tokens_and_pairs(&e, &god, &users);
    // Deploy and initialize the manager contract
    let manager_address = e.register(CM_WASM, ());
    let manager = FlashCampaignManagerClient::new(&e, &manager_address);

    let initial_flash: i128 = 100_000 * TOKEN_UNIT;
    let initial_usdc: i128 = 250 * TOKEN_UNIT;
    std::println!(
        "[TESTS - FRESH_ENV]\n    [INITIALIZE]\n        initializing the manager contract at address {:?} with initial flash={} and usdc={}",
        manager_address, initial_flash / TOKEN_UNIT, initial_usdc / TOKEN_UNIT
    );
    // Call initialize and capture the returned pair address
    let flash_usdc_pair = manager.initialize(
        &god,
        &flash.address,
        &usdc.address,
        &initial_flash,
        &initial_usdc,
        &factory_addr,
    );
    std::println!(
        "[TESTS - FRESH_ENV]\n    [INITIALIZE]\n        flash_usdc_pair={:?}",
        flash_usdc_pair
    );
    (
        e,
        manager,
        god,
        users,
        flash,
        usdc,
        eurc,
        some,
        factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    )
}

pub fn fresh_env_native<'a>(
    caller: &str,
) -> (
   E,
    A,
    A,      // god
    Vec<A>, // users
    TC<'a>, // flash
    TC<'a>, // usdc
    TC<'a>, // eurc
    TC<'a>, // some
    A,      // factory
    A,      // flash_usdc_pair
    A,      // usdc_eurc_pair
    A,      // usdc_some_pair
) {
    std::println!(
        "\n── [FRESH_ENV1] - tests - setup requested by {} ──",
        caller
    );

    let e = E::default();
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths();

    let god = A::generate(&e);
    let mut users = Vec::new(&e);
    for _ in 0..10 {
        let user = A::generate(&e);
        users.push_back(user.clone());
    }

    let (flash, usdc, eurc, some, factory_addr, usdc_eurc_pair, usdc_some_pair) =
        setup_tokens_and_pairs(&e, &god, &users);
    // Register the contract but DO NOT use the WASM client
    let manager_address = e.register(CM_WASM, ());

    let initial_flash: i128 = 100_000 * TOKEN_UNIT;
    let initial_usdc: i128 = 250 * TOKEN_UNIT;

    // Call the native Rust method directly
    let flash_usdc_pair = e.as_contract(&manager_address, || {
        FlashCampaignManager::initialize(
            e.clone(),
            god.clone(),
            flash.address.clone(),
            usdc.address.clone(),
            initial_flash,
            initial_usdc,
            factory_addr.clone(),
        )
    }).unwrap();

    (
        e,
        manager_address,
        god,
        users,
        flash,
        usdc,
        eurc,
        some,
        factory_addr,
        flash_usdc_pair,
        usdc_eurc_pair,
        usdc_some_pair,
    )
}

pub fn generate_account_address(e: &E, seed: u8) -> A {
    let mut pk = [0u8; 32];
    pk[0] = seed; // Use seed to make addresses unique for each user
    let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(pk)));
    let sc_addr = ScAddress::Account(account_id);
    A::try_from_val(e, &sc_addr).unwrap()
}

pub fn address_to_account_id(_e: &E, addr: &A) -> AccountId {
    let sc_addr: ScAddress = addr.into(); // Uses From<&Address> for ScAddress
    match sc_addr {
        ScAddress::Account(account_id) => account_id,
        _ => panic!("address must be Ed25519 for asset issuer"),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_tokens_and_pairs<'a>(
    e: &E,
    god: &A,
    users: &Vec<A>,
) -> (
    TC<'a>,
    TC<'a>,
    TC<'a>,
    TC<'a>, // flash, usdc, eurc, some
    A,      // factory_addr
    A,      // usdc_eurc_pair
    A,      // usdc_some_pair
) {
    let (flash, flash_admin) = create_token(e, god);
    let (usdc, usdc_admin) = create_token(e, god);
    let (eurc, eurc_admin) = create_token(e, god);
    let (some, some_admin) = create_token(e, god);

    flash_admin.mint(god, &(10_000_000 * TOKEN_UNIT));
    usdc_admin.mint(god, &(1_250_000 * TOKEN_UNIT));
    eurc_admin.mint(god, &(1_000_000 * TOKEN_UNIT));
    some_admin.mint(god, &(1_000_000 * TOKEN_UNIT));

    for user in users.iter() {
        usdc_admin.mint(&user, &(12_500 * TOKEN_UNIT));
        eurc_admin.mint(&user, &(10_000 * TOKEN_UNIT));
        some_admin.mint(&user, &(10_000 * TOKEN_UNIT));
    }

    let pair_wasm_hash = e.deployer().upload_contract_wasm(PAIR_WASM);
    let factory_addr = e.register(soroswap_factory_wasm, ());
    let factory = SS_FC::new(e, &factory_addr);
    factory.initialize(god, &pair_wasm_hash);

    let usdc_eurc_pair = create_pair_ordered(&factory, &usdc, &eurc, "USDC", "EURC");
    setup_and_log_pair_liquidity(
        e,
        god,
        &usdc,
        "USDC",
        &eurc,
        "EURC",
        &usdc_eurc_pair,
        25_000 * TOKEN_UNIT,
        31_250 * TOKEN_UNIT,
    );
    let usdc_some_pair = create_pair_ordered(&factory, &usdc, &some, "USDC", "SOME");
    setup_and_log_pair_liquidity(
        e,
        god,
        &usdc,
        "USDC",
        &some,
        "SOME",
        &usdc_some_pair,
        5_000 * TOKEN_UNIT,
        10_000 * TOKEN_UNIT,
    );

    (
        flash,
        usdc,
        eurc,
        some,
        factory_addr,
        usdc_eurc_pair,
        usdc_some_pair,
    )
}
