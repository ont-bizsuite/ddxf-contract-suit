#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate common;
extern crate ontio_std as ostd;
use ostd::abi::{EventBuilder, Sink, Source};
use ostd::database;
use ostd::prelude::*;
use ostd::runtime;
use ostd::types::{Address, U128};
mod basic;
use basic::*;
mod dtoken;
use common::*;
use dtoken::*;
use ostd::contract::{ong, ont, wasm};
use ostd::runtime::{check_witness, contract_migrate};

#[cfg(test)]
mod test;

const SHA256_SIZE: u32 = 32;
const CRC32_SIZE: u32 = 4;

const KEY_SELLER_ITEM_INFO: &[u8] = b"01";
const KEY_SELLER_ITEM_SOLD: &[u8] = b"02";
const KEY_DTOKEN_CONTRACT: &[u8] = b"03";
const KEY_ADMIN: &[u8] = b"04";

const ADMIN: Address = ostd::macros::base58!("AYnhakv7kC9R5ppw65JoE2rt6xDzCjCTvD");

// need admin signature
fn set_dtoken_contract(new_addr: &Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(KEY_DTOKEN_CONTRACT, new_addr);
    true
}

fn get_dtoken_contract() -> Address {
    database::get::<_, Address>(KEY_DTOKEN_CONTRACT).unwrap()
}

// need old admin signature
fn update_admin(new_admin: &Address) -> bool {
    let old_admin = get_admin();
    assert!(check_witness(&old_admin));
    database::put(KEY_ADMIN, new_admin);
    true
}

fn get_admin() -> Address {
    database::get::<_, Address>(KEY_ADMIN).unwrap_or(ADMIN)
}

fn dtoken_seller_publish(resource_id: &[u8], resource_ddo_bytes: &[u8], item_bytes: &[u8]) -> bool {
    let resource_ddo = ResourceDDO::from_bytes(resource_ddo_bytes);
    let item = DTokenItem::from_bytes(item_bytes);
    assert!(runtime::check_witness(&resource_ddo.manager));
    let resource =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id));
    assert!(resource.is_none());
    assert_ne!(item.templates.len(), 0);
    for token_template in item.templates.iter() {
        for rt in resource_ddo.token_resource_ty_endpoints.iter() {
            match rt.resource_type {
                RT::Other => {
                    for token_hash in token_template.token_hash.iter() {
                        assert_eq!(token_hash.len() as u32, SHA256_SIZE);
                    }
                }
                RT::RTStaticFile => {
                    if token_template.data_id.is_none() {
                        for token_hash in token_template.token_hash.iter() {
                            assert_eq!(token_hash.len() as u32, SHA256_SIZE + CRC32_SIZE);
                        }
                    } else {
                        for token_hash in token_template.token_hash.iter() {
                            assert_eq!(token_hash.len() as u32, SHA256_SIZE);
                        }
                    }
                }
            }
        }
    }

    let seller = SellerItemInfo::new(item.clone(), resource_ddo.clone());
    database::put(utils::generate_seller_item_info_key(resource_id), seller);
    let mut sink = Sink::new(16);
    sink.write(resource_ddo);
    let mut sink2 = Sink::new(16);
    sink2.write(item);
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
        buyer_account,
        reseller_account,
        item_info.resource_ddo.mp_contract_address,
        item_info.resource_ddo.split_policy_contract_address,
        item_info.item.fee.clone(),
        n
    ));
    let dtoken = get_dtoken_contract();
    assert!(transfer_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        reseller_account,
        buyer_account,
        resource_id,
        &item_info.item.get_templates_bytes(),
        n
    ));
    EventBuilder::new()
        .string("buyDtokenFromReseller")
        .bytearray(resource_id)
        .number(n)
        .address(buyer_account)
        .address(reseller_account)
        .notify();
    true
}

fn buy_dtokens(resource_ids: Vec<&[u8]>, ns: Vec<U128>, buyer_account: &Address) -> bool {
    let l = resource_ids.len();
    assert_eq!(l, ns.len());
    for i in 0..l {
        assert!(buy_dtoken(resource_ids[i], ns[i], buyer_account));
    }
    true
}

fn buy_dtokens_and_set_agents(
    resource_ids: Vec<&[u8]>,
    ns: Vec<U128>,
    use_index: U128,
    authorized_index: U128,
    authorized_token_template_bytes: &[u8],
    use_template_bytes: &[u8],
    buyer_account: &Address,
    agent: &Address,
) -> bool {
    let l = resource_ids.len();
    assert_eq!(l, ns.len());
    for i in 0..l {
        assert!(buy_dtoken(resource_ids[i], ns[i], buyer_account));
    }
    assert!(set_token_agents(
        resource_ids[authorized_index as usize],
        buyer_account,
        vec![agent.clone()],
        authorized_token_template_bytes,
        ns[authorized_index as usize],
    ));
    assert!(use_token(
        resource_ids[use_index as usize],
        buyer_account,
        use_template_bytes,
        ns[use_index as usize]
    ));
    true
}

