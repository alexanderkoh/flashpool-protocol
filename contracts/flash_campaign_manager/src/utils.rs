use soroban_sdk::{Address as A, Env as E, token::Client as TC,  token::StellarAssetClient as SAC};
use crate::{pair::Client as PC, FlashErr, soroswap_factory::Client as FC};


pub trait CheckedCeilingDiv {
    fn checked_ceiling_div(self, divisor: i128) -> Option<i128>;
}

impl CheckedCeilingDiv for i128 {
    fn checked_ceiling_div(self, divisor: i128) -> Option<i128> {
        let result = self.checked_div(divisor)?;
        if self % divisor != 0 {
            result.checked_add(1)
        } else {
            Some(result)
        }
    }
}

// order token clients: returns ordered token clients for a pair in a tuple.
// todo: perhaps there's a variation that can use a struct to determine named orders instead of unnamed.
pub fn o_t_c<'a>(e: &'a E, t0: &'a A, t1: &'a A) -> (TC<'a>, TC<'a>) {
    if t0 >= t1 {
        (TC::new(e, t1), TC::new(e, t0))
    } else {
        (TC::new(e, t0), TC::new(e, t1))
    }
}

// deposit_to_pair:
// processes a deposit for a soroswap pair for the current contract by transferring the specified tokens and minting lp tokens.
pub fn d_t_p(
    e: &E,
    p: &A,
    t0: &A,
    t1: &A,
    a0: i128,
    a1: i128,
) -> i128 {
    let (tc0, tc1) = o_t_c(e, t0, t1);
    let cca = &e.current_contract_address();
    // Approve
    tc0.approve(cca, p, &a0, &0);
    tc1.approve(cca, p, &a1, &0);

    // Transfer
    tc0.transfer(cca, p, &a0);
    tc1.transfer(cca, p, &a1);

    // Deposit
    let pc = PC::new(e, p);
    let lp = pc.deposit(cca);
    lp
}

// calculate_liquidity_amounts:
/// Given a pair and an amount for either token0 or token1, returns the required amount of the other token for balanced liquidity.
/// Pass `which = 0` if `amount` is for token0, or `which = 1` if `amount` is for token1.
#[allow(dead_code)]
pub fn c_l_a(e: &E, p: &A, a: i128, w: u8) -> Result<i128, FlashErr> {
    let pc = PC::new(e, p);
    let (r0, r1) = pc.get_reserves();
    match w {
        0 => Ok(a.checked_mul(r1).unwrap().checked_div(r0).unwrap()),
        1 => Ok(a.checked_mul(r0).unwrap().checked_div(r1).unwrap()),
        _ => Err(FlashErr::InvalidToken),
    }
}

// calculate_swap_output:
// given an input and reserves calculate the output amount.
// amount_in: The amount of the token you are swapping into the pool (the input token).
// reserve_in: The current reserve of the input token in the pool.
// reserve_out: The current reserve of the output token in the pool.
/*How do you know which is which?
If you are swapping token0 for token1:
    amount_in   :  amount of token0 you provide
    reserve_in  :  reserve of token0 in the pool
    reserve_out :  reserve of token1 in the pool

If you are swapping token1 for token0:
    amount_in   :  amount of token1 you provide
    reserve_in  :  reserve of token1 in the pool
    reserve_out :  reserve of token0 in the pool
*/
pub fn c_s_o(a: i128, r_in: i128, r_out: i128) -> i128 {
    let wf = a.checked_mul(997).unwrap();
    let n = wf.checked_mul(r_out).unwrap();
    let d = r_in.checked_mul(1000).unwrap().checked_add(wf).unwrap();
    n.checked_div(d).unwrap()
}

pub fn c_s_o1(a_in: i128, r_in: i128, r_out: i128) -> i128 {
    let fee = (a_in.checked_mul(3).unwrap()).checked_ceiling_div(1000).unwrap();
    let net_in = a_in.checked_sub(fee).unwrap();
    // Standard constant product out calculation
    let n = net_in.checked_mul(r_out).unwrap();
    let d = r_in.checked_add(net_in).unwrap();
    n.checked_div(d).unwrap()
}

// create_pair:
// create an ordered pair (a liquidity pool with token0 and token1 in the correct order) by calling the soroswap factory contract:
// args:
// e: The current environment
// f: The soroswap factory contract client
// t_a: The address of the first token
// t_b: The address of the second token
// returns the address of the new pair contract
pub fn c_p<'a>(e: &'a E, f: &'a FC, t_a: &'a A, t_b: &'a A) -> A {
    let (t0, t1) = o_t_c(e, &t_a, &t_b);
    f.create_pair(&t0.address, &t1.address)
}

/* 
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
*/

/*
/// Transfers admin of a Stellar Asset Contract to this contract.
pub fn claim_sac_admin(e: Env, sac_contract: Address, current_admin: Address) {
    current_admin.require_auth();
    let sac = StellarAssetClient::new(&e, &sac_contract);
    sac.set_admin(&e.current_contract_address());
}
fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
    let config = get_core_config(&e);
    config.admin.require_auth();
    e.deployer().update_current_contract_wasm(new_wasm_hash);
}
fn version() -> u32 {
    1
}
     */

// claim_sac_admin:
/// Transfers admin of a Stellar Asset Contract to this contract.
/// args:
/// env(e): The current environment
/// sac_contract(sac): The address of the Stellar Asset Contract
/// current_admin(ad): The address of the current admin
#[allow(dead_code)]
pub fn csa(
    e: &E,
    sac: &A,
    ad: &A,
) {
    ad.require_auth();
    let sac = SAC::new(e, sac);
    sac.set_admin(&e.current_contract_address());
}

#[allow(dead_code)]
pub fn cca(e: &E) -> A {
    e.current_contract_address()
}