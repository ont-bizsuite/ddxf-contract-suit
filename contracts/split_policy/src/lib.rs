#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;
use ostd::abi::{EventBuilder, Sink, Source};
use ostd::database;
use ostd::prelude::*;
use ostd::runtime::{address, check_witness, input, ret, storage_read};
extern crate common;
use common::TokenType;
use ostd::contract::{ong, ont, wasm};

const KEY_TOS: &[u8] = b"01";
const KEY_BALANCE: &[u8] = b"02";

#[derive(Encoder, Decoder)]
struct AddrAmt {
    to: Address,
    percent: U128,
}

fn split(addrs: Vec<Address>, total: U128) -> bool {
    true
}

fn register(key: &[u8], tos: Vec<AddrAmt>) -> bool {
    let data = storage_read(key.as_ref()).map(|val: Vec<u8>| val);
    assert!(data.is_none());
    database::get::<_, Vec<AddrAmt>>(generate_tos_key(key));
    for aa in tos.iter() {
        if check_witness(&aa.to) {
            database::put(generate_tos_key(key), tos);
            EventBuilder::new()
                .string("register")
                .bytearray(key)
                .notify();
            return true;
        }
    }
    false
}

fn transfer(from: &Address, key: &[u8], amt: U128, token_ty: TokenType) -> bool {
    let self_addr = address();
    match token_ty {
        TokenType::ONG => {
            assert!(ong::transfer(from, &self_addr, amt));
        }
        TokenType::ONT => {
            assert!(ont::transfer(from, &self_addr, amt));
        }
        TokenType::OEP4 => {
            //TODO
            let contract_address = contract_addr.unwrap();
            let res =
                wasm::call_contract(&contract_address, ("transfer", (from, to, amt))).unwrap();
            let mut source = Source::new(&res);
            let r: bool = source.read().unwrap();
            assert!(r);
        }
    }
    true
}

fn generate_tos_key(key: &[u8]) -> Vec<u8> {
    [KEY_TOS, key].concat()
}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"split" => {
            let (addrs, total): (Vec<Address>, U128) = source.read().unwrap();
            sink.write(split(addrs, total));
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
