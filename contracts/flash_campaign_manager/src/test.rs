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
#[allow(unused_imports)]
use super::*;
use soroban_sdk::xdr::{
    AccountId, AlphaNum4, Asset, AssetCode4, ContractExecutable, ContractIdPreimage,
    CreateContractArgs, HostFunction, PublicKey, ScAddress, ScVal, Uint256,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger, Logs, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal, String as SorobanString, TryFromVal, TryIntoVal, Val, Vec,
};
use crate::storage::upos_key;

extern crate std;

#[cfg(not(target_family = "wasm"))]
fn soroban_string_to_std(s: &SorobanString) -> std::string::String {
    use soroban_sdk::xdr::{ScString, ScVal};
    let sc_val: ScVal = s.try_into().unwrap();
    if let ScVal::String(ScString(inner)) = sc_val {
        inner.to_utf8_string().unwrap()
    } else {
        panic!("value is not a string");
    }
}

mod campaign_manager {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/flash_campaign_manager.wasm");
    pub type FlashCampaignManagerClient<'a> = Client<'a>;
}
use campaign_manager::FlashCampaignManagerClient;
mod soroswap_pair {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_pair.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
use soroswap_pair::SoroswapPairClient;
mod soroswap_factory {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/soroswap_factory.wasm");
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
use soroswap_factory::SoroswapFactoryClient;

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

fn create_pair_ordered<'a>(
    factory: &soroswap_factory::Client<'a>,
    token_a: &token::Client<'a>,
    token_b: &token::Client<'a>,
    label_a: &str,
    label_b: &str,
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
fn log_pair_creation(
    pair_addr: &Address,
    token0_label: &str,
    token0_addr: &Address,
    token1_label: &str,
    token1_addr: &Address,
) {
    std::println!("     PAIR CREATED\n          {:?}", pair_addr);
    std::println!(
        "             token0: {t0}\n             Address:    {:?}",
        token0_addr,
        t0 = token0_label
    );
    std::println!(
        "             token1: {t1}\n             Address:    {:?}",
        token1_addr,
        t1 = token1_label
    );
}

fn fresh_env<'a>(
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
        flash.address,
        flashsymbol,
        flashname,
        usdc.address,
        usdcsymbol,
        usdcname
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
        
    // Only create the USDC/EURC pair here
    let usdc_eurc_pair = create_pair_ordered(&factory, &usdc, &eurc, "USDC", "EURC");
    setup_and_log_pair_liquidity(
        &e,
        &god,
        &usdc,
        "USDC",
        &eurc,
        "EURC",
        &usdc_eurc_pair,
        250_000 * TOKEN_UNIT,
        312_500 * TOKEN_UNIT,
    );

    // Deploy and initialize the manager contract
    let manager_address = e.register(campaign_manager::WASM, ());
    let manager = FlashCampaignManagerClient::new(&e, &manager_address);

