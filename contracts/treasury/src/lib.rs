#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, token::Client as TokenClient,
    Address, Env,
};

mod storage;
use storage::*;

#[cfg(test)]
mod test;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 10,
}

#[contract]
pub struct Treasury;

#[contractimpl]
impl Treasury {
    pub fn initialize(e: Env, admin: Address, flash: Address) -> Result<(), TreasuryError> {
        if is_initialized(&e) {
            return Err(TreasuryError::AlreadyInitialized);
        }
        admin.require_auth();
        set_admin(&e, &admin);
        set_flash(&e, &flash);
        set_initialized(&e);
        Ok(())
    }

    pub fn approve_spender(e: Env, spender: Address, amount: i128) -> Result<(), TreasuryError> {
        if !is_initialized(&e) {
            return Err(TreasuryError::NotInitialized);
        }
        let admin = get_admin(&e);
        admin.require_auth();
        let flash = get_flash(&e);
        TokenClient::new(&e, &flash).approve(
            &e.current_contract_address(),
            &spender,
            &amount,
            &e.ledger().sequence().saturating_add(1_000_000),
        );
        Ok(())
    }

    pub fn revoke_spender(e: Env, spender: Address) -> Result<(), TreasuryError> {
        if !is_initialized(&e) {
            return Err(TreasuryError::NotInitialized);
        }
        let admin = get_admin(&e);
        admin.require_auth();
        let flash = get_flash(&e);
        TokenClient::new(&e, &flash).approve(
            &e.current_contract_address(),
            &spender,
            &0i128,
            &e.ledger().sequence(),
        );
        Ok(())
    }

    pub fn get_allowance(e: Env, spender: Address) -> i128 {
        let flash = get_flash(&e);
        TokenClient::new(&e, &flash).allowance(&e.current_contract_address(), &spender)
    }

    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), TreasuryError> {
        if !is_initialized(&e) {
            return Err(TreasuryError::NotInitialized);
        }
        let admin = get_admin(&e);
        admin.require_auth();
        let flash = get_flash(&e);
        TokenClient::new(&e, &flash).transfer(&e.current_contract_address(), &to, &amount);
        Ok(())
    }
}
