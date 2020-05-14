#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate common;
extern crate ontio_std as ostd;
use alloc::collections::btree_map::BTreeMap;
use common::*;
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::database;
use ostd::prelude::H256;
use ostd::prelude::*;
use ostd::runtime;
use ostd::types::{Address, U128};
mod basic;
use basic::*;
#[cfg(test)]
mod test;

const KEY_DTOKEN: &[u8] = b"01";
const KEY_AGENT: &[u8] = b"02";
const KEY_ACCOUNT_DTOKENS: &[u8] = b"03";

const DDXF_CONTRACT_ADDRESS: Address = ostd::macros::base58!("AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP");

fn generate_dtoken(
    account: &Address,
    resource_id: &[u8],
    templates: Vec<TokenTemplate>,
    n: U128,
) -> bool {
    check_caller();
    assert!(runtime::check_witness(account));
    for token_template in templates.iter() {
        let mut caa = get_count_and_agent(resource_id, account, token_template);
        caa.count += n as u32;
        update_count(resource_id, account, token_template, caa);
    }
    true
}

fn use_token(
    account: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, &token_template);
    assert!(caa.count >= n as u32);
    caa.count -= n as u32;
    let key = utils::generate_dtoken_key(resource_id, account, &token_template);
    if caa.count == 0 {
        database::delete(key);
    } else {
        database::put(key, caa);
    }
    true
}

fn use_token_by_agent(
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, &token_template);
    assert!(caa.count >= n as u32);
    let agent_count = caa.agents.get_mut(agent).unwrap();
    assert!(*agent_count >= n as u32);
    if caa.count == n as u32 && *agent_count == n as u32 {
        database::delete(utils::generate_dtoken_key(
            resource_id,
            account,
            &token_template,
        ));
    } else {
        caa.count -= n as u32;
        *agent_count -= n as u32;
        update_count(resource_id, account, &token_template, caa);
    }
    true
}

fn transfer_dtoken(
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates: Vec<TokenTemplate>,
    n: U128,
) -> bool {
    check_caller();
    for token_template in templates.iter() {
        let mut from_caa = get_count_and_agent(resource_id, from_account, &token_template);
        assert!(from_caa.count >= n as u32);
        from_caa.count -= n as u32;
        update_count(resource_id, from_account, token_template, from_caa);
        let mut to_caa = get_count_and_agent(resource_id, to_account, &token_template);
        to_caa.count += n as u32;
        update_count(resource_id, to_account, token_template, to_caa);
    }
    true
}

fn set_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    n: U128,
    token_templates: Vec<TokenTemplate>,
) -> bool {
    check_caller();
    for token_template in token_templates.iter() {
        assert!(set_token_agents(
            account,
            resource_id,
            token_template.clone(),
            agents.clone(),
            n
        ));
    }
    true
}

fn set_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    agents: Vec<Address>,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, &token_template);
    caa.set_token_agents(agents.as_slice(), n);
    update_count(resource_id, account, &token_template, caa);
    true
}

fn add_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    n: U128,
    token_templates: Vec<TokenTemplate>,
) -> bool {
    check_caller();
    for token_template in token_templates.iter() {
        let mut caa = get_count_and_agent(resource_id, account, token_template);
        caa.add_agents(agents.as_slice(), n as u32);
        update_count(resource_id, account, token_template, caa);
    }
    true
}

fn add_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    agents: Vec<Address>,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, &token_template);
    caa.add_agents(agents.as_slice(), n as u32);
    update_count(resource_id, account, &token_template, caa);
    true
}

fn remove_agents(
    account: &Address,
    resource_id: &[u8],
    agents: Vec<Address>,
    token_templates: Vec<TokenTemplate>,
) -> bool {
    check_caller();
    for token_template in token_templates.iter() {
        assert!(remove_token_agents(
            account,
            resource_id,
            token_template,
            agents.as_slice()
        ));
    }
    true
}

fn remove_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_template: &TokenTemplate,
    agents: &[Address],
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_template);
    caa.remove_agents(agents);
    update_count(resource_id, account, token_template, caa);
    true
}

fn check_caller() {
    //    let caller = runtime::caller();
    //    assert!(caller == DDXF_CONTRACT_ADDRESS);
}

fn get_count_and_agent<'a>(
    resource_id: &[u8],
    account: &Address,
    token_template: &TokenTemplate,
) -> CountAndAgent {
    database::get::<_, CountAndAgent>(utils::generate_dtoken_key(
        resource_id,
        account,
        token_template,
    ))
    .unwrap_or(CountAndAgent::new(account.clone()))
}

fn update_count(
    resource_id: &[u8],
    account: &Address,
    token_template: &TokenTemplate,
    caa: CountAndAgent,
) {
    database::put(
        utils::generate_dtoken_key(resource_id, account, token_template),
        caa,
    );
}

#[no_mangle]
pub fn invoke() {
    let input = runtime::input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"generateDtoken" => {
            let (account, resource_id, templates, n) = source.read().unwrap();
            sink.write(generate_dtoken(account, resource_id, templates, n));
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
            let (from_account, to_account, resource_id, account, n) = source.read().unwrap();
            sink.write(transfer_dtoken(
                from_account,
                to_account,
                resource_id,
                account,
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
            let (account, resource_id, token_template, agents, n) = source.read().unwrap();
            sink.write(add_token_agents(
                account,
                resource_id,
                token_template,
                agents,
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
                TokenTemplate,
                Vec<Address>,
            ) = source.read().unwrap();
            sink.write(remove_token_agents(
                account,
                resource_id,
                &token_template,
                agents.as_slice(),
            ));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("not support method:{}", method)
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
        token_template: &TokenTemplate,
    ) -> Vec<u8> {
        [
            KEY_DTOKEN,
            resource_id,
            account.as_ref(),
            &token_template.serialize(),
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