    let initial_flash: i128 = 100_000 * TOKEN_UNIT;
    let initial_usdc: i128 = 250 * TOKEN_UNIT;
    std::println!(
        "[TESTS - FRESH_ENV]\n    [INITIALIZE]\n        initializing the manager contract at address {:?} with initial flash={} and usdc={}",
        manager_address, initial_flash / TOKEN_UNIT, initial_usdc / TOKEN_UNIT
    );
    // Call initialize and capture the returned pair address
    let flash_usdc_pair = manager.initialize(&god, &flash.address, &usdc.address, &initial_flash, &initial_usdc, &factory_addr);
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

fn order_tokens_and_labels<'a>(
    token0: &'a token::Client,
    token0_label: &'a str,
    token1: &'a token::Client,
    token1_label: &'a str,
    deposit0: i128,
    deposit1: i128,
) -> (
    &'a token::Client<'a>,
    &'a str,
    &'a token::Client<'a>,
    &'a str,
    i128,
    i128,
) {
    if token0.address < token1.address {
        (token0, token0_label, token1, token1_label, deposit0, deposit1)
    } else {
        (token1, token1_label, token0, token0_label, deposit1, deposit0)
    }
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

    
    std::println!("[SETUP PAIR LIQUIDITY] --- {:?} {t0}/{t1} ---\n \n     Depositing:\n            {t0}:{:?} -     {:.7}\n            {t1}:{:?} -     {:.7}",
        pair_addr,
        token_a.address.to_string(),
        deposit_a as f64 / TOKEN_UNIT as f64,
        token_b.address.to_string(),
        deposit_b as f64 / TOKEN_UNIT as f64,
        t0=label_a,
        t1=label_b
    );
    std::println!("    User god address: {:?}\n    balances:\n        {t0}: {:.7}\n        {t1}: {:.7}", god, token_a.balance(god) as f64 / TOKEN_UNIT as f64, token_b.balance(god) as f64 / TOKEN_UNIT as f64, t0=label_a, t1=label_b);

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

fn generate_account_address(e: &Env, seed: u8) -> Address {
    use soroban_sdk::xdr::{AccountId, PublicKey, ScAddress, Uint256};
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

fn register_custom_stellar_asset_contract<'a>(
    e: &Env,
    admin: &Address,
    code: &str, // up to 4 chars
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    // Pad or truncate code to 4 bytes
    let mut code_bytes = [0u8; 4];
    let code_slice = code.as_bytes();
    for (i, b) in code_slice.iter().take(4).enumerate() {
        code_bytes[i] = *b;
    }
    let issuer = address_to_account_id(e, admin);
    let asset = Asset::CreditAlphanum4(AlphaNum4 {
        asset_code: AssetCode4(code_bytes),
        issuer,
    });
    let create = HostFunction::CreateContract(CreateContractArgs {
        contract_id_preimage: ContractIdPreimage::Asset(asset),
        executable: ContractExecutable::StellarAsset,
    });

    // Call the host to create the contract
    let contract_id: Address = e
        .host()
        .invoke_function(create)
        .unwrap()
        .try_into_val(e)
        .unwrap();

    // Set admin
    let prev_auth_manager = e.host().snapshot_auth_manager().unwrap();
    e.host()
        .switch_to_recording_auth_inherited_from_snapshot(&prev_auth_manager)
        .unwrap();
    let client = token::StellarAssetClient::new(e, &contract_id);
    client.set_admin(admin);
    e.host().set_auth_manager(prev_auth_manager).unwrap();

    (
        token::Client::new(e, &contract_id),
        token::StellarAssetClient::new(e, &contract_id),
    )
}

fn set_sac_admin_to_contract(
    e: &Env,
    sac_admin: &Address,
    sac_client: &token::StellarAssetClient,
    contract_addr: &Address,
) {
    // Build args using Soroban Vec
    let mut args = Vec::new(e);
    args.push_back(contract_addr.clone().into_val(e));
    std::println!(
    "[set_sac_admin_to_contract] contract_addr={:?}",
    contract_addr);
    let dummy_invoke = MockAuthInvoke {
        contract: &sac_client.address, // the SAC contract address
        fn_name: "set_admin",
        args,
        sub_invokes: &[],
    };
    e.mock_auths(&[MockAuth {
        address: sac_admin,
        invoke: &dummy_invoke,
    }]);

    sac_client.set_admin(contract_addr);

    std::println!(
        "SAC admin changed from {:?} to contract address {:?}",
        sac_admin, contract_addr
    );
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

// The test cases:
#[test]
fn test_initialize_contract_and_seeding() {
    let (e, mgr, god, _users, flash, usdc, eurc, _factory_addr, flash_usdc_pair, usdc_eurc_pair) =
        fresh_env("test_initialize_contract_and_seeding");

    let manager_address = &mgr.address;
    let usdc_address = &usdc.address;
    let eurc_address = &eurc.address;
    let flash_address = &flash.address;
    std::println!("[test_initialize_contract_and_seeding]\ninitialized an env:\n  Addresses:\n   manager:       {:?}\n   god address=   {:?}\n   flash address= {:?}\n   usdc address=  {:?}\n   eurc address=  {:?}\n   factory=       {:?}\n   flash_usdc=    {:?}\n   usdc_eurc=     {:?}",manager_address, god, flash_address, usdc_address, eurc_address, _factory_addr, flash_usdc_pair, usdc_eurc_pair);

    // Log contract's balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for contract:\n   flash=    {}\n   usdc=     {}\n   eurc=     {}",
        flash.balance(&mgr.address) / TOKEN_UNIT,
        usdc.balance(&mgr.address) / TOKEN_UNIT,
        eurc.balance(&mgr.address) / TOKEN_UNIT
    );

    // Log god's balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for god:\n   flash=    {}\n   usdc=     {}\n   eurc=     {}",
        flash.balance(&god) / TOKEN_UNIT,
        usdc.balance(&god) / TOKEN_UNIT,
        eurc.balance(&god) / TOKEN_UNIT
    );

    // Log pair balances
    std::println!(
        "[test_initialize_contract_and_seeding]\nBalances for pairs:\n   flash_usdc: flash={} usdc={}\n   usdc_eurc: usdc={} eurc={}",
        flash.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&flash_usdc_pair) / TOKEN_UNIT,
        usdc.balance(&usdc_eurc_pair) / TOKEN_UNIT,
        eurc.balance(&usdc_eurc_pair) / TOKEN_UNIT
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
fn test_create_campaign_single_user_no_bonus() {
    std::println!("[TEST] ---- test_create_campaign_single_user_no_bonus----");
    let (e, mgr, _god, users, _flash, usdc, eurc, _factory_addr, _flash_usdc_pair, usdc_eurc_pair) =
        fresh_env("test_create_campaign_single_user");
    let alice = users.get(0).unwrap();
    std::println!(
        "[TEST] Balances: alice usdc={}",
        usdc.balance(&alice) / TOKEN_UNIT
    );
    let cid = mgr.create_campaign(&(500 * TOKEN_UNIT), &usdc_eurc_pair, &10, &(100_000*TOKEN_UNIT), &0, &alice);
    dump(&e, "create_campaign_single_user");
    let key = upos_key(&e, cid, &alice);
    assert!(!e.storage().instance().has(&key));
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
