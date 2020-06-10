#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate ontio_std as ostd;
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::database;
use ostd::prelude::*;
use ostd::runtime::{address, check_witness, input, ret};
use ostd::types::{Address, U128};
mod utils;
use ostd::contract::{ong, ont, wasm};
use utils::*;
mod basic;
use basic::*;
extern crate common;
use common::{Fee, TokenType};
#[cfg(test)]
mod test;

const MAX_PERCENTAGE: U128 = 10000;
const ADMIN: Address = ostd::macros::base58!("AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP");

fn set_mp(mp_account: &Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(utils::KEY_MP, mp_account);
    true
}

fn set_fee_split_model(seller_acc: &Address, fee_split_model: FeeSplitModel) -> bool {
    assert!(fee_split_model.percentage <= MAX_PERCENTAGE as u16);
    let mp = get_mp_account();
    assert!(check_witness(seller_acc) && check_witness(&mp));
    let mp = database::get::<_, Address>(KEY_MP).unwrap();
    assert!(check_witness(&mp) && check_witness(&seller_acc));
    database::put(
        utils::generate_fee_split_model_key(seller_acc),
        fee_split_model,
    );
    true
}

fn get_fee_split_model(seller_acc: &Address) -> FeeSplitModel {
    database::get::<_, FeeSplitModel>(utils::generate_fee_split_model_key(seller_acc))
        .unwrap_or(FeeSplitModel { percentage: 0 })
}

fn transfer_amount(
    resource_id: &[u8],
    buyer_acc: &Address,
    split_contract_address: &Address,
    fee: Fee,
    n: U128,
) -> bool {
    assert!(check_witness(buyer_acc));
    let amt = n.checked_mul(fee.count as U128).unwrap();
    let self_addr = address();
    assert!(transfer(
        buyer_acc,
        &self_addr,
        amt,
        &fee.contract_type,
        Some(fee.contract_addr.clone()),
    ));

    //store information that split_contract needs
    database::put(utils::generate_resource_id_key(split_contract_address),resource_id);


    let mut balance = balance_of(split_contract_address, &fee.contract_type);
    balance.balance += amt;
    balance.contract_address = Some(fee.contract_addr);
    database::put(
        utils::generate_balance_key(split_contract_address, &fee.contract_type),
        balance,
    );
    true
}

fn balance_of(account: &Address, token_type: &TokenType) -> TokenBalance {
    database::get::<_, TokenBalance>(utils::generate_balance_key(account, token_type))
        .unwrap_or(TokenBalance::new(token_type.clone()))
}

fn settle(seller_acc: &Address, orderId:) -> bool {
    assert!(check_witness(seller_acc));
    let self_addr = address();
    let mp = get_mp_account();
    let tokens = vec![TokenType::ONG, TokenType::ONT, TokenType::OEP4];
    for token in tokens.iter() {
        let balance = balance_of(seller_acc, token);
        if balance.balance != 0 {
            assert!(settle_inner(seller_acc, &self_addr, &mp, balance));
        }
        database::delete(utils::generate_balance_key(seller_acc, token));
    }
    true
}

fn settle_inner(
    seller_acc: &Address,
    self_addr: &Address,
    mp: &Address,
    balance: TokenBalance,
) -> bool {
    let fee_split = get_fee_split_model(seller_acc);
    let fee = balance
        .balance
        .checked_mul(fee_split.percentage as U128)
        .unwrap();
    let mp_amt = fee.checked_div(MAX_PERCENTAGE).unwrap();
    assert!(transfer(
        &self_addr,
        &mp,
        mp_amt,
        &balance.token_type,
        balance.contract_address
    ));
    let seller_amt = balance.balance.checked_sub(mp_amt).unwrap();
    assert!(transfer(
        &self_addr,
        seller_acc,
        seller_amt,
        &balance.token_type,
        balance.contract_address
    ));
    true
}

fn transfer(
    from: &Address,
    to: &Address,
    amt: U128,
    contract_type: &TokenType,
    contract_addr: Option<Address>,
) -> bool {
    match contract_type {
        TokenType::ONG => {
            assert!(ong::transfer(from, to, amt));
        }
        TokenType::ONT => {
            assert!(ont::transfer(from, to, amt));
        }
        TokenType::OEP4 => {
            //TODO
            let contract_address = contract_addr.unwrap();
            let res =
                wasm::call_contract(&contract_address, ("transfer", (from, to, amt))).unwrap();
            let mut source = Source::new(&res);
            let b: bool = source.read().unwrap();
            assert!(b);
        }
    }
    true
}

fn get_mp_account() -> Address {
    database::get::<_, Address>(utils::KEY_MP).unwrap()
}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"setFeeSplitModel" => {
            let (seller_acc, fee_split_model) = source.read().unwrap();
            sink.write(set_fee_split_model(seller_acc, fee_split_model));
        }
        b"get_fee_split_model" => {
            let seller_acc = source.read().unwrap();
            sink.write(get_fee_split_model(seller_acc));
        }
        b"transferAmount" => {
            let (buyer_acc, seller_acc, fee, n) = source.read().unwrap();
            sink.write(transfer_amount(buyer_acc, seller_acc, fee, n));
        }
        b"balance_of" => {
            let (account, token_type) = source.read().unwrap();
            sink.write(balance_of(account, &token_type));
        }
        b"settle" => {
            let seller_acc = source.read().unwrap();
            sink.write(settle(seller_acc));
        }
        b"set_mp" => {
            let mp_addr = source.read().unwrap();
            sink.write(set_mp(mp_addr));
        }
        b"get_mp_account" => {
            sink.write(get_mp_account());
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("not support method:{}", method)
        }
    }
    ret(sink.bytes());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
