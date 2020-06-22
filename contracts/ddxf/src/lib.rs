//! ddxf contract
//!
//! Seller publishing process
//!
//! seller only need invoke [`dtoken_seller_publish`](fn.dtoken_seller_publish.html) method to publish products
//!
//! Buyer purchase and use process
//!
//! first of all, buyers should invoke [`buy_dtoken`](fn.buy_dtoken.html) method to buy the released products
//!
//! Second, buyer invoke the [`use_token`](fn.use_token.html) method to consume token

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
use ostd::contract::wasm;
use ostd::runtime::{check_witness, contract_migrate, current_txhash};

#[cfg(test)]
mod test;

const SHA256_SIZE: u32 = 32;
const CRC32_SIZE: u32 = 4;

const KEY_SELLER_ITEM_INFO: &[u8] = b"01";
const KEY_DTOKEN_CONTRACT: &[u8] = b"03";
const KEY_SPLIT_POLICY_CONTRACT: &[u8] = b"04";
const KEY_ADMIN: &[u8] = b"05";

//AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP
//AYnhakv7kC9R5ppw65JoE2rt6xDzCjCTvD
const ADMIN: Address = ostd::macros::base58!("Aejfo7ZX5PVpenRj23yChnyH64nf8T1zbu");
const DEFAULT_SPLIT_CONTRACT: Address = ostd::macros::base58!("ANNA5KBoRaY2PbKuHYZJCedZ6NkpJN2tYh");
const DEFAULT_DTOKEN_CONTRACT: Address =
    ostd::macros::base58!("AR91qGoLrLqhb1CJ5MztpwfpdDveAEDJU5");

/// set dtoken contract address as the default dtoken contract address,
/// ddxf contract will invoke dtoken contract to pay the fee
/// need admin signature
/// `new_addr` is the new dtoken contract address
pub fn set_dtoken_contract(new_addr: &Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(KEY_DTOKEN_CONTRACT, new_addr);
    true
}

/// init contract
/// set dtoken and split contract address
pub fn init(dtoken: Address, split_policy: Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(KEY_DTOKEN_CONTRACT, dtoken);
    database::put(KEY_SPLIT_POLICY_CONTRACT, split_policy);
    true
}

/// query the default dtoken contract address
pub fn get_dtoken_contract() -> Address {
    database::get::<_, Address>(KEY_DTOKEN_CONTRACT).unwrap_or(DEFAULT_DTOKEN_CONTRACT)
}

/// set split contract address as the default split contract address,
///
/// When there are multiple data owners, split contract is used to set the income distribution strategy.
///
/// need admin signature
pub fn set_split_policy_contract(new_addr: &Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(KEY_SPLIT_POLICY_CONTRACT, new_addr);
    true
}

/// query the default split contract address
fn get_split_policy_contract() -> Address {
    database::get::<_, Address>(KEY_SPLIT_POLICY_CONTRACT).unwrap_or(DEFAULT_SPLIT_CONTRACT)
}

/// need old admin signature
///
/// update the admin address, admin has the right to set the default dtoken and split contract address
fn update_admin(new_admin: &Address) -> bool {
    let old_admin = get_admin();
    assert!(check_witness(&old_admin));
    database::put(KEY_ADMIN, new_admin);
    true
}

/// query admin address
fn get_admin() -> Address {
    database::get::<_, Address>(KEY_ADMIN).unwrap_or(ADMIN)
}

