use super::alloc::vec::Vec;
use super::*;
const KEY_FEE_SPLIT_MODEL: &[u8] = b"01";
const KEY_BALANCE: &[u8] = b"02";
pub const KEY_MP: &[u8] = b"03";
const KEY_RESOURCE_ID :&[u8] = b"04";

pub fn generate_fee_split_model_key(account: &Address) -> Vec<u8> {
    [KEY_FEE_SPLIT_MODEL, account.as_ref()].concat()
}
pub fn generate_balance_key(account: &Address, token_type: &TokenType) -> Vec<u8> {
    let token_ty: &[u8] = match token_type {
        TokenType::OEP4 => b"oep4",
        TokenType::ONT => b"ont",
        TokenType::ONG => b"ong",
    };
    [KEY_BALANCE, account.as_ref(), token_ty].concat()
}

pub fn generate_resource_id_key(addr: &Address) -> Vec<u8> {
    [KEY_RESOURCE_ID, addr.as_ref()].concat()
}