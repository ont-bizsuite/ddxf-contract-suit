#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;
use ostd::abi::{Decoder, Encoder, Sink, Source};
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
    owner: Address,
}

fn register_data_id(info_bytes: &[u8]) -> bool {
    let mut source = Source::new(info_bytes);
    let data_id_info: DataIdInfo = source.read().unwrap();
    database::put(
        utils::generate_data_id_key(data_id_info.data_id.as_slice()),
        info_bytes,
    );
    true
}

fn get_data_id_info(id: Vec<u8>) -> DataIdInfo {
    database::get::<_, DataIdInfo>(utils::generate_data_id_key(id.as_slice())).unwrap()
}

fn check_owner(data_id: Vec<u8>) -> bool {
    let info = get_data_id_info(data_id);
    return check_witness(&info.owner);
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
            let data_id: Vec<u8> = source.read().unwrap();
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