/// seller publish product, need seller signature
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `resource_ddo_bytes` is the result of ResourceDDO struct serialization
///
/// `item_bytes` is the result of DTokenItem struct serialization
///
/// `split_policy_param_bytes` is the result of RegisterParam struct serialization
///
/// # Example
/// ```no_run
/// use common::{Fee,TokenType};
/// let resource_id = b"resource_id";
/// let ddo = ResourceDDO {
///        token_resource_ty_endpoints: vec![],
///        item_meta_hash: H256::repeat_byte(1),
///        manager: manager.clone(),
///        dtoken_contract_address: Some(dtoken_contract_address.clone()),
///        mp_contract_address: None,
///        split_policy_contract_address: None,
///    };
/// let contract_addr = Address::repeat_byte(4);
///    let fee = Fee {
///        contract_addr,
///        contract_type: TokenType::ONG,
///        count: 0,
///    };
///    let mut templates = vec![];
///    templates.push(token_template.clone());
///    let dtoken_item = DTokenItem {
///        fee,
///        expired_date: 1,
///        stocks: 1,
///        templates,
///    };
///  let split_param = b"test";
///  assert!(supper::dtoken_seller_publish(
///        resource_id,
///        &ddo.to_bytes(),
///        &dtoken_item.to_bytes(),
///        split_param
///    ));
/// ```
pub fn dtoken_seller_publish(
    resource_id: &[u8],
    resource_ddo_bytes: &[u8],
    item_bytes: &[u8],
    split_policy_param_bytes: &[u8],
) -> bool {
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

    //invoke split_policy contract
    let split_addr = get_split_policy_contract();
    let res = wasm::call_contract(
        &resource_ddo
            .split_policy_contract_address
            .unwrap_or(split_addr),
        ("register", (resource_id, split_policy_param_bytes)),
    );
    if let Some(r) = res {
        let mut source = Source::new(r.as_slice());
        let rr: bool = source.read().unwrap();
        assert!(rr);
    } else {
        panic!("call split contract register failed");
    }

    //event
    let mut sink = Sink::new(16);
    sink.write(resource_ddo);
    let mut sink2 = Sink::new(16);
    sink2.write(item);
    events::dtoken_seller_publish_event(resource_id, sink.bytes(), sink2.bytes());
    true
}

fn freeze(resource_id: &[u8]) -> bool {
    let mut item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(check_witness(&item_info.resource_ddo.manager));
    item_info.resource_ddo.is_freeze = true;
    if item_info.item.sold == 0 {
        database::delete(utils::generate_seller_item_info_key(resource_id));
    } else {
        database::put(utils::generate_seller_item_info_key(resource_id), item_info);
    }
    EventBuilder::new()
        .string("freeze")
        .bytearray(resource_id)
        .notify();
    true
}

pub fn get_seller_item_info(resource_id: &[u8]) -> Vec<u8> {
    let r = runtime::storage_read(utils::generate_seller_item_info_key(resource_id).as_slice())
        .map(|val: Vec<u8>| val);
    if let Some(rr) = r {
        rr
    } else {
        vec![]
    }
}

