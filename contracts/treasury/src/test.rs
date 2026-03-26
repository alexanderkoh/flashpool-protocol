#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{TreasuryClient, TreasuryError};

fn setup_env() -> (Env, Address, Address, Address, TreasuryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let flash = env.register_stellar_asset_contract_v2(admin.clone()).address().clone();
    let contract_id = env.register(crate::Treasury, ());
    let client = TreasuryClient::new(&env, &contract_id);
    (env, admin, flash, contract_id, client)
}

#[test]
fn test_initialize() {
    let (_env, admin, flash, _contract_id, client) = setup_env();
    client.initialize(&admin, &flash);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize() {
    let (_env, admin, flash, _, client) = setup_env();
    client.initialize(&admin, &flash);
    client.initialize(&admin, &flash);
}

#[test]
fn test_approve_and_get_allowance() {
    let (env, admin, flash, _contract_id, client) = setup_env();
    client.initialize(&admin, &flash);

    let spender = Address::generate(&env);
    client.approve_spender(&spender, &1_000_000i128);

    let allowance = client.get_allowance(&spender);
    assert_eq!(allowance, 1_000_000i128);
}

#[test]
fn test_revoke_spender() {
    let (env, admin, flash, _, client) = setup_env();
    client.initialize(&admin, &flash);

    let spender = Address::generate(&env);
    client.approve_spender(&spender, &1_000_000i128);
    client.revoke_spender(&spender);

    let allowance = client.get_allowance(&spender);
    assert_eq!(allowance, 0i128);
}

#[test]
fn test_withdraw() {
    let (env, admin, flash, contract_id, client) = setup_env();
    client.initialize(&admin, &flash);

    let flash_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &flash);
    flash_admin_client.mint(&contract_id, &500_000i128);

    let recipient = Address::generate(&env);
    client.withdraw(&recipient, &100_000i128);

    let token_client = soroban_sdk::token::Client::new(&env, &flash);
    assert_eq!(token_client.balance(&recipient), 100_000i128);
    assert_eq!(token_client.balance(&contract_id), 400_000i128);
}
