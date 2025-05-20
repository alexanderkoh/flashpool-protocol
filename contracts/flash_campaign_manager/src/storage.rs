use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, Symbol, Val};

use crate::Campaign;

pub const KEY_ADMIN: &str = "A";
pub const KEY_FLASH: &str = "F";
pub const KEY_USDC : &str = "U";
pub const KEY_CORE_PAIR: &str = "P";
pub const KEY_NEXT : &str = "N";
pub const KEY_SURP : &str = "S";
pub const KEY_TTLT : &str = "T";
pub const KEY_TTLB : &str = "B";
pub const PREFIX_UCNT: &str = "UC";
pub const PREFIX_CAMP: &str = "C";
pub const PREFIX_UPOS: &str = "U";

pub fn s(e:&Env,k:&'static str)->Symbol { Symbol::new(e,k) }
pub fn set_addr(e:&Env,k:&'static str,a:&Address){ e.storage().instance().set(&s(e,k),a) }
pub fn get_addr(e:&Env,k:&'static str)->Address { e.storage().instance().get(&s(e,k)).unwrap_optimized() }
pub fn set_u32(e:&Env,k:&'static str,v:u32){ e.storage().instance().set(&s(e,k),&v) }
pub fn get_u32(e:&Env,k:&'static str,d:u32)->u32{ e.storage().instance().get(&s(e,k)).unwrap_or(d) }

pub fn camp_key(e:&Env,id:u32)->Val { (PREFIX_CAMP,id).into_val(e) }
pub fn upos_key(e:&Env,id:u32,w:&Address)->Val { (PREFIX_UPOS,id,w).into_val(e) }
pub fn ucnt_key(e: &Env, id: u32) -> Val { (PREFIX_UCNT, id).into_val(e) }

pub fn load_camp(e:&Env,id:u32)->Campaign {
    e.storage().instance().get::<Val,Campaign>(&camp_key(e,id)).unwrap_optimized()
}
pub fn save_camp(e:&Env,id:u32,c:&Campaign){
    e.storage().instance().set::<Val,Campaign>(&camp_key(e,id),c)
}