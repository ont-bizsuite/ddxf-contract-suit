#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate common;
extern crate ontio_std as ostd;
use alloc::collections::btree_map::BTreeMap;
use common::*;
use ostd::abi::{Decoder, Encoder, Error, EventBuilder, Sink, Source};
use ostd::database;
use ostd::prelude::H256;
use ostd::prelude::*;
use ostd::runtime;
use ostd::types::{Address, U128};
mod basic;
use basic::*;
use ostd::runtime::check_witness;

#[cfg(test)]
mod test;

const KEY_DTOKEN: &[u8] = b"01";
const KEY_AGENT: &[u8] = b"02";
const KEY_ACCOUNT_DTOKENS: &[u8] = b"03";
const KEY_DDXF_CONTRACT: &[u8] = b"04";

const ADMIN: Address = ostd::macros::base58!("AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP");

fn set_ddxf_contract(new_addr: &Address) -> bool {
    assert!(check_witness(&ADMIN));
    database::put(KEY_DDXF_CONTRACT, new_addr);
    true
}

fn get_ddxf_contract() -> Address {
    database::get(KEY_DDXF_CONTRACT).unwrap()
}

fn generate_dtoken(account: &Address, resource_id: &[u8], templates_bytes: &[u8], n: U128) -> bool {
    let mut source = Source::new(templates_bytes);
    let templates: Vec<TokenTemplate> = source.read().unwrap();
    check_caller();
    assert!(runtime::check_witness(account));
    for token_template in templates.iter() {
        let key = token_template.to_bytes();
        let mut caa = get_count_and_agent(resource_id, account, &key);
        caa.count += n as u32;
        update_count(
            resource_id,
            account,
            &token_template.to_bytes(),
            caa.clone(),
        );
    }
    EventBuilder::new()
        .string("generateDToken")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

pub fn use_token(
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template_bytes);
    assert!(caa.count >= n as u32);
    caa.count -= n as u32;
    let key = utils::generate_dtoken_key(resource_id, account, token_template_bytes);
    if caa.count == 0 {
        database::delete(key);
    } else {
        database::put(key, caa);
    }
    EventBuilder::new()
        .string("useToken")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

pub fn use_token_by_agent(
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template_bytes);
    assert!(caa.count >= n as u32);
    let agent_count = caa.agents.get_mut(agent).unwrap();
    assert!(*agent_count >= n as u32);
    if caa.count == n as u32 && *agent_count == n as u32 {
        database::delete(utils::generate_dtoken_key(
            resource_id,
            account,
            token_template_bytes,
        ));
    } else {
        caa.count -= n as u32;
        *agent_count -= n as u32;
        update_count(resource_id, account, token_template_bytes, caa);
    }
    EventBuilder::new()
        .string("useTokenByAgent")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn transfer_dtoken(
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    check_caller();
    let mut source = Source::new(templates_bytes);
    let templates: Vec<TokenTemplate> = source.read().unwrap();
    for token_template in templates.iter() {
        let template_bytes = token_template.to_bytes();
        let mut from_caa = get_count_and_agent(resource_id, from_account, &template_bytes);
        assert!(from_caa.count >= n as u32);
        from_caa.count -= n as u32;
        update_count(resource_id, from_account, &template_bytes, from_caa);
        let mut to_caa = get_count_and_agent(resource_id, to_account, &template_bytes);
        to_caa.count += n as u32;
        update_count(resource_id, to_account, &template_bytes, to_caa);
    }
    true
}

fn set_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    n: U128,
    token_templates_bytes: &[u8],
) -> bool {
    let mut source = Source::new(token_templates_bytes);
    let token_templates: Vec<TokenTemplate> = source.read().unwrap();
    check_caller();
    for token_template in token_templates.iter() {
        assert!(set_token_agents(
            account,
            resource_id,
            &token_template.to_bytes(),
            agents.clone(),
            n
        ));
    }
    true
}

fn set_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: Vec<Address>,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template_bytes);
    caa.set_token_agents(agents.as_slice(), n);
    update_count(resource_id, account, token_template_bytes, caa);
    EventBuilder::new()
        .string("setTokenAgents")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn add_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    n: U128,
    token_templates_bytes: &[u8],
) -> bool {
    let mut source = Source::new(token_templates_bytes);
    let token_templates: Vec<TokenTemplate> = source.read().unwrap();
    check_caller();
    for token_template in token_templates.iter() {
        assert!(add_token_agents(
            account,
            resource_id,
            &token_template.to_bytes(),
            &agents,
            n
        ));
    }
    true
}

fn add_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: &[Address],
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template_bytes);
    caa.add_agents(agents, n as u32);
    update_count(resource_id, account, token_template_bytes, caa);
    EventBuilder::new()
        .string("addTokenAgents")
        .bytearray(resource_id)
        .address(account)
        .number(n)
        .notify();
    true
}