fn get_token_templates_endpoint(resource_id: &[u8]) -> Vec<u8> {
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let mut sink = Sink::new(64);
    sink.write(item_info.item.templates.len() as u32);
    for token in item_info.resource_ddo.token_resource_ty_endpoints.iter() {
        sink.write(&token.token_template);
        sink.write(&token.endpoint);
    }
    return sink.bytes().to_vec();
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
        buyer_account,
        &item_info.resource_ddo.manager,
        item_info.resource_ddo.mp_contract_address.clone(),
        item_info.resource_ddo.split_policy_contract_address,
        item_info.item.fee.clone(),
        n
    ));
    database::put(utils::generate_seller_item_sold_key(resource_id), sum);

    let dtoken = get_dtoken_contract();
    assert!(generate_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        buyer_account,
        resource_id,
        &item_info.item.get_templates_bytes(),
        n
    ));
    EventBuilder::new()
        .string("buyDtoken")
        .bytearray(resource_id)
        .number(n)
        .address(buyer_account)
        .notify();
    true
}

fn use_token(resource_id: &[u8], account: &Address, token_template_bytes: &[u8], n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(use_token_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        token_template_bytes,
        n
    ));
    EventBuilder::new()
        .string("useToken")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn use_token_by_agent(
    resource_id: &[u8],
    account: &Address,
    agent: &Address,
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    assert!(runtime::check_witness(agent));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(use_token_by_agent_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        agent,
        resource_id,
        token_template_bytes,
        n
    ));
    EventBuilder::new()
        .string("useTokenByAgent")
        .bytearray(resource_id)
        .address(account)
        .address(agent)
        .number(n)
        .notify();
    true
}

fn set_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(set_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        agents,
        n,
        &item_info.item.get_templates_bytes(),
    ));
    true
}

fn set_token_agents(
    resource_id: &[u8],
    account: &Address,
    agents: Vec<Address>,
    template_bytes: &[u8],
    n: U128,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(set_token_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        &template_bytes,
        agents.as_slice(),
        n,
    ));
    EventBuilder::new()
        .string("setTokenAgents")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn add_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>, n: U128) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(add_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        agents,
        n,
        &item_info.item.get_templates_bytes()
    ));
    EventBuilder::new()
        .string("addAgents")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn add_token_agents(
    resource_id: &[u8],
    account: &Address,
    token_template_bytes: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(add_token_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        token_template_bytes,
        agents,
        n
    ));
    EventBuilder::new()
        .string("addTokenAgents")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn remove_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(remove_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        agents,
        &item_info.item.get_templates_bytes()
    ));
    EventBuilder::new()
        .string("removeAgents")
        .bytearray(resource_id)
        .address(account)
        .notify();
    true
}

