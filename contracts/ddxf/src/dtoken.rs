use super::ostd::contract::wasm;
use super::DEFAULT_DTOKEN_CONTRACT_ADDRESS;
use super::{Address, Sink, Vec, U128};
use common::TokenTemplate;

pub fn remove_agents(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    templates: &[TokenTemplate],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("removeAgents", (account, resource_id, agents, templates)),
    );
    true
}
pub fn set_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
    token_templates: &[TokenTemplate],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "setAgents",
            (account, resource_id, agents, n, token_templates),
        ),
    );
    true
}

pub fn set_token_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("setTokenAgents", (account, resource_id, agents, n)),
    );
    true
}

pub fn use_token_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("useToken", (account, resource_id, token_template, n)),
    );
    true
}

pub fn add_dtoken_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &str,
    token_hash: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "addTokenAgents",
            (account, resource_id, token_hash, agents, n),
        ),
    );
    true
}
pub fn add_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
    templates: &[TokenTemplate],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("addAgents", (account, resource_id, agents, n, templates)),
    );
    true
}

pub fn add_token_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    token_hash: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "addTokenAgents",
            (account, resource_id, token_hash, sink.bytes(), n),
        ),
    );
    true
}

pub fn use_token_by_agent_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "useTokenByAgent",
            (account, agent, resource_id, token_template, n),
        ),
    );
    true
}

pub fn transfer_dtoken(
    contract_address: &Option<Address>,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates: &Vec<TokenTemplate>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_templates(templates, &mut sink);
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "transferDToken",
            (from_account, to_account, resource_id, sink.bytes(), n),
        ),
    );
    true
}

pub fn generate_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    templates: &[TokenTemplate],
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_templates(templates, &mut sink);
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("generateDToken", (account, resource_id, sink.bytes(), n)),
    );
    true
}

fn serialize_templates(templates: &[TokenTemplate], sink: &mut Sink) {
    let mut sink = Sink::new(16);
    let l = templates.len() as u32;
    sink.write(l);
    for template in templates.iter() {
        sink.write(template);
    }
}
