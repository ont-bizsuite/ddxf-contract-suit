use super::ostd::contract::wasm;
use super::{verify_result, Address, U128};

pub fn transfer_dtoken(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    resource_id: &[u8],
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        (
            "transferDToken",
            (from_account, to_account, resource_id, templates_bytes, n),
        ),
    ));
    true
}

pub fn generate_dtoken(
    contract_address: &Address,
    account: &Address,
    templates_bytes: &[u8],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        ("generateDToken", (account, templates_bytes, n)),
    ));
    true
}
