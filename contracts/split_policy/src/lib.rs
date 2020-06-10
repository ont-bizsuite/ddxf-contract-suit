#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;
use ostd::abi::{EventBuilder, Sink, Source};
use ostd::database;
use ostd::prelude::*;
use ostd::runtime::{address, check_witness, input, ret, storage_read};
extern crate common;
use common::TokenType;
use ostd::abi::{Decoder, Encoder};
use ostd::contract::{ong, ont, wasm};

#[cfg(test)]
mod test;

const KEY_TOS: &[u8] = b"01";
const KEY_BALANCE: &[u8] = b"02";

const TOTAL: U128 = 10000;

#[derive(Encoder, Decoder, Clone)]
struct AddrAmt {
    to: Address,
    percent: U128,
    has_withdraw: bool,
}

#[derive(Encoder, Decoder)]
struct RegisterParam {
    addr_amt: Vec<AddrAmt>,
    token_type: TokenType,
    contract_addr: Option<Address>,
}

impl RegisterParam {
    fn default() -> Self {
        RegisterParam {
            addr_amt: vec![],
            token_type: TokenType::ONG,
            contract_addr: None,
        }
    }
    fn from_bytes(data: &[u8]) -> RegisterParam {
        let mut source = Source::new(data);
        let rp: RegisterParam = source.read().unwrap();
        rp
    }
}

fn register(key: &[u8], param_bytes: &[u8]) -> bool {
    let param = RegisterParam::from_bytes(param_bytes);
    let data = storage_read(key.as_ref()).map(|val: Vec<u8>| val);
    assert!(data.is_none());
    match param.token_type {
        TokenType::OEP4 => {
            assert!(param.contract_addr.is_some());
        }
        _ => {}
    }
    let mut total: U128 = 0;
    let mut valid = false;
    for aa in param.addr_amt.iter() {
        total += aa.percent;
        if !valid {
            valid = check_witness(&aa.to);
        }
    }
    assert!(valid);
    assert_eq!(total, TOTAL);
    database::put(generate_tos_key(key), param);
    EventBuilder::new()
        .string("register")
        .bytearray(key)
        .notify();
    true
}

fn get_register_param(key: &[u8]) -> RegisterParam {
    database::get::<_, RegisterParam>(generate_tos_key(key)).unwrap_or(RegisterParam::default())
}

fn transfer(from: &Address, key: &[u8], amt: U128) -> bool {
    let self_addr = address();
    let param = get_register_param(key);
    assert!(transfer_inner(
        from,
        &self_addr,
        amt,
        &param.token_type,
        param.contract_addr
    ));
    let balance = get_balance(key);
    let balance = balance.checked_add(amt).unwrap();
    database::put(generate_balance_key(key), balance);
    true
}

fn get_balance(key: &[u8]) -> U128 {
    database::get::<_, U128>(generate_balance_key(key)).unwrap_or(0)
}

fn withdraw(key: &[u8], addr: &Address) -> bool {
    assert!(check_witness(addr));
    let mut rp = get_register_param(key);
    let index = rp.addr_amt.iter().position(|addr_amt| &addr_amt.to == addr);
    if let Some(ind) = index {
        let addr_amt = rp.addr_amt.get_mut(ind).unwrap();
        if addr_amt.has_withdraw == false {
            let self_addr = address();
            let balance = get_balance(key);
            let temp = balance.checked_mul(addr_amt.percent).unwrap();
            let amt = temp.checked_div(TOTAL).unwrap();
            assert!(transfer_inner(
                &self_addr,
                &addr_amt.to,
                amt,
                &rp.token_type,
                rp.contract_addr
            ));
            addr_amt.has_withdraw = true;
            database::put(generate_tos_key(key), rp);
            return true;
        } else {
            panic!("has withdraw")
        }
    }
    panic!("not found the addr")
}

fn transfer_inner(
    from: &Address,
    to: &Address,
    amt: U128,
    token_type: &TokenType,
    contract: Option<Address>,
) -> bool {
    match token_type {
        TokenType::ONG => {
            assert!(ong::transfer(from, to, amt));
        }
        TokenType::ONT => {
            assert!(ont::transfer(from, to, amt));
        }
        TokenType::OEP4 => {
            //TODO
            if let Some(contract_addr) = contract {
                let res =
                    wasm::call_contract(&contract_addr, ("transfer", (from, to, amt))).unwrap();
                let mut source = Source::new(&res);
                let r: bool = source.read().unwrap();
                assert!(r);
                return true;
            }
            panic!("not reachable");
        }
    }
    true
}

fn generate_tos_key(key: &[u8]) -> Vec<u8> {
    [KEY_TOS, key].concat()
}

fn generate_balance_key(key: &[u8]) -> Vec<u8> {
    [KEY_BALANCE, key].concat()
}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"register" => {
            let (key, param_bytes) = source.read().unwrap();
            sink.write(register(key, param_bytes));
        }
        b"get_register_param" => {
            let key = source.read().unwrap();
            sink.write(get_register_param(key));
        }
        b"transfer" => {
            let (from, key, amt): (Address, &[u8], U128) = source.read().unwrap();
            sink.write(transfer(&from, key, amt));
        }
        b"get_balance" => {
            let key = source.read().unwrap();
            sink.write(get_balance(key));
        }
        b"withdraw" => {
            let (key, addr): (&[u8], Address) = source.read().unwrap();
            sink.write(withdraw(key, &addr));
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
