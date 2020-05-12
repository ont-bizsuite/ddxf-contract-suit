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
#[cfg(test)]
mod test;

const KEY_DTOKEN: &[u8] = b"01";
const KEY_AGENT: &[u8] = b"02";
const KEY_ACCOUNT_DTOKENS: &[u8] = b"03";

const DDXF_CONTRACT_ADDRESS: Address = ostd::macros::base58!("AbtTQJYKfQxq4UdygDsbLVjE8uRrJ2H3tP");

fn generate_dtoken(
    account: &Address,
    resource_id: &[u8],
    templates: TokenTemplates,
    n: U128,
) -> bool {
    check_caller();
    runtime::check_witness(account);
    let mut token_hashs =
        database::get::<_, Vec<&[u8]>>(utils::generate_account_dtokens_key(resource_id, account))
            .unwrap_or(vec![]);
    for (&token_hash, _) in templates.val.iter() {
        let mut caa = get_count_and_agent(resource_id, account, token_hash);
        caa.count += n as u32;
        update_count(resource_id, account, token_hash, caa);
        if !token_hashs.contains(&token_hash) {
            token_hashs.push(token_hash);
        }
    }

    database::put(
        utils::generate_account_dtokens_key(resource_id, account),
        token_hashs,
    );
    true
}

fn use_token(account: &Address, resource_id: &[u8], token_hash: &[u8], n: U128) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_hash);
    assert!(caa.count >= n as u32);
    caa.count -= n as u32;
    let key = utils::generate_dtoken_key(resource_id, account, token_hash);
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
    token_hash: &[u8],
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_hash);
    assert!(caa.count >= n as u32);
    let agent_count = caa.agents.get_mut(agent).unwrap();
    assert!(*agent_count >= n as u32);
    if caa.count == n as u32 && *agent_count == n as u32 {
        database::delete(utils::generate_dtoken_key(resource_id, account, token_hash));
    } else {
        caa.count -= n as u32;
        *agent_count -= n as u32;
        update_count(resource_id, account, token_hash, caa);
    }
    true
}

fn transfer_dtoken(
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates: TokenTemplates,
    n: U128,
) -> bool {
    check_caller();
    for (token_hash, v) in templates.val.iter() {
        let mut from_caa = get_count_and_agent(resource_id, from_account, token_hash);
        assert!(from_caa.count >= n as u32);
        from_caa.count -= n as u32;
        update_count(resource_id, from_account, token_hash, from_caa);
        let mut to_caa = get_count_and_agent(resource_id, to_account, token_hash);
        to_caa.count += n as u32;
        update_count(resource_id, to_account, token_hash, to_caa);
    }
    true
}

fn set_agent(account: &Address, resource_id: &[u8], agents: Vec<Address>, n: U128) -> bool {
    check_caller();
    let token_hashs = get_token_hashs(resource_id, account);
    for token_hash in token_hashs.iter() {
        assert!(set_token_agents(
            account,
            resource_id,
            token_hash,
            agents.clone(),
            n
        ));
    }
    true
}

fn set_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_hash: &[u8],
    agents: Vec<Address>,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_hash);
    caa.set_token_agents(agents.as_slice(), n);
    update_count(resource_id, account, token_hash, caa);
    true
}

fn add_agents(account: &Address, resource_id: &[u8], agents: Vec<Address>, n: U128) -> bool {
    check_caller();
    let token_hashs = get_token_hashs(resource_id, account);
    for token_hash in token_hashs.iter() {
        let mut caa = get_count_and_agent(resource_id, account, token_hash);
        caa.add_agents(agents.as_slice(), n as u32);
        update_count(resource_id, account, token_hash, caa);
    }
    true
}

fn add_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_hash: &[u8],
    agents: Vec<Address>,
    n: U128,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_hash);
    caa.add_agents(agents.as_slice(), n as u32);
    update_count(resource_id, account, token_hash, caa);
    true
}

fn remove_agents(account: &Address, resource_id: &[u8], agents: Vec<Address>) -> bool {
    check_caller();
    let token_hashs = get_token_hashs(resource_id, account);
    for token_hash in token_hashs.iter() {
        assert!(remove_token_agents(
            account,
            resource_id,
            token_hash,
            agents.clone()
        ));
    }
    true
}

fn remove_token_agents(
    account: &Address,
    resource_id: &[u8],
    token_hash: &[u8],
    agents: Vec<Address>,
) -> bool {
    check_caller();
    let mut caa = get_count_and_agent(resource_id, account, token_hash);
    caa.remove_agents(agents.as_slice());
    update_count(resource_id, account, token_hash, caa);
    true
}

fn get_token_hashs(resource_id: &[u8], account: &Address) -> Vec<Vec<u8>> {
    database::get::<_, Vec<Vec<u8>>>(utils::generate_account_dtokens_key(resource_id, account))
        .unwrap()
}

fn check_caller() {
    //    let caller = runtime::caller();
    //    assert!(caller == DDXF_CONTRACT_ADDRESS);
}

fn get_count_and_agent<'a>(
    resource_id: &[u8],
    account: &Address,
    token_hash: &[u8],
) -> CountAndAgent {
    database::get::<_, CountAndAgent>(utils::generate_dtoken_key(resource_id, account, token_hash))
        .unwrap_or(CountAndAgent::new(account.clone()))
}

fn update_count(resource_id: &[u8], account: &Address, token_hash: &[u8], caa: CountAndAgent) {
    database::put(
        utils::generate_dtoken_key(resource_id, account, token_hash),
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
    runtime::ret(sink.bytes());
}

mod utils {
    use super::*;
    use alloc::vec::Vec;
    pub fn generate_dtoken_key(
        resource_id: &[u8],
        account: &Address,
        token_hash: &[u8],
    ) -> Vec<u8> {
        [KEY_DTOKEN, resource_id, account.as_ref(), token_hash].concat()
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
    pub fn generate_account_dtokens_key(resource_id: &[u8], account: &Address) -> Vec<u8> {
        [KEY_AGENT, resource_id, account.as_ref()].concat()
    }
}
