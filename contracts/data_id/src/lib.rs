#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std;
use ontio_std as ostd;
use ontio_std::abi::EventBuilder;
use ontio_std::runtime::{check_witness, contract_migrate};
use ostd::abi::{Decoder, Encoder, Sink, Source};
use ostd::contract::ontid;
use ostd::contract::ontid::{DDOAttribute, Group, Signer};
use ostd::prelude::*;
use ostd::runtime::{contract_delete, input, ret};

#[cfg(test)]
mod test;

const ADMIN: Address = ostd::macros::base58!("Aejfo7ZX5PVpenRj23yChnyH64nf8T1zbu");

#[derive(Encoder, Decoder)]
struct RegIdAddAttributesParam {
    ont_id: Vec<u8>,
    group: Group,
    signer: Vec<Signer>,
    attributes: Vec<DDOAttribute>,
}

impl RegIdAddAttributesParam {
    fn from_bytes(data: &[u8]) -> RegIdAddAttributesParam {
        let mut source = Source::new(data);
        source.read().unwrap()
    }
}

fn destroy() {
    assert!(check_witness(&ADMIN));
    contract_delete();
}

/// upgrade contract
fn migrate(
    code: &[u8],
    vm_type: u32,
    name: &str,
    version: &str,
    author: &str,
    email: &str,
    desc: &str,
) -> bool {
    assert!(check_witness(&ADMIN));
    let new_addr = contract_migrate(code, vm_type, name, version, author, email, desc);
    let empty_addr = Address::new([0u8; 20]);
    assert_ne!(new_addr, empty_addr);
    EventBuilder::new()
        .string("migrate")
        .address(&new_addr)
        .notify();
    true
}

pub fn register_data_id_add_attribute_array(reg_id_bytes: Vec<Vec<u8>>) -> bool {
    for param_bytes in reg_id_bytes.iter() {
        let reg_id = RegIdAddAttributesParam::from_bytes(param_bytes.as_slice());
        assert!(ontid::reg_id_with_controller(
            reg_id.ont_id.as_slice(),
            &reg_id.group,
            reg_id.signer.as_slice()
        ));
        assert!(ontid::add_attributes_by_controller(
            reg_id.ont_id.as_slice(),
            reg_id.attributes.as_slice(),
            reg_id.signer.as_slice()
        ));
    }
    true
}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"destroy" => {
            destroy();
        }
        b"migrate" => {
            let (code, vm_type, name, version, author, email, desc) = source.read().unwrap();
            sink.write(migrate(code, vm_type, name, version, author, email, desc));
        }
        b"reg_id_add_attribute_array" => {
            let data_id_bytes: Vec<Vec<u8>> = source.read().unwrap();
            sink.write(register_data_id_add_attribute_array(data_id_bytes));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("data_id contract not support method:{}", method)
        }
    }
    ret(sink.bytes());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
