use super::ostd::types::Address;
use super::*;
use hexutil::to_hex;
use ostd::mock::build_runtime;

#[test]
fn test2() {
    let account = Address::repeat_byte(1);
    let mut caa = CountAndAgent::new(account);
    caa.count += 1;
    let mut sink = Sink::new(16);
    sink.write(caa);
    println!("{}", to_hex(sink.bytes()));
}

#[test]
fn generate_dtoken_test() {
    let account = Address::repeat_byte(1);
    let resource_id = b"resource_id";
    let token_hash = vec![0u8, 32];
    let template = TokenTemplate::new(None, token_hash);
    let templates = vec![template.clone()];
    let n = 10;

    let template_bytes = template.to_bytes();
    let templates_bytes = serialize_templates(&templates);

    let handle = build_runtime();
    handle.witness(&[account.clone()]);
    assert!(generate_dtoken(&account, resource_id, &templates_bytes, n));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n);

    assert!(use_token(&account, resource_id, &template_bytes, 1));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1);

    let agent = Address::repeat_byte(2);
    let agents: Vec<Address> = vec![agent.clone()];

    assert!(set_agents(
        &account,
        resource_id,
        agents.clone(),
        n,
        &templates_bytes
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.agents.len() as U128, 1);
    assert_eq!(caa.agents[&agents.clone()[0]] as U128, n);

    assert!(use_token_by_agent(
        &account,
        &agent,
        resource_id,
        &template_bytes,
        1
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 1);
    assert_eq!(caa.agents[&agents.clone()[0]] as U128, n - 1);

    let to_account = Address::repeat_byte(3);
    assert!(transfer_dtoken(
        &account,
        &to_account,
        resource_id,
        &templates_bytes,
        1
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 1);
    assert_eq!(caa.agents[&agents.clone()[0]] as U128, n - 1);

    let caa = get_count_and_agent(resource_id, &to_account, &template_bytes);
    assert_eq!(caa.count as U128, 1);
    assert_eq!(caa.agents.len() as U128, 0);

    assert!(set_token_agents(
        &account,
        resource_id,
        &template_bytes,
        agents.clone(),
        1
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 1);
    assert_eq!(caa.agents[&agents.clone()[0]] as U128, 1);

    let agent2 = Address::repeat_byte(4);
    let agents2: Vec<Address> = vec![agent2.clone()];

    assert!(add_agents(
        &account,
        resource_id,
        agents2.clone(),
        1,
        &templates_bytes
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 2);
    assert_eq!(caa.agents[&agents2.clone()[0]] as U128, 1);

    assert!(add_token_agents(
        &account,
        resource_id,
        &template_bytes,
        agents2.clone(),
        1
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 2);
    assert_eq!(caa.agents[&agents2.clone()[0]] as U128, 2);

    assert!(remove_agents(
        &account,
        resource_id,
        agents2.clone(),
        &templates_bytes
    ));

    let caa = get_count_and_agent(resource_id, &account, &template_bytes);
    assert_eq!(caa.count as U128, n - 1 - 1 - 1);
    assert_eq!(caa.agents.len() as U128, 1);
    assert_eq!(caa.agents[&agents.clone()[0]] as U128, 1);

    assert!(remove_token_agents(
        &account,
        resource_id,
        &template_bytes,
        agents2.as_slice()
    ));
}

fn serialize_templates(templates: &[TokenTemplate]) -> Vec<u8> {
    let mut sink = Sink::new(16);
    sink.write(templates);
    sink.bytes().to_vec()
}
