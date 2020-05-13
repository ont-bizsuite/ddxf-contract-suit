use super::ostd::abi::VmValueBuilder;
use super::ostd::contract::{neo, wasm};
use super::ostd::runtime;
use super::BTreeMap;
use super::{Address, Sink, String, Vec, H256, U128};
use common::TokenTemplate;

pub fn remove_agents(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        contract_address,
        ("removeAgents", (account, resource_id, sink.bytes())),
    );
    true
}

pub fn set_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        contract_address,
        ("setTokenAgents", (account, resource_id, sink.bytes(), n)),
    );
    true
}

pub fn use_token_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        ("useToken", (account, resource_id, token_template, n)),
    );
    true
}

pub fn add_dtoken_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &str,
    token_hash: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        contract_address,
        (
            "addTokenAgents",
            (account, resource_id, token_hash, sink.bytes(), n),
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
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        contract_address,
        ("addAgents", (account, resource_id, sink.bytes(), n)),
    );
    true
}

pub fn add_token_agents_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_hash: &[u8],
    agents: Vec<&Address>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    wasm::call_contract(
        contract_address,
        (
            "addTokenAgents",
            (account, resource_id, token_hash, sink.bytes(), n),
        ),
    );
    true
}

pub fn use_token_by_agent_dtoken(
    contract_address: &Address,
    account: &Address,
    agent: &Address,
    resource_id: &[u8],
    token_template: TokenTemplate,
    n: U128,
) -> bool {
    wasm::call_contract(
        contract_address,
        (
            "useTokenByAgent",
            (account, agent, resource_id, token_template, n),
        ),
    );
    true
}

pub fn transfer_dtoken(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates: &Vec<TokenTemplate>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_templates(templates, &mut sink);
    wasm::call_contract(
        contract_address,
        (
            "transferDToken",
            (from_account, to_account, resource_id, sink.bytes(), n),
        ),
    );
    true
}

pub fn generate_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    templates: &[TokenTemplate],
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_templates(templates, &mut sink);
    wasm::call_contract(
        contract_address,
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
