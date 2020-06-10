#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;
use ostd::abi::{Decoder, Encoder, EventBuilder, Sink, Source};
use ostd::database;
use ostd::prelude::*;
use ostd::runtime::{check_witness, input, ret};
use ostd::types::H256;
const KEY_DATA_ID: &[u8] = b"01";

#[derive(Encoder, Decoder)]
struct DataIdInfo {
    data_id: Vec<u8>,
    data_type: u8,
    data_meta_hash: H256,
    data_hash: H256,
    owners: Vec<Address>,
}

impl DataIdInfo {
    fn default() -> Self {
        DataIdInfo {
            data_id: vec![0u8],
            data_type: 0u8,
            data_meta_hash: H256::new([0u8; 32]),
            data_hash: H256::new([0u8; 32]),
            owners: vec![],
        }
    }
}

fn register_data_id(info_bytes: &[u8]) -> bool {
    let mut source = Source::new(info_bytes);
    let data_id_info: DataIdInfo = source.read().unwrap();
    assert!(data_id_info.owners.len() >= 1);
    let mut valid = false;
    for owner in data_id_info.owners.iter() {
        if valid {
            break;
        }
        valid = check_witness(owner);
    }
    assert!(valid);
    database::put(
        utils::generate_data_id_key(data_id_info.data_id.as_slice()),
        info_bytes,
    );
    EventBuilder::new()
        .string("registerDataId")
        .address(&data_id_info.owners[0])
        .bytearray(data_id_info.data_id.as_slice())
        .notify();
    true
}

fn get_data_id_info(id: Vec<u8>) -> DataIdInfo {
    database::get::<_, DataIdInfo>(utils::generate_data_id_key(id.as_slice()))
        .unwrap_or(DataIdInfo::default())
}

fn check_owner(data_id: Vec<u8>) -> bool {
    let info = get_data_id_info(data_id);
    let mut valid = false;
    for owner in info.owners.iter() {
        if valid {
            break;
        }
        valid = check_witness(owner);
    }
    assert!(valid);
    return true;
}

#[no_mangle]
pub fn invoke() {
    let input = input();
    let mut source = Source::new(&input);
    let action: &[u8] = source.read().unwrap();
    let mut sink = Sink::new(12);
    match action {
        b"registerDataId" => {
            let data_id_bytes: &[u8] = source.read().unwrap();
            sink.write(register_data_id(data_id_bytes));
        }
        b"get_data_id_info" => {
            let data_id: Vec<u8> = source.read().unwrap();
            sink.write(get_data_id_info(data_id));
        }
        b"check_owner" => {
            let data_id = source.read().unwrap();
            sink.write(check_owner(data_id));
        }
        _ => {
            let method = str::from_utf8(action).ok().unwrap();
            panic!("data_id contract not support method:{}", method)
        }
    }
    ret(sink.bytes());
}

mod utils {
    use super::*;
    pub fn generate_data_id_key(data_id: &[u8]) -> Vec<u8> {
        [KEY_DATA_ID, data_id].concat()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
