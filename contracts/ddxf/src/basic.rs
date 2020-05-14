use super::ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use super::ostd::prelude::*;
use super::ostd::types::{Address, H256};
use super::BTreeMap;
use super::String;
use common::TokenTemplate;

#[derive(Clone)]
pub struct ResourceDDO {
    pub resource_type: RT,                                //0:RTStaticFile,
    pub token_resource_type: BTreeMap<TokenTemplate, RT>, // RT for tokens
    pub manager: Address,                                 // data owner id
    pub endpoint: String,                                 // data service provider uri
    pub token_endpoint: BTreeMap<TokenTemplate, String>,  // endpoint for tokens
    pub desc_hash: Option<H256>,                          // required if len(Templates) > 1
    pub dtoken_contract_address: Option<Address>,         // can not be empty
    pub mp_contract_address: Option<Address>,             // can be empty
    pub split_policy_contract_address: Option<Address>,   //can be empty
}

impl<'a> Encoder for ResourceDDO {
    fn encode(&self, sink: &mut Sink) {
        sink.write(self.resource_type.clone());
        let l = self.token_resource_type.len() as u32;
        sink.write(l);
        for (k, v) in self.token_resource_type.iter() {
            sink.write(k);
            sink.write(v);
        }
        sink.write(&self.manager);
        sink.write(&self.endpoint);
        sink.write(self.token_endpoint.len() as u32);
        for (k, v) in self.token_endpoint.iter() {
            sink.write(k);
            sink.write(v);
        }
        if let Some(desc_hash) = &self.desc_hash {
            sink.write(true);
            sink.write(desc_hash);
        } else {
            sink.write(false);
        }
        if let Some(addr) = &self.dtoken_contract_address {
            sink.write(true);
            sink.write(&self.dtoken_contract_address);
        } else {
            sink.write(false);
        }
        if let Some(mp_addr) = &self.mp_contract_address {
            sink.write(true);
            sink.write(mp_addr);
        } else {
            sink.write(false);
        }
        if let Some(split_contract_address) = &self.split_policy_contract_address {
            sink.write(true);
            sink.write(split_contract_address);
        } else {
            sink.write(false);
        }
    }
}

impl<'a> Decoder<'a> for ResourceDDO {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let resource_type = source.read()?;
        let l: u32 = source.read()?;
        let mut token_resource_type: BTreeMap<TokenTemplate, RT> = BTreeMap::new();
        for _ in 0..l {
            let (k, v) = source.read()?;
            token_resource_type.insert(k, v);
        }
        let manager: Address = source.read().unwrap();
        let endpoint: String = source.read().unwrap();
        let l: u32 = source.read().unwrap();
        let mut bmap: BTreeMap<TokenTemplate, String> = BTreeMap::new();
        for _ in 0..l {
            let k: TokenTemplate = source.read().unwrap();
            let v: String = source.read().unwrap();
            bmap.insert(k, v);
        }
        let is: bool = source.read()?;
        let desc_hash = match is {
            true => {
                let temp: H256 = source.read()?;
                Some(temp)
            }
            false => None,
        };
        let is: bool = source.read()?;
        let dtoken_contract_address = match is {
            true => source.read()?,
            false => None,
        };

        let is_val: bool = source.read()?;
        let mp_contract_address = match is_val {
            true => {
                let addr: Address = source.read()?;
                Some(addr)
            }
            false => None,
        };
        let is_val: bool = source.read()?;
        let split_policy_contract_address = match is_val {
            true => {
                let addr: Address = source.read()?;
                Some(addr)
            }
            false => None,
        };
        Ok(ResourceDDO {
            resource_type,
            token_resource_type,
            manager,
            endpoint,
            token_endpoint: bmap,
            desc_hash,
            dtoken_contract_address,
            mp_contract_address,
            split_policy_contract_address,
        })
    }
}

#[derive(Clone)]
pub enum RT {
    RTStaticFile,
}

impl Encoder for RT {
    fn encode(&self, sink: &mut Sink) {
        match self {
            RT::RTStaticFile => {
                sink.write(0u8);
            }
        }
    }
}

impl<'a> Decoder<'a> for RT {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let u = source.read_byte()?;
        match u {
            0 => Ok(RT::RTStaticFile),
            _ => panic!(""),
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

#[derive(Clone)]
pub struct DTokenItem {
    pub fee: Fee,
    pub expired_date: u64,
    pub stocks: u32,
    pub templates: Vec<TokenTemplate>,
}
impl Encoder for DTokenItem {
    fn encode(&self, sink: &mut Sink) {
        sink.write(&self.fee);
        sink.write(self.expired_date);
        sink.write(self.stocks);
        sink.write(self.templates.len() as u32);
        for token in self.templates.iter() {
            sink.write(token);
        }
    }
}

impl<'a> Decoder<'a> for DTokenItem {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let fee: Fee = source.read()?;
        let expired_date: u64 = source.read()?;
        let stocks: u32 = source.read()?;
        let mut templates: Vec<TokenTemplate> = vec![];
        let l: u32 = source.read()?;
        for _ in 0..l {
            let v = source.read()?;
            templates.push(v);
        }
        Ok(DTokenItem {
            fee,
            expired_date,
            stocks,
            templates,
        })
    }
}

#[derive(Encoder, Decoder, Clone)]
pub struct Fee {
    pub contract_addr: Address,
    pub contract_type: TokenType,
    pub count: u64,
}

#[derive(Clone)]
pub enum TokenType {
    ONT,
    ONG,
    OEP4,
}

impl Encoder for TokenType {
    fn encode(&self, sink: &mut Sink) {
        match self {
            TokenType::ONT => {
                sink.write(0u8);
            }
            TokenType::ONG => {
                sink.write(1u8);
            }
            TokenType::OEP4 => {
                sink.write(2u8);
            }
        }
    }
}

impl<'a> Decoder<'a> for TokenType {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let ty: u8 = source.read().unwrap();
        match ty {
            0u8 => Ok(TokenType::ONT),
            1u8 => Ok(TokenType::ONG),
            2u8 => Ok(TokenType::OEP4),
            _ => {
                panic!("");
            }
        }
    }
}
