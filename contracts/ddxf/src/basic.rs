use super::ostd::abi::{Decoder, Encoder, Sink, Source};
use super::ostd::prelude::*;
use super::ostd::types::{Address, H256};
use common::{Fee, TokenTemplate, RT};

#[derive(Clone, Encoder, Decoder)]
pub struct TokenResourceTyEndpoint {
    pub token_template: TokenTemplate,
    pub resource_type: RT,
    pub endpoint: Vec<u8>,
}

#[derive(Clone, Encoder, Decoder)]
pub struct ResourceDDO {
    pub manager: Address,                                          // data owner
    pub token_resource_ty_endpoints: Vec<TokenResourceTyEndpoint>, // RT for tokens
    pub item_meta_hash: H256,
    pub dtoken_contract_address: Option<Address>, // can not be empty
    pub mp_contract_address: Option<Address>,     // can be empty
    pub split_policy_contract_address: Option<Address>, //can be empty
    pub is_freeze: bool,
}

impl ResourceDDO {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut source = Source::new(data);
        source.read().unwrap()
    }
    #[cfg(test)]
    pub fn to_bytes(&self) -> Vec<u8> {
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
    pub raw_stocks:u32,
    pub stocks: u32,
    pub templates: Vec<TokenTemplate>,
}

impl DTokenItem {
    pub fn get_templates_bytes(&self) -> Vec<u8> {
        let mut sink = Sink::new(16);
        sink.write(&self.templates);
        sink.bytes().to_vec()
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut source = Source::new(data);
        source.read().unwrap()
    }

    #[cfg(test)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut sink = Sink::new(16);
        sink.write(self);
        sink.bytes().to_vec()
    }
}
