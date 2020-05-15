#![cfg_attr(not(feature = "mock"), no_std)]
#![feature(proc_macro_hygiene)]
extern crate ontio_std as ostd;

use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use ostd::prelude::*;

#[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct TokenTemplate {
    pub data_ids: Option<Vec<u8>>,
    pub token_hash: Vec<u8>,
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

impl Encoder for TokenTemplate {
    fn encode(&self, sink: &mut Sink) {
        if let Some(data_ids) = &self.data_ids {
            sink.write(true);
            sink.write(data_ids);
        } else {
            sink.write(false);
        }
        sink.write(&self.token_hash);
    }
}

impl<'a> Decoder<'a> for TokenTemplate {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let is_val: bool = source.read()?;
        let data_ids = match is_val {
            true => {
                let data_ids: Vec<u8> = source.read()?;
                Some(data_ids)
            }
            false => None,
        };
        let token_hash: Vec<u8> = source.read()?;
        Ok(TokenTemplate {
            data_ids,
            token_hash,
        })
    }
}

impl TokenTemplate {
    pub fn new(data_ids: Option<Vec<u8>>, token_hash: Vec<u8>) -> Self {
        TokenTemplate {
            data_ids,
            token_hash,
        }
    }

    pub fn split(&mut self) -> Vec<Vec<u8>> {
        let mut res: Vec<Vec<u8>> = vec![];
        let mut item: Vec<u8> = vec![];
        if let Some(data_ids) = &self.data_ids {
            for &i in data_ids.iter() {
                if i == ';' as u8 {
                    res.push(item.clone());
                    item.clear();
                } else {
                    item.push(i);
                }
            }
        }
        res
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut sink = Sink::new(16);
        if let Some(data_ids) = &self.data_ids {
            sink.write(true);
            sink.write(data_ids);
        } else {
            sink.write(false);
        }
        sink.write(&self.token_hash);
        sink.bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