/// buy dtoken from reseller
///
/// The seller can sell what he bought before he used it
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `n` is the number of purchases
///
/// `buyer_account` is buyer address, need this address signature
///
/// `reseller_account` is reseller address, need this address signature
pub fn buy_dtoken_from_reseller(
    resource_id: &[u8],
    n: U128,
    buyer_account: &Address,
    reseller_account: &Address,
) -> bool {
    assert!(runtime::check_witness(buyer_account) && runtime::check_witness(reseller_account));
    let item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    let oi = OrderId {
        item_id: resource_id.to_vec(),
        tx_hash: current_txhash(),
    };
    let split_contract = get_split_policy_contract();
    assert!(transfer_fee(
        &oi,
        buyer_account,
        item_info.resource_ddo.mp_contract_address,
        &item_info
            .resource_ddo
            .split_policy_contract_address
            .unwrap_or(split_contract),
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

/// Buy more than one dtoken at a time
///
/// `resource_ids` is array of resource_id which used to mark the only commodity in the chain
///
/// `ns` is array of n which is the number of purchases. the length of resource_ids must be the same with the length of ns.
///
/// `buyer_account` is buyer address, need this address signature
pub fn buy_dtokens(
    resource_ids: Vec<&[u8]>,
    ns: Vec<U128>,
    buyer_account: &Address,
    payer: &Address,
) -> bool {
    let l = resource_ids.len();
    assert_eq!(l, ns.len());
    for i in 0..l {
        assert!(buy_dtoken(resource_ids[i], ns[i], buyer_account, payer));
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
    payer: &Address,
    agent: &Address,
) -> bool {
    let l = resource_ids.len();
    assert_eq!(l, ns.len());
    for i in 0..l {
        assert!(buy_dtoken(resource_ids[i], ns[i], buyer_account, payer));
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

pub fn buy_use_token(
    resource_id: &[u8],
    n: U128,
    buyer_account: &Address,
    payer: &Address,
    token_template_bytes: &[u8],
) -> bool {
    assert!(buy_dtoken(resource_id, n, buyer_account, payer));
    assert!(use_token(
        resource_id,
        buyer_account,
        token_template_bytes,
        n
    ));
    true
}

/// buy dtoken
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `n` is the number of purchases
///
/// `buyer_account` is buyer address, need this address signature
pub fn buy_dtoken(resource_id: &[u8], n: U128, buyer_account: &Address, payer: &Address) -> bool {
    assert!(runtime::check_witness(buyer_account) && runtime::check_witness(payer));
    let mut item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(item_info.resource_ddo.is_freeze == false);
    let now = runtime::timestamp();
    assert!(now < item_info.item.expired_date);

    assert!(item_info.item.sold < item_info.item.stocks);
    item_info.item.sold = n.checked_add(item_info.item.sold as U128).unwrap() as u32;
    assert!(item_info.item.sold <= item_info.item.stocks);
    let oi = OrderId {
        item_id: resource_id.to_vec(),
        tx_hash: current_txhash(),
    };
    assert!(transfer_fee(
        &oi,
        payer,
        item_info.resource_ddo.mp_contract_address.clone(),
        &item_info
            .resource_ddo
            .split_policy_contract_address
            .unwrap_or(get_split_policy_contract()),
        item_info.item.fee.clone(),
        n
    ));
    database::put(
        utils::generate_seller_item_info_key(resource_id),
        &item_info,
    );
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
        .address(payer)
        .notify();
    true
}

/// buy_dtoken_reward
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `n` is the number of purchases
///
/// `buyer_account` is buyer address, need this address signature
/// `payer` is the address who pay the fee
/// `unit_price` unit price the buyer is willing to pay
pub fn buy_dtoken_reward(
    resource_id: &[u8],
    n: U128,
    buyer_account: &Address,
    payer: &Address,
    unit_price: U128,
) -> bool {
    assert!(runtime::check_witness(buyer_account) && runtime::check_witness(payer));
    let mut item_info =
        database::get::<_, SellerItemInfo>(utils::generate_seller_item_info_key(resource_id))
            .unwrap();
    assert!(item_info.resource_ddo.is_freeze == false);
    assert!(item_info.item.fee.count == 0);
    let now = runtime::timestamp();
    assert!(now < item_info.item.expired_date);

    assert!(item_info.item.sold < item_info.item.stocks);
    item_info.item.sold = n.checked_add(item_info.item.sold as U128).unwrap() as u32;
    assert!(item_info.item.sold <= item_info.item.stocks);
    let oi = OrderId {
        item_id: resource_id.to_vec(),
        tx_hash: current_txhash(),
    };
    let mut fee = item_info.item.fee.clone();
    fee.count = unit_price as u64;
    assert!(transfer_fee(
        &oi,
        payer,
        item_info.resource_ddo.mp_contract_address.clone(),
        &item_info
            .resource_ddo
            .split_policy_contract_address
            .unwrap_or(get_split_policy_contract()),
        fee,
        n
    ));
    database::put(
        utils::generate_seller_item_info_key(resource_id),
        &item_info,
    );
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
        .address(payer)
        .notify();
    true
}

/// use dtoken, after buy dtoken, user can consume the token
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is the address who consume token, need the address signature
///
/// `token_template_bytes` used to mark the only token user consume
///
/// `n` is the number of consuming
pub fn use_token(
    resource_id: &[u8],
    account: &Address,
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
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
        .bytearray(token_template_bytes)
        .notify();
    true
}

/// consume token by agent
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is buyer address, need the address signature
///
/// `agent` is the agent address who is authored to consume token
///
/// `token_template_bytes` used to mark the only token user consume
///
/// `n` is the number of consuming
pub fn use_token_by_agent(
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

/// set agent
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is user address who authorize the other address is agent, need account signature
///
/// `agent` is the array of agent address
///
/// `n` is number of authorizations per agent
pub fn set_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>, n: U128) -> bool {
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

/// set token agents, this method will clear the old agents
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account is` user address who authorize the other address is agent, need account signature
///
/// `agents` is the array of agent address
///
/// `template_bytes` used to mark the only token user consume
///
/// `n` is number of authorizations per agent
pub fn set_token_agents(
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

/// add_agents, this method only append agents for the all token
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is user address who authorize the other address is agent, need account signature
///
/// `agents` is the array of agent address
///
/// `n` is number of authorizations per agent
pub fn add_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>, n: U128) -> bool {
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

/// add_agents, this method only append agents for the specified token.
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is user address who authorize the other address is agent, need account signature
///
/// `token_template_bytes` used to specified which token to set agents.
///
/// `agents` is the array of agent address
///
/// `n` is number of authorizations per agent
pub fn add_token_agents(
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

/// product owner remove all the agents
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is user address who authorize the other address is agent, need account signature
///
/// `agents` is the array of agent address which will be removed by account
pub fn remove_agents(resource_id: &[u8], account: &Address, agents: Vec<&Address>) -> bool {
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

/// product owner remove the agents of specified token
///
/// `resource_id` used to mark the only commodity in the chain
///
/// `account` is user address who authorize the other address is agent, need account signature
///
/// `agents` is the array of agent address which will be removed by account
pub fn remove_token_agents(
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

/// upgrade contract
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

// inner method
fn transfer_fee(
    oi: &OrderId,
    buyer_account: &Address,
    mp_contract_address: Option<Address>,
    split_contract_address: &Address,
    fee: Fee,
    n: U128,
) -> bool {
    let res = match mp_contract_address {
        Some(mp_addr) => wasm::call_contract(
            &mp_addr,
            (
                "transferAmount",
                (oi.to_bytes(), buyer_account, split_contract_address, fee, n),
            ),
        ),
        _ => {
            let amt = n.checked_mul(fee.count as U128).unwrap();
            wasm::call_contract(
                split_contract_address,
                (
                    "transferWithdraw",
                    (buyer_account, oi.item_id.as_slice(), amt),
                ),
            )
        }
    };
    verify_result(res);
    true
}

fn verify_result(res: Option<Vec<u8>>) {
    if let Some(r) = res {
        let mut source = Source::new(r.as_slice());
        let r: bool = source.read().unwrap();
        assert!(r);
    } else {
        panic!("call contract failed")
    }
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
        b"init" => {
            let (dtoken, split_policy) = source.read().unwrap();
            sink.write(init(dtoken, split_policy));
        }
        b"setSplitPolicyContract" => {
            let new_addr = source.read().unwrap();
            sink.write(set_split_policy_contract(new_addr));
        }
        b"getSplitPolicyContract" => sink.write(get_split_policy_contract()),
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
            let (resource_id, resource_ddo, item, split_policy_param_bytes) =
                source.read().unwrap();
            sink.write(dtoken_seller_publish(
                resource_id,
                resource_ddo,
                item,
                split_policy_param_bytes,
            ));
        }
        b"getSellerItemInfo" => {
            let resource_id = source.read().unwrap();
            sink.write(get_seller_item_info(resource_id))
        }
        b"freeze" => {
            let resource_id = source.read().unwrap();
            sink.write(freeze(resource_id));
        }
        b"freezeAndPublish" => {
            let (old_resource_id, new_resource_id, resource_ddo, item, split_policy_param_bytes) =
                source.read().unwrap();
            sink.write(freeze_and_publish(
                old_resource_id,
                new_resource_id,
                resource_ddo,
                item,
                split_policy_param_bytes,
            ));
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
            let (resource_ids, ns, buyer, payer) = source.read().unwrap();
            sink.write(buy_dtokens(resource_ids, ns, buyer, payer));
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
                payer,
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
                payer,
                agent,
            ));
        }
        b"buyDtoken" => {
            let (resource_id, n, buyer_account, payer) = source.read().unwrap();
            sink.write(buy_dtoken(resource_id, n, buyer_account, payer));
        }
        b"buyDtokenReward" => {
            let (resource_id, n, buyer_account, payer, unit_price) = source.read().unwrap();
            sink.write(buy_dtoken_reward(
                resource_id,
                n,
                buyer_account,
                payer,
                unit_price,
            ));
        }
        b"buyAndUseToken" => {
            let (resource_id, n, buyer_account, payer, token_template_bytes) =
                source.read().unwrap();
            sink.write(buy_use_token(
                resource_id,
                n,
                buyer_account,
                payer,
                token_template_bytes,
            ));
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
