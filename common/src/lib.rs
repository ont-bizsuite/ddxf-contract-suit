#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use core::option::Option;
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::prelude::*;

#[cfg(test)]
mod test;

#[derive(Encoder, Decoder)]
pub struct OrderId {
    item_id:Vec<u8>,
    tx_hash:H256,
}

#[derive(Clone, Debug, Ord, Eq, PartialEq, PartialOrd, Encoder, Decoder)]
pub struct TokenTemplate {
    pub data_id: Option<Vec<u8>>,
    pub token_hash: Vec<Vec<u8>>,
}

impl TokenTemplate {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut source = Source::new(data);
        source.read().unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut sink = Sink::new(16);
        sink.write(self);
        sink.bytes().to_vec()
    }
}

impl TokenTemplate {
    pub fn new(data_id: Option<Vec<u8>>, token_hash: Vec<Vec<u8>>) -> Self {
        TokenTemplate {
            data_id,
            token_hash,
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