fn remove_token_agents(
    resource_id: &[u8],
    token_template_bytes: &[u8],
    account: &Address,
    agents: Vec<&Address>,
) -> bool {
    assert!(runtime::check_witness(account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let dtoken = get_dtoken_contract();
    assert!(remove_token_agents_dtoken(
        &item_info
            .resource_ddo
            .dtoken_contract_address
            .unwrap_or(dtoken),
        account,
        resource_id,
        token_template_bytes,
        agents,
    ));
    EventBuilder::new()
        .string("removeTokenAgents")
        .bytearray(resource_id)
        .address(account)
        .notify();
    true
}

fn migrate(
    code: &[u8],
    vm_type: u32,
    name: &str,
    version: &str,
    author: &str,
    email: &str,
    desc: &str,
) -> bool {
    let admin = get_admin();
    assert!(check_witness(&admin));
    let new_addr = contract_migrate(code, vm_type, name, version, author, email, desc);
    let empty_addr = Address::new([0u8; 20]);
    assert_ne!(new_addr, empty_addr);
    EventBuilder::new()
        .string("migrate")
        .address(&new_addr)
        .notify();
    true
}

fn transfer_fee(
    buyer_account: &Address,
    seller_account: &Address,
    mp_contract_address: Option<Address>,
    split_contract_address: Option<Address>,
    fee: Fee,
    n: U128,
) -> bool {
    if let Some(mp_addr) = mp_contract_address {
        wasm::call_contract(
            &mp_addr,
            ("transferAmount", (buyer_account, seller_account, fee, n)),
        );
    } else {
        let amt = n.checked_mul(fee.count as U128).unwrap();
        if let Some(split_contract_addr) = split_contract_address {
            assert!(transfer_inner(
                buyer_account,
                &split_contract_addr,
                amt,
                &fee.contract_type,
                Some(fee.contract_addr)
            ));
        } else {
            assert!(transfer_inner(
                buyer_account,
                seller_account,
                amt,
                &fee.contract_type,
                Some(fee.contract_addr)
            ));
        }
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
            let res =
                wasm::call_contract(&contract_address, ("transfer", (from, to, amt))).unwrap();
            let mut source = Source::new(&res);
            let r: bool = source.read().unwrap();
            assert!(r);
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
        b"updateAdmin" => {
            let new_admin = source.read().unwrap();
            sink.write(update_admin(&new_admin));
        }
        b"getAdmin" => {
            sink.write(get_admin());
        }
        b"setDTokenContract" => {
            let new_addr = source.read().unwrap();
            sink.write(set_dtoken_contract(&new_addr));
        }
        b"getDTokenContract" => {
            sink.write(get_dtoken_contract());
        }
        b"migrate" => {
            let (code, vm_type, name, version, author, email, desc) = source.read().unwrap();
            sink.write(migrate(code, vm_type, name, version, author, email, desc));
        }
        b"dtokenSellerPublish" => {
            let (resource_id, resource_ddo, item) = source.read().unwrap();
            sink.write(dtoken_seller_publish(resource_id, resource_ddo, item));
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
        b"buyDtokens" => {
            let (resource_ids, ns, buyer) = source.read().unwrap();
            sink.write(buy_dtokens(resource_ids, ns, buyer));
        }
        b"buyDtokensAndSetAgents" => {
            let (
                resource_ids,
                ns,
                use_index,
                authorized_index,
                authorized_token_template_bytes,
                use_template_bytes,
                buyer,
                agent,
            ) = source.read().unwrap();
            sink.write(buy_dtokens_and_set_agents(
                resource_ids,
                ns,
                use_index,
                authorized_index,
                authorized_token_template_bytes,
                use_template_bytes,
                buyer,
                agent,
            ));
        }
        b"buyDtoken" => {
            let (resource_id, n, buyer_account) = source.read().unwrap();
            sink.write(buy_dtoken(resource_id, n, buyer_account));
        }
        b"getTokenTemplates" => {
            let resource_id = source.read().unwrap();
            sink.write(get_token_templates_endpoint(resource_id));
        }
        b"useToken" => {
            let (resource_id, account, token_template, n) = source.read().unwrap();
            sink.write(use_token(resource_id, account, token_template, n));
        }
        b"useTokenByAgent" => {
            let (resource_id, account, agent, token_template, n) = source.read().unwrap();
            sink.write(use_token_by_agent(
                resource_id,
                account,
                agent,
                token_template,
                n,
            ));
        }
        b"setAgents" => {
            let (resource_id, account, agents, n) = source.read().unwrap();
            sink.write(set_agents(resource_id, account, agents, n));
        }
        b"setTokenAgents" => {
            let (resource_id, account, agents, template_bytes, n) = source.read().unwrap();
            sink.write(set_token_agents(
                resource_id,
                account,
                agents,
                template_bytes,
                n,
            ));
        }
        b"addAgents" => {
            let (resource_id, account, agents, n) = source.read().unwrap();
            sink.write(add_agents(resource_id, account, agents, n));
        }
        b"addTokenAgents" => {
            let (resource_id, account, token_template_bytes, agents, n) = source.read().unwrap();
            sink.write(add_token_agents(
                resource_id,
                account,
                token_template_bytes,
                agents,
                n,
            ));
        }
        b"removeAgents" => {
            let (resource_id, account, agents) = source.read().unwrap();
            sink.write(remove_agents(resource_id, account, agents));
        }
        b"removeTokenAgents" => {
            let (resource_id, template_bytes, account, agents) = source.read().unwrap();
            sink.write(remove_token_agents(
                resource_id,
                template_bytes,
                account,
                agents,
            ));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("ddxf contract, not support method:{}", method)
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
    #[event(buyDtoken)]
    pub fn buy_dtoken(resource_id: &[u8], n: U128, buyer_account: &Address) {}
    #[event(useToken)]
    pub fn use_token(resource_id: &[u8], account: &Address, token_template_bytes: &[u8], n: U128) {}
    #[event(useTokenByAgent)]
    pub fn use_token_by_agent(
        resource_id: &[u8],
        account: &Address,
        agent: &Address,
        token_template_bytes: &[u8],
        n: U128,
    ) {
    }
    #[event(setDtokenAgents)]
    pub fn set_agents(resource_id: &[u8], account: &Address, n: U128) {}
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
