use super::*;
use ostd::mock::build_runtime;

#[test]
fn publish() {
    let resource_id = b"resource_id";

    let mut bmap: BTreeMap<String, RT> = BTreeMap::new();
    bmap.insert("token_hash".to_string(), RT::RTStaticFile);

    let manager = Address::repeat_byte(1);
    let dtoken_contract_address = Address::repeat_byte(2);
    let mp_contract_address = Address::repeat_byte(3);

    let ddo = ResourceDDO {
        resource_type: RT::RTStaticFile,
        token_resource_type: bmap,
        manager: manager.clone(),
        endpoint: "endpoint".to_string(),
        token_endpoint: BTreeMap::new(),
        desc_hash: None,
        dtoken_contract_address,
        mp_contract_address: Some(mp_contract_address),
    };
    let contract_addr = Address::repeat_byte(4);
    let fee = Fee {
        contract_addr,
        contract_type: TokenType::ONG,
        count: 0,
    };
    let dtoken_item = DTokenItem {
        fee,
        expired_date: 1,
        stocks: 1,
        templates: BTreeMap::new(),
    };

    let handle = build_runtime();
    handle.witness(&[manager]);
    assert!(dtoken_seller_publish(resource_id, &ddo, &dtoken_item));
}
