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

const SHA256_SIZE: u32 = 32;
const CRC32_SIZE: u32 = 4;

const KEY_SELLER_ITEM_INFO: &[u8] = b"01";
const KEY_SELLER_ITEM_SOLD: &[u8] = b"02";

#[derive(Clone)]
struct ResourceDDO {
    resource_type: RT,                        //0:RTStaticFile,
    manager: Address,                         // data owner id
    endpoint: String,                         // data service provider uri
    token_endpoint: BTreeMap<String, String>, // endpoint for tokens
    desc_hash: Option<H256>,                  // required if len(Templates) > 1
}

impl<'a> Encoder for ResourceDDO {
    fn encode(&self, sink: &mut Sink) {
        match self.resource_type {
            RT::RTStaticFile => {
                sink.write(0u8);
            }
        }
        sink.write(&self.manager);
        sink.write(&self.endpoint);
        sink.write(self.token_endpoint.len() as u32);
        for (k, v) in self.token_endpoint.iter() {
            sink.write(k);
            sink.write(v);
        }
        if let Some(desc_hash) = &self.desc_hash {
            sink.write(desc_hash);
        }
    }
}

impl<'a> Decoder<'a> for ResourceDDO {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let rt: u8 = source.read().unwrap();
        let manager: Address = source.read().unwrap();
        let endpoint: String = source.read().unwrap();
        let l: u32 = source.read().unwrap();
        let mut bmap: BTreeMap<String, String> = BTreeMap::new();
        for _ in 0..l {
            let k: String = source.read().unwrap();
            let v: String = source.read().unwrap();
            bmap.insert(k, v);
        }
        let resource_type = RT::RTStaticFile;
        if rt != 0 {
            panic!("");
        }
        let desc_hash = source.read().ok();
        Ok(ResourceDDO {
            resource_type,
            manager,
            endpoint,
            token_endpoint: bmap,
            desc_hash,
        })
    }
}

#[derive(Clone)]
enum RT {
    RTStaticFile,
}

#[derive(Encoder, Decoder, Clone)]
struct SellerItemInfo {
    item: DTokenItem,
    resource_ddo: ResourceDDO,
}

impl SellerItemInfo {
    fn new(item: DTokenItem, resource_ddo: ResourceDDO) -> Self {
        SellerItemInfo { item, resource_ddo }
    }
}

#[derive(Clone)]
struct DTokenItem {
    fee: Fee,
    expired_date: u64,
    stocks: u32,
    templates: BTreeMap<String, bool>,
}
impl Encoder for DTokenItem {
    fn encode(&self, sink: &mut Sink) {
        sink.write(&self.fee);
        sink.write(self.expired_date);
        sink.write(self.stocks);
        sink.write(self.templates.len() as u32);
        for (k, v) in self.templates.iter() {
            sink.write(k);
            sink.write(v);
        }
    }
}
impl<'a> Decoder<'a> for DTokenItem {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let fee: Fee = source.read()?;
        let expired_date: u64 = source.read()?;
        let stocks: u32 = source.read()?;
        let mut templates: BTreeMap<String, bool> = BTreeMap::new();
        let l: u32 = source.read()?;
        for _ in 0..l {
            let (k, v) = source.read()?;
            templates.insert(k, v);
        }
        Ok(DTokenItem {
            fee,
            expired_date,
            stocks,
            templates,
        })
    }
}

#[derive(Encoder, Decoder, Clone)]
struct Fee {
    contract_addr: Address,
    contract_type: TokenType,
    count: u64,
}

#[derive(Clone)]
enum TokenType {
    ONT,
    ONG,
    OEP4,
}

impl Encoder for TokenType {
    fn encode(&self, sink: &mut Sink) {
        match self {
            TokenType::ONT => {
                sink.write(0u8);
            }
            TokenType::ONG => {
                sink.write(1u8);
            }
            TokenType::OEP4 => {
                sink.write(2u8);
            }
        }
    }
}

impl<'a> Decoder<'a> for TokenType {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let ty: u8 = source.read().unwrap();
        match ty {
            0u8 => Ok(TokenType::ONT),
            1u8 => Ok(TokenType::ONG),
            2u8 => Ok(TokenType::OEP4),
            _ => {
                panic!("");
            }
        }
    }
}

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
    match resource_ddo.resource_type {
        RT::RTStaticFile => {
            for (token_hash, _) in item.templates.iter() {
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
