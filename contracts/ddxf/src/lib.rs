#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate ontio_std as ostd;
use alloc::collections::btree_map::BTreeMap;
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::database;
use ostd::prelude::H256;
use ostd::prelude::*;
use ostd::runtime;
use ostd::types::{Address, U128};
pub mod basic;
use basic::*;

const SHA256_SIZE: u32 = 32;
const CRC32_SIZE: u32 = 4;

const KEY_SELLER_ITEM_INFO: &[u8] = b"01";
const KEY_SELLER_ITEM_SOLD: &[u8] = b"02";

fn dtoken_seller_publish(resource_id: &str, resource_ddo: &ResourceDDO, item: &DTokenItem) -> bool {
    assert!(runtime::check_witness(&resource_ddo.manager));
    let resource =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(resource.is_none());
    if &resource_ddo.endpoint == "" {
        assert_ne!(resource_ddo.token_endpoint.len(), 0);
        for (token_hash, _) in item.templates.iter() {
            assert_ne!(resource_ddo.token_endpoint[token_hash], "");
        }
    }
    assert_ne!(item.templates.len(), 0);
    for (token_hash, _) in item.templates.iter() {
        let rt = resource_ddo
            .token_resource_type
            .get(token_hash)
            .unwrap_or(&resource_ddo.resource_type);
        match rt {
            RT::RTStaticFile => {
                assert_eq!(token_hash.len() as u32, SHA256_SIZE + CRC32_SIZE);
            }
        }
    }

    if item.templates.len() > 1 {
        assert!(resource_ddo.desc_hash.is_some())
    }
    let seller = SellerItemInfo::new(item.clone(), resource_ddo.clone());
    database::put(utils::generate_seller_item_info_key(resource_id), seller);
    let mut sink = Sink::new(16);
    resource_ddo.encode(&mut sink);
    let mut sink2 = Sink::new(16);
    (*item).encode(&mut sink2);
    events::dtoken_seller_publish_event(resource_id, sink.bytes(), sink2.bytes());
    true
}

