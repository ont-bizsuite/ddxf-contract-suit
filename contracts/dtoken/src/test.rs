use super::ostd::types::Address;
use super::TokenTemplates;
use super::*;

#[test]
fn generate_dtoken_test() {
    let account = Address::repeat_byte(1);
    let resource_id = b"resource_id";
    let token_hash = b"token_hash";
    let templates = TokenTemplates::new(token_hash);
    let n = 10;
    assert!(generate_dtoken(&account, resource_id, templates.clone(), n));

    assert!(use_token(&account, resource_id, token_hash, 1));

    let agent = Address::repeat_byte(2);
    let agents: Vec<Address> = vec![agent.clone()];

    assert!(set_agent(&account, resource_id, agents.clone(), n));

    assert!(use_token_by_agent(
        &account,
        &agent,
        resource_id,
        token_hash,
        1
    ));

    let to_account = Address::repeat_byte(3);
    assert!(transfer_dtoken(
        &account,
        &to_account,
        resource_id,
        templates.clone(),
        1
    ));

    assert!(set_token_agents(
        &account,
        resource_id,
        token_hash,
        agents,
        1
    ));

    let agent2 = Address::repeat_byte(4);
    let agents2: Vec<Address> = vec![agent2.clone()];
    assert!(add_agents(&account, resource_id, agents2.clone(), 1));

    assert!(add_token_agents(
        &account,
        resource_id,
        token_hash,
        agents2.clone(),
        1
    ));

    assert!(remove_agents(&account, resource_id, agents2.clone()));
    assert!(remove_token_agents(
        &account,
        resource_id,
        token_hash,
        agents2
    ));
}
