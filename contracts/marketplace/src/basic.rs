use super::ostd::abi::{Decoder, Encoder, Source};
use super::ostd::prelude::*;
use super::ostd::types::{Address, H256};
use common::Fee;

#[derive(Clone, Encoder, Decoder)]
pub struct ResourceDDO {
    pub manager: Address, // data owner
    pub item_meta_hash: H256,
    pub dtoken_contract_address: Option<Vec<Address>>, // can be empty
    pub accountant_contract_address: Option<Address>,  // can be empty
    pub split_policy_contract_address: Option<Address>, //can be empty
}

impl ResourceDDO {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut source = Source::new(data);
        source.read().unwrap()
    }
    #[cfg(test)]
    pub fn to_bytes(&self) -> Vec<u8> {
        use super::ostd::abi::Sink;
        let mut sink = Sink::new(16);
        sink.write(self);
        sink.bytes().to_vec()
    }
}

#[derive(Encoder, Decoder, Clone)]
pub struct SellerItemInfo {
    pub item: DTokenItem,
    pub resource_ddo: ResourceDDO,
}

impl SellerItemInfo {
    pub fn new(item: DTokenItem, resource_ddo: ResourceDDO) -> Self {
        SellerItemInfo { item, resource_ddo }
    }
}

#[derive(Clone, Encoder, Decoder)]
pub struct DTokenItem {
    pub fee: Fee,
    pub expired_date: u64,
    pub stocks: u64,
    pub sold: u64,
    pub token_template_ids: Vec<Vec<u8>>,
}

impl DTokenItem {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut source = Source::new(data);
        source.read().unwrap()
    }

    #[cfg(test)]
    pub fn to_bytes(&self) -> Vec<u8> {
        use super::ostd::abi::Sink;
        let mut sink = Sink::new(16);
        sink.write(self);
        sink.bytes().to_vec()
    }
}
