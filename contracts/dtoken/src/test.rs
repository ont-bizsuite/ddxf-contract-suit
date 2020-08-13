use super::ostd::types::Address;
use super::*;
use hexutil::{read_hex, to_hex};
use ostd::mock::build_runtime;

#[test]
fn test_create_tt() {
    let handle = build_runtime();
    let creator = Address::repeat_byte(1);
    handle.witness(&[creator.clone()]);
    let tt = TokenTemplate::new(
        b"name".to_vec(),
        b"symbol".to_vec(),
        None,
        vec![],
        vec![0u8],
    );
    assert!(create_token_template(&creator, tt));
    let token_template_id = b"0";
    let authorized_addr = Address::repeat_byte(2);
    assert!(authorize_token_template(
        token_template_id,
        &[authorized_addr.clone()]
    ));
    let addr = get_authorized_addr(token_template_id);
    assert_eq!(addr.len(), 1);

    handle.witness(&[*CONTRACT_COMMON.admin(), creator.clone()]);
    assert!(set_mp_contract(CONTRACT_COMMON.admin()));
    assert!(generate_dtoken(&creator, token_template_id, 1000));
    let token_id = get_token_id_by_template_id(token_template_id);
    assert_eq!(token_id.as_slice(), b"0");
    let template_id = get_template_id_by_token_id(token_id.as_slice());
    assert_eq!(template_id.as_slice(), token_template_id);

    let ba = oep8::balance_of(&creator, token_id.as_slice());
    assert_eq!(ba, 1000);
    let to = Address::repeat_byte(4);
    handle.witness(&[creator.clone()]);
    assert!(oep8::transfer(&creator, &to, token_id.as_slice(), 100));
}

#[test]
fn generate_dtoken_test() {
    let account = Address::repeat_byte(1);
    let token_template_id = b"token_template_id";
    let token_hash = vec![0u8, 32];
    let template = TokenTemplate::new(
        b"name".to_vec(),
        b"symbol".to_vec(),
        None,
        vec![token_hash],
        vec![],
    );
    let templates = vec![template.clone()];
    let n = 10;

    let template_bytes = template.to_bytes();
    let templates_bytes = serialize_templates(&templates);

    let creator = Address::repeat_byte(9);
    let handle = build_runtime();
    handle.witness(&[account.clone(), creator.clone(), *CONTRACT_COMMON.admin()]);
    assert!(set_mp_contract(&creator));
    assert!(create_token_template(&creator, template_bytes.as_slice()));
    let token_template_id = "0".to_string().as_bytes().to_vec();

    let temp_acc = Address::repeat_byte(8);
    assert!(authorize_token_template(
        token_template_id.as_slice(),
        &[account.clone(), temp_acc.clone()]
    ));

    let addrs = get_authorized_addr(token_template_id.as_slice());
    assert_eq!(addrs.len(), 2);

    assert!(remove_authorize_addr(
        token_template_id.as_slice(),
        &[temp_acc]
    ));
    let addrs = get_authorized_addr(token_template_id.as_slice());
    assert_eq!(addrs.len(), 1);

    assert!(generate_dtoken(&account, token_template_id.as_slice(), n));

    let token_id = "0".to_string().as_bytes().to_vec();
    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba as U128, n);

    assert!(use_token(&account, &token_id, 1));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba as U128, n - 1);

    let agent = Address::repeat_byte(2);
    let agents: Vec<Address> = vec![agent.clone()];

    assert!(set_agents(
        &account,
        agents.clone(),
        vec![n - 1],
        vec![token_id.clone()]
    ));

    let caa = get_agent_balance(&agent, &token_id);
    assert_eq!(caa, n - 1);

    handle.witness(&[agent.clone()]);
    assert!(use_token_by_agent(&account, &agent, &token_id, 1));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba as U128, n - 1 - 1);
    let caa = get_agent_balance(&agent, &token_id);
    assert_eq!(caa, n - 1 - 1);

    let to_account = Address::repeat_byte(3);
    handle.witness(&[account.clone()]);
    assert!(oep8::transfer(&account, &to_account, &token_id, 1));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba as U128, n - 1 - 1 - 1);
    let caa = get_agent_balance(&agent, &token_id);
    assert_eq!(caa, n - 1 - 1);

    assert!(set_token_agents(
        &account,
        &token_id,
        agents.clone(),
        vec![1]
    ));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba, n - 1 - 1 - 1);
    let caa = get_agent_balance(&agent, &token_id);
    assert_eq!(caa, 1);

    let agent2 = Address::repeat_byte(4);
    let agents2: Vec<Address> = vec![agent2.clone()];

    assert!(add_agents(
        &account,
        agents2.clone(),
        vec![1],
        vec![token_id.clone()]
    ));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba, n - 1 - 1 - 1);
    let caa = get_agent_balance(&agent2, &token_id);
    assert_eq!(caa, 1);

    assert!(add_token_agents(&account, &token_id, &agents2, vec![1]));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba, n - 1 - 1 - 1);
    let caa = get_agent_balance(&agent2, &token_id);
    assert_eq!(caa, 2);

    assert!(remove_agents(
        &account,
        agents2.clone(),
        vec![token_id.clone()]
    ));

    let ba = oep8::balance_of(&account, token_id.as_slice());
    assert_eq!(ba, n - 1 - 1 - 1);
    let caa = get_agent_balance(&agent2, &token_id);
    assert_eq!(caa, 0);

    assert!(remove_token_agents(&account, &token_id, agents2.as_slice()));
}

fn serialize_templates(templates: &[TokenTemplate]) -> Vec<u8> {
    let mut sink = Sink::new(16);
    sink.write(templates);
    sink.bytes().to_vec()
}
