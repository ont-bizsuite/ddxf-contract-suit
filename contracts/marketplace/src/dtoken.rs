use super::ostd::contract::wasm;
use super::ostd::prelude::*;
use super::{verify_result, Address, U128};

pub fn verify_creator_sig_multi(dtoken: &Address, token_template_ids: &[Vec<u8>]) -> bool {
    verify_result(wasm::call_contract(
        dtoken,
        ("verifyCreatorSigMulti", (token_template_ids)),
    ));
    true
}

pub fn verify_creator_sig(dtoken: &Address, token_template_id: &[u8]) -> bool {
    verify_result(wasm::call_contract(
        dtoken,
        ("verifyCreatorSig", (token_template_id,)),
    ));
    true
}

pub fn transfer_dtoken(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    token_template_id: &[u8],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        (
            "transferDtoken",
            (from_account, to_account, token_template_id, n),
        ),
    ));
    true
}

pub fn transfer_dtoken_multi(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    token_template_ids: &[Vec<u8>],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        (
            "transferDTokenMulti",
            (from_account, to_account, token_template_ids, n),
        ),
    ));
    true
}

pub fn generate_dtoken(
    contract_address: &Address,
    account: &Address,
    token_template_id: &[u8],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        ("generateDToken", (account, token_template_id, n)),
    ));
    true
}

pub fn generate_dtoken_multi(
    contract_address: &Address,
    account: &Address,
    token_template_ids: &[Vec<u8>],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        ("generateDTokenMulti", (account, token_template_ids, n)),
    ));
    true
}
