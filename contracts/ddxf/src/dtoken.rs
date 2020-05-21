use super::ostd::contract::wasm;
use super::{Address, Vec, U128};

pub fn remove_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "removeAgents",
            (account, resource_id, agents, templates_bytes),
        ),
    );
    true
}

pub fn remove_token_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    template_bytes: &[u8],
    agents: Vec<&Address>,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "removeAgents",
            (account, resource_id, agents, template_bytes),
        ),
    );
    true
}

pub fn set_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
    token_templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "setAgents",
            (account, resource_id, agents, n, token_templates_bytes),
        ),
    );
    true
}

pub fn set_token_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: &[Address],
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "setTokenAgents",
            (account, resource_id, token_template_bytes, agents, n),
        ),
    );
    true
}

pub fn use_token_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        ("useToken", (account, resource_id, token_template_bytes, n)),
    );
    true
}

pub fn add_token_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "addTokenAgents",
            (account, resource_id, token_template_bytes, agents, n),
        ),
    );
    true
}
pub fn add_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
    token_templates_bytes: &[u8],
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "addAgents",
            (account, resource_id, agents, n, token_templates_bytes),
        ),
    );
    true
}

pub fn use_token_by_agent_dtoken(
    contract_address: &Address,
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "useTokenByAgent",
            (account, agent, resource_id, token_template_bytes, n),
        ),
    );
    true
}

pub fn transfer_dtoken(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "transferDToken",
            (from_account, to_account, resource_id, templates_bytes, n),
        ),
    );
    true
}

pub fn generate_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        ("generateDToken", (account, resource_id, templates_bytes, n)),
    );
    true
}
