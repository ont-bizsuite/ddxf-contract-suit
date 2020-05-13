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
mod basic;
use basic::*;
mod dtoken;
use dtoken::*;
use ostd::contract::{neo, ong, ont};

#[cfg(test)]
mod test;

const SHA256_SIZE: u32 = 32;
const CRC32_SIZE: u32 = 4;

const KEY_SELLER_ITEM_INFO: &[u8] = b"01";
const KEY_SELLER_ITEM_SOLD: &[u8] = b"02";

fn dtoken_seller_publish(
    resource_id: &[u8],
    resource_ddo: &ResourceDDO,
    item: &DTokenItem,
) -> bool {
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
    resource_id: &[u8],
    n: U128,
    buyer_account: &Address,
    reseller_account: &Address,
) -> bool {
    assert!(runtime::check_witness(buyer_account) && runtime::check_witness(reseller_account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(transfer_fee(
        &item_info.resource_ddo.dtoken_contract_address,
        buyer_account,
        reseller_account,
        None,
        &item_info.item.fee,
        n
    ));
    assert!(transfer_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        reseller_account,
        buyer_account,
        resource_id,
        &item_info.item.templates,
        n
    ));
    true
}

fn buy_dtoken(resource_id: &[u8], n: U128, buyer_account: &Address) -> bool {
    assert!(runtime::check_witness(buyer_account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let now = runtime::timestamp();
    assert!(now < item_info.item.expired_date);
    let sold =
        database::get::<_, U128>(utils::generate_seller_item_sold_key(resource_id)).unwrap_or(0);
    assert!(sold < item_info.item.stocks as U128);
    let sum = sold.checked_add(n).unwrap();
    assert!(sum <= item_info.item.stocks as U128);
    assert!(transfer_fee(
        &item_info.resource_ddo.dtoken_contract_address,
        buyer_account,
        &item_info.resource_ddo.manager,
        item_info.resource_ddo.mp_contract_address.clone(),
        &item_info.item.fee,
        n
    ));
    database::put(utils::generate_seller_item_sold_key(resource_id), sum);
    assert!(generate_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        buyer_account,
        resource_id,
        &item_info.item.templates,
        n
    ));
    true
}

fn use_token(resource_id: &[u8], account: &Address, token_hash: &H256, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(use_token_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        resource_id,
        token_hash,
        n
    ));
    true
}

fn use_token_by_agent(
    resource_id: &[u8],
    account: &Address,
    agent: &Address,
    token_hash: &[u8],
    n: U128,
) -> bool {
    assert!(runtime::check_witness(agent));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(use_token_by_agent_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        agent,
        resource_id,
        token_hash,
        n
    ));
    true
}

fn set_dtoken_agents(
    resource_id: &[u8],
    account: &Address,
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    set_agents_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        resource_id,
        agents,
        n,
    );
    true
}

fn add_dtoken_agents(
    resource_id: &[u8],
    account: &Address,
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(add_agents_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        resource_id,
        agents,
        n
    ));
    true
}

fn add_token_agents(
    resource_id: &[u8],
    account: &Address,
    token_hash: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(add_token_agents_dtoken(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        resource_id,
        token_hash,
        agents,
        n
    ));
    true
}

fn remove_dtoken_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(remove_agents(
        &item_info.resource_ddo.dtoken_contract_address,
        account,
        resource_id,
        agents
    ));
    true
}
fn transfer_fee(
    contract_address: &Address,
    buyer_account: &Address,
    seller_account: &Address,
    mp_contract_address: Option<Address>,
    fee: &Fee,
    n: U128,
) -> bool {
    if let Some(mp_addr) = mp_contract_address {
        let mut sink = Sink::new(16);
        sink.write(fee);
        neo::call_contract(
            &mp_addr,
            (
                "transferAmount",
                (buyer_account, seller_account, sink.bytes(), n),
            ),
        );
    } else {
        let amt = n.checked_mul(fee.count as U128).unwrap();
        assert!(transfer_inner(
            buyer_account,
            seller_account,
            amt,
            &fee.contract_type,
            Some(fee.contract_addr)
        ));
    }
    true
}

fn transfer_inner(
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
            let res = neo::call_contract(&contract_address, ("transfer", (from, to, amt))).unwrap();
        }
    }
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
        b"addTokenAgents" => {
            let (resource_id, account, token_hash, agents, n) = source.read().unwrap();
            sink.write(add_token_agents(
                resource_id,
                account,
                token_hash,
                agents,
                n,
            ));
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
    pub fn dtoken_seller_publish_event(resource_id: &[u8], resource_ddo: &[u8], item: &[u8]) {}
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
    pub fn generate_seller_item_info_key(resource_id: &[u8]) -> Vec<u8> {
        [KEY_SELLER_ITEM_INFO, resource_id].concat()
    }
    pub fn generate_seller_item_sold_key(resource_id: &[u8]) -> Vec<u8> {
        [KEY_SELLER_ITEM_SOLD, resource_id].concat()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
