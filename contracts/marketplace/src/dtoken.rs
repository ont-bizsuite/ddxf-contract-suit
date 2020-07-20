use super::get_dtoken_contract;
use super::ostd::contract::wasm;
use super::ostd::prelude::*;
use super::{verify_result, Address, U128};
use ontio_std::runtime::address;

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

pub fn verify_auth(dtokens_contract_addr: &Option<Vec<Address>>, token_template_ids: &[Vec<u8>]) {
    if let Some(dtokens) = dtokens_contract_addr {
        let l = dtokens.len();
        for i in 0..l {
            assert!(verify_creator_sig(
                dtokens.get(i).unwrap(),
                token_template_ids.get(i).unwrap()
            ));
            let self_addr = address();
            assert!(auth_token_template(
                dtokens.get(i).unwrap(),
                token_template_ids.get(i).unwrap(),
                &self_addr,
            ));
        }
    } else {
        let dtoken = get_dtoken_contract();
        assert!(verify_creator_sig_multi(&dtoken, token_template_ids));
        let self_addr = address();
        assert!(auth_token_template_multi(
            &dtoken,
            token_template_ids,
            &self_addr,
        ));
    }
}

pub fn auth_token_template_multi(
    dtoken: &Address,
    token_template_ids: &[Vec<u8>],
    authorized_addr: &Address,
) -> bool {
    verify_result(wasm::call_contract(
        dtoken,
        (
            "authorizeTokenTemplateMulti",
            (token_template_ids, authorized_addr),
        ),
    ));
    true
}

pub fn auth_token_template(
    dtoken: &Address,
    token_template_id: &[u8],
    authorized_addr: &Address,
) -> bool {
    verify_result(wasm::call_contract(
        dtoken,
        (
            "authorizeTokenTemplate",
            (token_template_id, authorized_addr),
        ),
    ));
    true
}

pub fn transfer_dtoken(
    dtokens: &Option<Vec<Address>>,
    token_template_ids: &[Vec<u8>],
    reseller_account: &Address,
    buyer_account: &Address,
    n: U128,
) {
    if let Some(d) = dtokens {
        let l = d.len();
        for i in 0..l {
            let token_template_id = token_template_ids.get(i).unwrap();
            assert!(transfer_dtoken_inner(
                d.get(i).unwrap(),
                reseller_account,
                buyer_account,
                token_template_id,
                n
            ));
        }
    } else {
        let dtoken = get_dtoken_contract();
        assert!(transfer_dtoken_multi(
            &dtoken,
            reseller_account,
            buyer_account,
            token_template_ids,
            n
        ));
    }
}

fn transfer_dtoken_inner(
    contract_address: &Address,
    from_account: &Address,
    to_account: &Address,
    token_template_id: &[u8],
    n: U128,
) -> bool {
    verify_result(wasm::call_contract(
        contract_address,
        (
            "transferDToken",
            (from_account, to_account, token_template_id, n),
        ),
    ));
    true
}

fn transfer_dtoken_multi(
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
    dtokens: &Option<Vec<Address>>,
    token_template_ids: &[Vec<u8>],
    buyer_account: &Address,
    n: U128,
) {
    if let Some(dtoken_addr) = dtokens {
        let l = dtoken_addr.len();
        for i in 0..l {
            assert!(generate_dtoken_inner(
                &dtoken_addr[i],
                buyer_account,
                token_template_ids.get(i).unwrap(),
                n
            ));
        }
    } else {
        let dtoken = get_dtoken_contract();
        assert!(generate_dtoken_multi(
            &dtoken,
            buyer_account,
            token_template_ids,
            n
        ));
    }
}
fn generate_dtoken_inner(
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

fn generate_dtoken_multi(
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
