#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate ontio_std as ostd;
use alloc::collections::btree_map::BTreeMap;
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::database;
use ostd::prelude::H256;
use ostd::prelude::*;
use ostd::runtime::{check_witness, input, ret};
use ostd::types::{Address, U128};

const ADMIN: Address = ostd::macros::base58!("AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP");

const KEY_FEE_SPLIT_MODEL: &[u8] = b"01";
const KEY_SELLER: &[u8] = b"02";
const KEY_MP: &[u8] = b"03";

#[derive(Encoder, Decoder)]
struct FeeSplitModel {
    percentage: u16,
}

fn init(mp_account: &Address, seller: &Address) -> bool {
    database::put(KEY_MP, mp_account);
    database::put(KEY_SELLER, seller);
    true
}

fn set_fee_split_model(fee_split_model: FeeSplitModel) -> bool {
    let mp = database::get::<_, Address>(KEY_MP).unwrap();
    let seller = database::get::<_, Address>(KEY_SELLER).unwrap();
    assert!(check_witness(&mp) && check_witness(&seller));
    database::put(KEY_FEE_SPLIT_MODEL, fee_split_model);
    true
}

fn get_fee_split_model() -> FeeSplitModel {
    database::get::<_, FeeSplitModel>(utils::generate_fee_split_model_key())
        .unwrap_or(FeeSplitModel { percentage: 0 })
}

fn settle() -> bool {}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"setFeeSplitModel" => {
            let (account, resource_id, templates, n) = source.read().unwrap();
            sink.write(generate_dtoken(account, resource_id, templates, n));
        }
        b"useToken" => {
            let (account, resource_id, token_hash, n) = source.read().unwrap();
            sink.write(use_token(account, resource_id, token_hash, n));
        }
        b"useTokenByAgent" => {
            let (account, agent, resource_id, token_hash, n) = source.read().unwrap();
            sink.write(use_token_by_agent(
                account,
                agent,
                resource_id,
                token_hash,
                n,
            ));
        }
        b"transferDtoken" => {
            let (from_account, to_account, resource_id, account, n) = source.read().unwrap();
            sink.write(transfer_dtoken(
                from_account,
                to_account,
                resource_id,
                account,
                n,
            ));
        }
        b"setAgent" => {
            let (account, resource_id, agents, n) = source.read().unwrap();
            sink.write(set_agent(account, resource_id, agents, n));
        }
        b"setTokenAgents" => {
            let (account, resource_id, token_hash, agents, n) = source.read().unwrap();
            sink.write(set_token_agents(
                account,
                resource_id,
                token_hash,
                agents,
                n,
            ));
        }
        b"addAgents" => {
            let (account, resource_id, agents, n) = source.read().unwrap();
            sink.write(add_agents(account, resource_id, agents, n));
        }
        b"addTokenAgents" => {
            let (account, resource_id, token_hash, agents, n) = source.read().unwrap();
            sink.write(add_token_agents(
                account,
                resource_id,
                token_hash,
                agents,
                n,
            ));
        }
        b"removeAgents" => {
            let (account, resource_id, agents) = source.read().unwrap();
            sink.write(remove_agents(account, resource_id, agents));
        }
        b"removeTokenAgents" => {
            let (account, resource_id, token_hash, agents) = source.read().unwrap();
            sink.write(remove_token_agents(
                account,
                resource_id,
                token_hash,
                agents,
            ));
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
