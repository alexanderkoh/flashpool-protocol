use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub const KEY_ADMIN: Symbol = symbol_short!("ADMIN");
pub const KEY_FLASH: Symbol = symbol_short!("FLASH");
pub const KEY_INIT: Symbol = symbol_short!("INIT");

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&KEY_ADMIN, admin);
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&KEY_ADMIN).unwrap()
}

pub fn set_flash(e: &Env, flash: &Address) {
    e.storage().instance().set(&KEY_FLASH, flash);
}

pub fn get_flash(e: &Env) -> Address {
    e.storage().instance().get(&KEY_FLASH).unwrap()
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&KEY_INIT, &true);
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().get(&KEY_INIT).unwrap_or(false)
}
