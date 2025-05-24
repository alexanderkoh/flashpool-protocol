use soroban_sdk::{Env, Address, token};
use crate::test::{soroswap_pair::SoroswapPairClient, TOKEN_UNIT};

use crate::tests::utils::order_tokens_and_labels;
extern crate std;

pub fn setup_and_log_pair_liquidity<'a>(
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