fn remove_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    token_templates_bytes: &[u8],
) -> bool {
    let mut source = Source::new(token_templates_bytes);
    let token_templates: Vec<TokenTemplate> = source.read().unwrap();
    check_caller();
    for token_template in token_templates.iter() {
        assert!(remove_token_agents(
            account,
            resource_id,
            &token_template.to_bytes(),
            agents.as_slice()
        ));
    }
    true
}

fn remove_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: &[Address],
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template_bytes);
    caa.remove_agents(agents);
    update_count(resource_id, account, token_template_bytes, caa);
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
    assert!(check_witness(&ADMIN));
    let new_addr = runtime::contract_migrate(code, vm_type, name, version, author, email, desc);
    let empty_addr = Address::new([0u8; 20]);
    assert_ne!(new_addr, empty_addr);
    true
}

fn check_caller() {
    let caller = runtime::caller();
    let ddxf = get_ddxf_contract();
    assert!(caller == ddxf);
}

fn get_count_and_agent(
    resource_id: &[u8],
    account: &Address,
    token_template_bytes: &[u8],
) -> CountAndAgent {
    let key = utils::generate_dtoken_key(resource_id, account, token_template_bytes);
    database::get::<_, CountAndAgent>(&key).unwrap_or(CountAndAgent::new(account.clone()))
}

fn update_count(resource_id: &[u8], account: &Address, token_template: &[u8], caa: CountAndAgent) {
    let key = utils::generate_dtoken_key(resource_id, account, token_template);
    database::put(key, caa);
}

#[no_mangle]
pub fn invoke() {
    let input = runtime::input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"setDdxfContract" => {
            let new_addr = source.read().unwrap();
            sink.write(set_ddxf_contract(new_addr));
        }
        b"getDdxfContract" => {
            sink.write(get_ddxf_contract());
        }
        b"migrate" => {
            let (code, vm_type, name, version, author, email, desc) = source.read().unwrap();
            sink.write(migrate(code, vm_type, name, version, author, email, desc));
        }
        b"generateDToken" => {
            let (account, resource_id, templates, n) = source.read().unwrap();
            sink.write(generate_dtoken(account, resource_id, templates, n));
        }
        b"getCountAndAgent" => {
            let (resource_id, account, token_template) = source.read().unwrap();
            sink.write(get_count_and_agent(resource_id, account, token_template));
        }
        b"useToken" => {
            let (account, resource_id, token_template, n) = source.read().unwrap();
            sink.write(use_token(account, resource_id, token_template, n));
        }
        b"useTokenByAgent" => {
            let (account, agent, resource_id, token_template, n) = source.read().unwrap();
            sink.write(use_token_by_agent(
                account,
                agent,
                resource_id,
                token_template,
                n,
            ));
        }
        b"transferDtoken" => {
            let (from_account, to_account, resource_id, templates_bytes, n) =
                source.read().unwrap();
            sink.write(transfer_dtoken(
                from_account,
                to_account,
                resource_id,
                templates_bytes,
                n,
            ));
        }
        b"setAgents" => {
            let (account, resource_id, agents, n, token_templates) = source.read().unwrap();
            sink.write(set_agents(account, resource_id, agents, n, token_templates));
        }
        b"setTokenAgents" => {
            let (account, resource_id, token_template, agents, n) = source.read().unwrap();
            sink.write(set_token_agents(
                account,
                resource_id,
                token_template,
                agents,
                n,
            ));
        }
        b"addAgents" => {
            let (account, resource_id, agents, n, token_templates) = source.read().unwrap();
            sink.write(add_agents(account, resource_id, agents, n, token_templates));
        }
        b"addTokenAgents" => {
            let (account, resource_id, token_template, agents, n): (
                &Address,
                &[u8],
                &[u8],
                Vec<Address>,
                U128,
            ) = source.read().unwrap();
            sink.write(add_token_agents(
                account,
                resource_id,
                token_template,
                agents.as_slice(),
                n,
            ));
        }
        b"removeAgents" => {
            let (account, resource_id, agents, token_templates) = source.read().unwrap();
            sink.write(remove_agents(account, resource_id, agents, token_templates));
        }
        b"removeTokenAgents" => {
            let (account, resource_id, token_template, agents): (
                &Address,
                &[u8],
                &[u8],
                Vec<Address>,
            ) = source.read().unwrap();
            sink.write(remove_token_agents(
                account,
                resource_id,
                token_template,
                agents.as_slice(),
            ));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("dtoken contract, not support method:{}", method)
        }
    }
    runtime::ret(sink.bytes());
}

mod utils {
    use super::*;
    use alloc::vec::Vec;
    pub fn generate_dtoken_key(
        resource_id: &[u8],
        account: &Address,
        token_template_bytes: &[u8],
    ) -> Vec<u8> {
        [
            KEY_DTOKEN,
            resource_id,
            account.as_ref(),
            token_template_bytes,
        ]
        .concat()
    }
    pub fn generate_agent_key(
        resource_id: &[u8],
        account: &Address,
        token_hash: &[u8],
        agent: &Address,
    ) -> Vec<u8> {
        [
            KEY_AGENT,
            resource_id,
            account.as_ref(),
            token_hash,
            agent.as_ref(),
        ]
        .concat()
    }
}
