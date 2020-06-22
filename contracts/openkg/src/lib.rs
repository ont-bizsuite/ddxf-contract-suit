#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate alloc;
extern crate ontio_std as ostd;
use ostd::abi::{EventBuilder, Sink, Source};
use ostd::contract::wasm;
use ostd::database;
use ostd::runtime;
use ostd::types::{Address, U128};

const MP_CONTRACT_ADDRESS: Address = ostd::macros::base58!("Aejfo7ZX5PVpenRj23yChnyH64nf8T1zbu");
const DTOKEN_CONTRACT_ADDRESS: Address =
    ostd::macros::base58!("Aejfo7ZX5PVpenRj23yChnyH64nf8T1zbu");

const KEY_MP_CONTRACT: &[u8] = b"01";
const KEY_DTOKEN_CONTRACT: &[u8] = b"02";

fn get_mp_contract_addr() -> Address {
    database::get::<_, Address>(KEY_MP_CONTRACT).unwrap_or(MP_CONTRACT_ADDRESS)
}

fn get_dtoken_contract_addr() -> Address {
    database::get::<_, Address>(KEY_DTOKEN_CONTRACT).unwrap_or(MP_CONTRACT_ADDRESS)
}

fn freeze_and_publish(
    old_resource_id: &[u8],
    new_resource_id: &[u8],
    resource_ddo_bytes: &[u8],
    item_bytes: &[u8],
    split_policy_param_bytes: &[u8],
) -> bool {
    let mp = get_mp_contract_addr();
    verify_result(wasm::call_contract(&mp, ("freeze", (old_resource_id,))));
    verify_result(wasm::call_contract(
        &mp,
        (
            "dtokenSellerPublish",
            (
                new_resource_id,
                resource_ddo_bytes,
                item_bytes,
                split_policy_param_bytes,
            ),
        ),
    ));
    true
}

pub fn buy_use_token(
    resource_id: &[u8],
    n: U128,
    buyer_account: &Address,
    payer: &Address,
    token_template_bytes: &[u8],
) -> bool {
    //call market place
    assert!(buy_dtoken(resource_id, n, buyer_account, payer));
    let mp = get_mp_contract_addr();
    verify_result(wasm::call_contract(
        &mp,
        ("buyDtoken", (resource_id, n, buyer_account, payer)),
    ));

    //call dtoken
    let dtoken = get_dtoken_contract_addr();
    verify_result(wasm::call_contract(
        &dtoken,
        (
            "useToken",
            (resource_id, buyer_account, token_template_bytes, n),
        ),
    ));
    true
}

fn verify_result(res: Option<Vec<u8>>) {
    if let Some(r) = res {
        let mut source = Source::new(r.as_slice());
        let r: bool = source.read().unwrap();
        assert!(r);
    } else {
        panic!("call contract failed")
    }
}

#[no_mangle]
fn invoke() {
    let input = runtime::input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"freezeAndPublish" => {
            let (old_resource_id, new_resource_id, resource_ddo, item, split_policy_param_bytes) =
                source.read().unwrap();
            sink.write(freeze_and_publish(
                old_resource_id,
                new_resource_id,
                resource_ddo,
                item,
                split_policy_param_bytes,
            ));
        }
        b"buyAndUseToken" => {
            let (resource_id, n, buyer_account, payer, token_template_bytes) =
                source.read().unwrap();
            sink.write(buy_use_token(
                resource_id,
                n,
                buyer_account,
                payer,
                token_template_bytes,
            ));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("openkg contract, not support method:{}", method)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