fn buy_dtoken_from_reseller(
    resource_id: &str,
    n: U128,
    buyer_account: &Address,
    reseller_account: &Address,
) -> bool {
    assert!(runtime::check_witness(buyer_account) && runtime::check_witness(reseller_account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(transfer_fee(
        buyer_account,
        reseller_account,
        &item_info.item.fee,
        n
    ));
    assert!(transfer_dtoken(
        reseller_account,
        buyer_account,
        resource_id,
        &item_info.item.templates,
        n
    ));
    true
}

fn buy_dtoken(resource_id: &str, n: U128, buyer_account: &Address) -> bool {
    assert!(runtime::check_witness(buyer_account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let now = runtime::timestamp();
    assert!(now < item_info.item.expired_date);
    let sold = database::get::<_, U128>(utils::generate_seller_item_sold_key(resource_id)).unwrap();
    assert!(sold < item_info.item.stocks as U128);
    let sum = sold.checked_add(n).unwrap();
    assert!(sum < item_info.item.stocks as U128);
    assert!(transfer_fee(
        buyer_account,
        &item_info.resource_ddo.manager,
        &item_info.item.fee,
        n
    ));
    database::put(utils::generate_seller_item_sold_key(resource_id), sum);
    assert!(generate_dtoken(
        buyer_account,
        resource_id,
        &item_info.item.templates,
        n
    ));
    true
}

fn use_token(resource_id: &str, account: &Address, token_hash: &H256, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(item_info.is_some());
    assert!(use_token_dtoken(account, resource_id, token_hash, n));
    true
}

fn use_token_by_agent(
    resource_id: &str,
    account: &Address,
    agent: &Address,
    token_hash: &str,
    n: U128,
) -> bool {
    assert!(runtime::check_witness(agent));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(item_info.is_some());
    assert!(use_token_by_agent_dtoken(
        account,
        agent,
        resource_id,
        token_hash,
        n
    ));
    true
}

fn set_dtoken_agents(resource_id: &str, account: &Address, agents: Vec<&Address>, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(item_info.is_some());
    set_agents_dtoken(account, resource_id, agents, n);
    true
}

fn add_dtoken_agents(resource_id: &str, account: &Address, agents: Vec<&Address>, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(item_info.is_some());
    assert!(add_dtoken_agents_dtoken(account, resource_id, agents, n));
    true
}

fn remove_dtoken_agents(resource_id: &str, account: &Address, agents: Vec<&Address>) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(item_info.is_some());
    assert!(remove_agents(account, resource_id, agents));
    true
}

fn remove_agents(account: &Address, resource_id: &str, agents: Vec<&Address>) -> bool {
    true
}

fn transfer_fee(buyer_account: &Address, reseller_account: &Address, fee: &Fee, n: U128) -> bool {
    true
}

fn set_agents_dtoken(account: &Address, resource_id: &str, agents: Vec<&Address>, n: U128) -> bool {
    true
}
fn use_token_dtoken(account: &Address, resource_id: &str, token_hash: &H256, n: U128) -> bool {
    true
}

fn add_dtoken_agents_dtoken(
    account: &Address,
    resource_id: &str,
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    true
}
fn use_token_by_agent_dtoken(
    account: &Address,
    agent: &Address,
    resource_id: &str,
    token_hash: &str,
    n: U128,
) -> bool {
    true
}

fn transfer_dtoken(
    from_account: &Address,
    to_account: &Address,
    resource_id: &str,
    templates: &BTreeMap<String, bool>,
    n: U128,
) -> bool {
    true
}

fn generate_dtoken(
    account: &Address,
    resource_id: &str,
    templates: &BTreeMap<String, bool>,
    n: U128,
) -> bool {
    true
}

#[no_mangle]
pub fn invoke() {
    let input = runtime::input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"dtokenSellerPublish" => {
            let (resource_id, resource_ddo, item) = source.read().unwrap();
            sink.write(dtoken_seller_publish(resource_id, &resource_ddo, &item));
        }
        b"buyDtokenFromReseller" => {
            let (resource_id, n, buyer_account, reseller_account) = source.read().unwrap();
            sink.write(buy_dtoken_from_reseller(
                resource_id,
                n,
                buyer_account,
                reseller_account,
            ));
        }
        b"buyDtoken" => {
            let (resource_id, n, buyer_account) = source.read().unwrap();
            sink.write(buy_dtoken(resource_id, n, buyer_account));
        }
        b"useToken" => {
            let (resource_id, account, token_hash, n) = source.read().unwrap();
            sink.write(use_token(resource_id, account, token_hash, n));
        }
        b"useTokenByAgent" => {
            let (resource_id, account, agent, token_hash, n) = source.read().unwrap();
            sink.write(use_token_by_agent(
                resource_id,
                account,
                agent,
                token_hash,
                n,
            ));
        }
        b"setDtokenAgents" => {
            let (resource_id, account, agents, n) = source.read().unwrap();
            sink.write(set_dtoken_agents(resource_id, account, agents, n));
        }
        b"addDtokenAgents" => {
            let (resource_id, account, agents, n) = source.read().unwrap();
            sink.write(add_dtoken_agents(resource_id, account, agents, n));
        }
        b"removeDtokenAgents" => {
            let (resource_id, account, agents) = source.read().unwrap();
            sink.write(remove_dtoken_agents(resource_id, account, agents));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("not support method:{}", method)
        }
    }
    runtime::ret(sink.bytes());
}

mod events {
    use super::*;
    use ostd::macros::event;
    #[event(dtokenSellerPublishEvent)]
    pub fn dtoken_seller_publish_event(resource_id: &str, resource_ddo: &[u8], item: &[u8]) {}
    #[event(buyDtokenFromReseller)]
    pub fn buy_dtoken_from_reseller(
        resource_id: &str,
        n: U128,
        buyer_account: &Address,
        reseller_account: &Address,
    ) {
    }
}

mod utils {
    use super::*;
    use alloc::vec::Vec;
    pub fn generate_seller_item_info_key(resource_id: &str) -> Vec<u8> {
        [KEY_SELLER_ITEM_INFO, resource_id.as_bytes()].concat()
    }
    pub fn generate_seller_item_sold_key(resource_id: &str) -> Vec<u8> {
        [KEY_SELLER_ITEM_SOLD, resource_id.as_bytes()].concat()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
