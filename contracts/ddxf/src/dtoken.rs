use super::ostd::abi::VmValueBuilder;
use super::ostd::contract::neo;
use super::BTreeMap;
use super::{Address, Sink, String, Vec, H256, U128};

pub fn remove_agents(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    agents: Vec<&Address>,
) -> bool {
    let mut sink = Sink::new(16);
    sink.write(agents);
    neo::call_contract(
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
    neo::call_contract(
        contract_address,
        ("setTokenAgents", (account, resource_id, sink.bytes(), n)),
    );
    true
}
pub fn use_token_dtoken(
    contract_address: &Address,
    account: &Address,
    resource_id: &[u8],
    token_hash: &H256,
    n: U128,
) -> bool {
    neo::call_contract(
        contract_address,
        ("useToken", (account, resource_id, token_hash, n)),
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
    neo::call_contract(
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
    neo::call_contract(
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
    neo::call_contract(
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
    token_hash: &[u8],
    n: U128,
) -> bool {
    neo::call_contract(
        contract_address,
        (
            "useTokenByAgent",
            (account, agent, resource_id, token_hash, n),
        ),
    );
    true
}

pub fn transfer_dtoken(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates: &BTreeMap<String, bool>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_template(templates, &mut sink);
    neo::call_contract(
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
    templates: &BTreeMap<String, bool>,
    n: U128,
) -> bool {
    let mut sink = Sink::new(16);
    serialize_template(templates, &mut sink);
    neo::call_contract(
        contract_address,
        ("generateDToken", (account, resource_id, sink.bytes(), n)),
    );
    true
}

fn serialize_template(templates: &BTreeMap<String, bool>, sink: &mut Sink) {
    let mut sink = Sink::new(16);
    let l = templates.len() as u32;
    sink.write(l);
    for (template, _) in templates.iter() {
        sink.write(template);
    }
}
