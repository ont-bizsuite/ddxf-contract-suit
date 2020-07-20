use super::*;

pub fn register(
    split_contract_addr: &Address,
    resource_id: &[u8],
    split_policy_param_bytes: &[u8],
) -> bool {
    let res = wasm::call_contract(
        split_contract_addr,
        ("register", (resource_id, split_policy_param_bytes)),
    );
    if let Some(r) = res {
        let mut source = Source::new(r.as_slice());
        let rr: bool = source.read().unwrap();
        assert!(rr);
    } else {
        panic!("call split contract register failed");
    }
    true
}
