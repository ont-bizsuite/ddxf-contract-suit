use super::ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use super::ostd::prelude::*;
use super::ostd::types::{Address, H256};
use common::{Fee, TokenTemplate};

#[derive(Clone, Encoder, Decoder)]
pub struct TokenResourceTyEndpoint {
    pub token_template: TokenTemplate,
    pub resource_type: RT,
    pub endpoint: Vec<u8>,
}

#[derive(Clone, Encoder, Decoder)]
pub struct ResourceDDO {
    pub managers: Vec<Address>, // data owner id
    pub token_resource_ty_endpoints: Vec<TokenResourceTyEndpoint>, // RT for tokens
    pub item_meta_hash: H256,
    pub dtoken_contract_address: Option<Address>, // can not be empty
    pub mp_contract_address: Option<Address>,     // can be empty
    pub split_policy_contract_address: Option<Address>, //can be empty
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

#[derive(Clone)]
pub enum RT {
    Other,
    RTStaticFile,
}

impl Encoder for RT {
    fn encode(&self, sink: &mut Sink) {
        match self {
            RT::Other => {
                sink.write(0u8);
            }
            RT::RTStaticFile => {
                sink.write(1u8);
            }
        }
    }
}

impl<'a> Decoder<'a> for RT {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let u = source.read_byte()?;
        match u {
            0 => Ok(RT::Other),
            1 => Ok(RT::RTStaticFile),
            _ => panic!("not support rt:{}", u),
        }
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
