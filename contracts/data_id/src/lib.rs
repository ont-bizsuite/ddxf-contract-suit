#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std;
use ontio_std as ostd;
use ostd::abi::{Decoder, Encoder, Sink, Source};
use ostd::contract::ontid;
use ostd::contract::ontid::{DDOAttribute, Group, Signer};
use ostd::prelude::*;
use ostd::runtime::{input, ret};

extern crate common;
use common::CONTRACT_COMMON;
use ontio_std::abi::EventBuilder;

#[cfg(test)]
mod test;

#[derive(Encoder, Decoder)]
pub struct RegIdAddAttributesParam {
    ont_id: Vec<u8>,               // data_id
    group: Group,                  // Group contains all controllers's ont_id
    signer: Vec<Signer>, // Signer represents the ontid of the controller contained in the group, need signer's signature
    attributes: Vec<DDOAttribute>, //attributes user defined data information
}

impl RegIdAddAttributesParam {
    #[cfg(test)]
    fn from_bytes(data: &[u8]) -> RegIdAddAttributesParam {
        let mut source = Source::new(data);
        source.read().unwrap()
    }
}

/// register data_id and add_attribute for dataId
///
/// `reg_id_bytes` is array of RegIdAddAttributesParam
/// `RegIdAddAttributesParam` is defined as follow:
/// ```
/// use ostd::contract::ontid::{DDOAttribute, Group, Signer};
/// #[derive(Encoder, Decoder)]
///    struct RegIdAddAttributesParam {
///        ont_id: Vec<u8>,               // data_id
///        group: Group,                  // Group contains all controllers's ont_id
///        signer: Vec<Signer>,           // Signer represents the ontid of the controller contained in the group, need signer's signature
///        attributes: Vec<DDOAttribute>, //attributes user defined data information
///    }
/// ```
///
pub fn reg_id_add_attribute_array(reg_id_vec: &[RegIdAddAttributesParam]) -> bool {
    for reg_id in reg_id_vec.iter() {
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
    let mut sink = Sink::new(64);
    sink.write(reg_id_vec);
    EventBuilder::new()
        .string("reg_id_add_attribute_array")
        .bytearray(sink.bytes())
        .notify();
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
            CONTRACT_COMMON.destroy();
        }
        b"migrate" => {
            let (code, vm_type, name, version, author, email, desc) = source.read().unwrap();
            sink.write(CONTRACT_COMMON.migrate(code, vm_type, name, version, author, email, desc));
        }
        b"reg_id_add_attribute_array" => {
            let data_id: Vec<RegIdAddAttributesParam> = source.read().unwrap();
            sink.write(reg_id_add_attribute_array(data_id.as_slice()));
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
