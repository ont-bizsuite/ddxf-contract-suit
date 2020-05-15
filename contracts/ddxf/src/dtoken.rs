use super::ostd::contract::wasm;
use super::DEFAULT_DTOKEN_CONTRACT_ADDRESS;
use super::{Address, Sink, Vec, U128};
use common::TokenTemplate;

pub fn remove_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "removeAgents",
            (account, resource_id, agents, templates_bytes),
        ),
    );
    true
}

pub fn remove_token_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    template_bytes: &[u8],
    agents: Vec<&Address>,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "removeAgents",
            (account, resource_id, agents, template_bytes),
        ),
    );
    true
}

pub fn set_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
    token_templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "setAgents",
            (account, resource_id, agents, n, token_templates_bytes),
        ),
    );
    true
}

pub fn set_token_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "setTokenAgents",
            (account, resource_id, token_template_bytes, agents, n),
        ),
    );
    true
}

pub fn use_token_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("useToken", (account, resource_id, token_template_bytes, n)),
    );
    true
}

pub fn add_token_agents_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "addTokenAgents",
            (account, resource_id, token_template_bytes, agents, n),
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
    token_templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "addAgents",
            (account, resource_id, agents, n, token_templates_bytes),
        ),
    );
    true
}

pub fn use_token_by_agent_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "useTokenByAgent",
            (account, agent, resource_id, token_template_bytes, n),
        ),
    );
    true
}

pub fn transfer_dtoken(
    contract_address: &Option<Address>,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        (
            "transferDToken",
            (from_account, to_account, resource_id, templates_bytes, n),
        ),
    );
    true
}

pub fn generate_dtoken(
    contract_address: &Option<Address>,
    account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        &contract_address.unwrap_or(DEFAULT_DTOKEN_CONTRACT_ADDRESS),
        ("generateDToken", (account, resource_id, templates_bytes, n)),
    );
    true
}
